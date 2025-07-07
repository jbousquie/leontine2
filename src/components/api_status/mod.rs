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

    // --- State Signals ---
    let mut flag_color = use_signal(|| "gray".to_string());
    let mut status_message = use_signal(|| "API URL not configured".to_string());
    let mut queue_info = use_signal(|| String::new());
    let mut last_checked = use_signal(|| String::new());

    // This signal acts as a trigger to re-run the `use_resource` hook
    let mut refresh_trigger = use_signal(|| 0);

    // Signals to hold the timer handles. This makes them part of the component's state,
    // protecting them from being dropped during re-renders.
    let mut initial_timeout_handle = use_signal(|| None::<Timeout>);
    let mut interval_handle = use_signal(|| None::<Interval>);

    // If URL is empty, set initial UI state
    if api_url.get().is_empty() {
        flag_color.set("gray".to_string());
        status_message.set("API URL not configured".to_string());
        queue_info.set(String::new());
    }

    // --- Helper Functions ---
    // Function to format a timestamp using WASM-compatible APIs
    fn format_timestamp() -> String {
        let now_ms = Date::now();
        let now_secs = (now_ms / 1000.0) as i64;
        let now_nanos = ((now_ms % 1000.0) * 1_000_000.0) as u32;

        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(now_secs, now_nanos)
            .unwrap_or_default()
            .format("%H:%M:%S")
            .to_string();

        format!("Last checked at {}", dt)
    }

    // --- Data Fetching and Effects ---
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
            last_checked.set(format_timestamp());

            get_status(&url).await
        }
    });

    // `use_effect` to handle the result of the API call from the resource
    use_effect(move || {
        // Match on a reference to the resource's value to avoid move errors
        if let Some(result) = api_status_resource.value().read().as_ref() {
            match result {
                Ok(status) => {
                    info!(
                        "API online: {} queued, {} processing",
                        status.queue_state.queued_jobs, status.queue_state.processing_jobs
                    );
                    flag_color.set("green".to_string());
                    status_message.set("API Online".to_string());
                    queue_info.set(format!(
                        "{} jobs in queue, {} jobs processing",
                        status.queue_state.queued_jobs, status.queue_state.processing_jobs
                    ));
                }
                Err(err) => {
                    let error_msg = match err {
                        ApiError::RequestFailed(msg) => format!("Request failed: {}", msg.clone()),
                        ApiError::HttpError(code, msg) => {
                            format!("HTTP error {}: {}", code, msg.clone())
                        }
                        ApiError::ParseError(msg) => {
                            format!("Invalid API response: {}", msg.clone())
                        }
                    };
                    error!("API status check failed: {}", error_msg);
                    flag_color.set("red".to_string());
                    status_message.set(error_msg);
                    queue_info.set(String::new());
                }
            }
        }
    });

    // `use_hook` runs its closure once on component mount. It is the correct
    // place to set up timers, subscriptions, or other side effects that
    // should not be re-run on every render.
    use_hook(move || {
        info!("Setting up API check timers (this will only happen once).");

        // Set up initial check with a short delay
        let initial_timeout = Timeout::new(API_STATUS_INITIAL_CHECK_MS as u32, move || {
            info!(
                "Initial API status check triggered after {}ms",
                API_STATUS_INITIAL_CHECK_MS
            );
            *refresh_trigger.write() += 1;
        });
        initial_timeout_handle.set(Some(initial_timeout));

        // Set up periodic check
        let interval = Interval::new(API_STATUS_CHECK_INTERVAL_MS as u32, move || {
            *refresh_trigger.write() += 1;
        });
        interval_handle.set(Some(interval));
    });

    // `use_drop` provides a dedicated place for cleanup logic to run when the
    // component is unmounted.
    use_drop(move || {
        info!("Component unmounting, clearing timers.");
        // Taking the value out of the signal will cause the handle to be dropped,
        // which correctly cancels the timer.
        initial_timeout_handle.take();
        interval_handle.take();
    });

    // --- Render ---
    rsx! {
        div { class: "api-status",
            div { class: "status-flag",
                div { class: "flag-icon {flag_color()}" }
                div { class: "status-text",
                    span { class: "status-label", "{status_message()}" }

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
                            *refresh_trigger.write() += 1;
                        },
                        "Refresh Status"
                    }
                }
            }
        }
    }
}
