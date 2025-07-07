//! API Status panel component
//! Displays the current status of the WhisperX API connection

use dioxus::prelude::*;
use log::{debug, error, info};

use crate::api::{get_status, ApiError};
use crate::hooks::persistent::use_persistent;

/// API Status Panel component
#[component]
pub fn ApiStatus() -> Element {
    // Get the API URL from persistent storage
    let api_url = use_persistent("api_url", || "".to_string());

    // Define component state
    let flag_color = use_signal(|| "gray".to_string());
    let status_message = use_signal(|| "API URL not configured".to_string());
    let queue_info = use_signal(|| String::new());

    // Check API status when component mounts or changes
    use_effect(move || {
        let url = api_url.get();
        to_owned![flag_color, status_message, queue_info, url];

        spawn(async move {
            // Skip if no URL is configured
            if url.is_empty() {
                return;
            }

            // Show loading state
            flag_color.set("yellow".to_string());
            status_message.set("Checking API status...".to_string());

            // Make the API request using the API client module
            debug!("Checking API status for URL: {}", url);

            match get_status(&url).await {
                Ok(status) => {
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
                Err(err) => {
                    // Handle different error types
                    let error_msg = match err {
                        ApiError::RequestFailed(msg) => format!("Request failed: {}", msg),
                        ApiError::HttpError(code, msg) => format!("HTTP error {}: {}", code, msg),
                        ApiError::ParseError(msg) => format!("Invalid API response: {}", msg),
                    };

                    error!("API status check failed: {}", error_msg);
                    flag_color.set("red".to_string());
                    status_message.set(error_msg);
                    queue_info.set(String::new());
                }
            }
        });

        // Return empty cleanup function
        (|| {})()
    });

    // Render component
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
                }
            }
        }
    }
}
