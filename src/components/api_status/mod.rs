//! API Status panel component
//! Displays the current status of the WhisperX API connection

use dioxus::prelude::*;
use gloo::timers::callback::{Interval, Timeout};
use log::{error, info};
use web_sys::js_sys::Date; // Use JavaScript's Date object for WASM compatibility

use crate::api::{get_status, ApiError};
use crate::config::{API_STATUS_CHECK_INTERVAL_MS, API_STATUS_INITIAL_CHECK_MS};
use crate::hooks::persistent::use_persistent;

/// API Status Panel component
#[component]
pub fn ApiStatus() -> Element {
    // Get the API URL from persistent storage
    let api_url = use_persistent("api_url", || "".to_string());

    // Define component state signals
    let mut flag_color = use_signal(|| "gray".to_string());
    let mut status_message = use_signal(|| "API URL not configured".to_string());
    let mut queue_info = use_signal(|| String::new());
    let mut last_checked = use_signal(|| String::new());

    // This signal acts as a trigger to re-run the `use_resource` hook
    let mut refresh_trigger = use_signal(|| 0);

    // If URL is empty, set initial UI state
    if api_url.get().is_empty() {
        flag_color.set("gray".to_string());
        status_message.set("API URL not configured".to_string());
        queue_info.set(String::new());
    }

    // Function to format a timestamp using WASM-compatible APIs
    fn format_timestamp() -> String {
        // Get milliseconds since UNIX epoch from JavaScript
        let now_ms = Date::now();
        // Convert to seconds and nanoseconds for chrono
        let now_secs = (now_ms / 1000.0) as i64;
        let now_nanos = ((now_ms % 1000.0) * 1_000_000.0) as u32;

        // Create a DateTime object and format it to H:M:S
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(now_secs, now_nanos)
            .unwrap_or_default()
            .format("%H:%M:%S")
            .to_string();

        format!("Last checked at {}", dt)
    }

    // `use_resource` to fetch API status reactively
    let api_status_resource = use_resource(move || {
        let url = api_url.get();
        // Depend on the trigger. When it changes, this resource re-runs.
        let _ = refresh_trigger();

        async move {
            if url.is_empty() {
                return Err(ApiError::RequestFailed(
                    "API URL not configured".to_string(),
                ));
            }

            // Set UI to a loading state before the request
            flag_color.set("yellow".to_string());
            status_message.set("Checking API status...".to_string());

            // Update timestamp at the start of the check
            last_checked.set(format_timestamp());

            // Make the API request
            let result = get_status(&url).await;
            info!("API status check completed: {:?}", result.is_ok());
            result
        }
    });

    // `use_effect` to handle the result of the API call from the resource
    use_effect(move || {
        // Match on a reference to the resource's value to avoid move errors
        match &*api_status_resource.value().read() {
            Some(Ok(status)) => {
                info!(
                    "API online: {} queued jobs, {} processing",
                    status.queue_state.queued_jobs, status.queue_state.processing_jobs
                );

                flag_color.set("green".to_string());
                status_message.set("API Online".to_string());
                queue_info.set(format!(
                    "{} jobs in queue, {} jobs processing",
                    status.queue_state.queued_jobs, status.queue_state.processing_jobs
                ));
            }
            Some(Err(err)) => {
                // Clone error data because we are borrowing it from the resource
                let error_msg = match err {
                    ApiError::RequestFailed(msg) => format!("Request failed: {}", msg.clone()),
                    ApiError::HttpError(code, msg) => {
                        format!("HTTP error {}: {}", code, msg.clone())
                    }
                    ApiError::ParseError(msg) => format!("Invalid API response: {}", msg.clone()),
                };

                error!("API status check failed: {}", error_msg);
                flag_color.set("red".to_string());
                status_message.set(error_msg);
                queue_info.set(String::new());
            }
            None => {
                // Resource is loading, UI state is set inside the resource's future
            }
        }
    });

    // `use_effect` to set up timers for initial and periodic checks
    use_effect(move || {
        // Set up initial check with a short delay
        let initial_timeout = Timeout::new(API_STATUS_INITIAL_CHECK_MS as u32, move || {
            info!(
                "Initial API status check triggered after {}ms",
                API_STATUS_INITIAL_CHECK_MS
            );
            // Correctly update the signal to avoid rules of hooks violation
            *refresh_trigger.write() += 1;
        });

        // Set up periodic check
        info!(
            "Setting up periodic API check every {}ms",
            API_STATUS_CHECK_INTERVAL_MS
        );

        let interval = Interval::new(API_STATUS_CHECK_INTERVAL_MS as u32, move || {
            info!("Triggering periodic API status check");
            // Correctly update the signal
            *refresh_trigger.write() += 1;
        });

        // Return a cleanup function to clear timers when component unmounts
        (move || {
            info!("Component unmounting, clearing timers");
            drop(initial_timeout);
            drop(interval);
        })()
    });

    // Render the component's UI
    rsx! {
        div { class: "api-status",
            div { class: "status-flag",
                div { class: "flag-icon {flag_color()}" }
                div { class: "status-text",
                    span { class: "status-label", "{status_message()}" }

                    // Show queue info if available
                    if !queue_info().is_empty() {
                        span { class: "queue-info", "{queue_info()}" }
                    }

                    if !last_checked().is_empty() {
                        div { class: "timestamp", "{last_checked()}" }
                    }

                    // Debug button to manually refresh
                    button {
                        onclick: move |_| {
                            info!("Manual refresh triggered");
                            // Correctly update the signal
                            *refresh_trigger.write() += 1;
                        },
                        "Refresh Status"
                    }
                }
            }
        }
    }
}
