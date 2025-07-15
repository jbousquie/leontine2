//! API Status display component.
//! This component renders the API status based on the shared `ApiConnectionStatus` state.

use crate::state::{ApiConnectionStatus, AppState};
use dioxus::prelude::*;

/// A component to display the API status. It gets its data from the shared context.
#[component]
#[allow(non_snake_case)]
pub fn ApiStatusDisplay() -> Element {
    let app_state = use_context::<AppState>();
    // Read the connection status from the global state.
    let connection_status = app_state.api_connection_status.read();

    // Determine the display properties based on the current connection status.
    let (flag_color, status_message, queue_info, last_checked) = match &*connection_status {
        ApiConnectionStatus::Available(status, timestamp) => (
            "green",
            "API Online".to_string(),
            format!(
                "{} jobs in queue, {} jobs processing",
                status.queue_state.queued_jobs, status.queue_state.processing_jobs
            ),
            format!("Last checked at {}", timestamp.format("%H:%M:%S")),
        ),
        ApiConnectionStatus::Unavailable(err, timestamp) => (
            "red",
            err.to_string(),
            "".to_string(),
            format!("Last check failed at {}", timestamp.format("%H:%M:%S")),
        ),
        ApiConnectionStatus::Pending => (
            "yellow",
            "Checking API status...".to_string(),
            "".to_string(),
            "".to_string(), // No timestamp when pending
        ),
    };

    rsx! {
        div { class: "api-status",
            div { class: "status-flag",
                div { class: "flag-icon {flag_color}" }
                div { class: "status-text",
                    span { class: "status-label", "{status_message}" }

                    if !queue_info.is_empty() {
                        span { class: "queue-info", "{queue_info}" }
                    }

                    if !last_checked.is_empty() {
                        div { class: "timestamp", "{last_checked}" }
                    }
                }
            }
        }
    }
}
