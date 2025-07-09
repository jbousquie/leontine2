//! API Status display component.
//! This is a "dumb" component that simply renders the API status it receives as a prop.

use crate::state::AppState;
use dioxus::prelude::*;
use web_sys::js_sys::Date;

/// A component to display the API status.
#[component]
#[allow(non_snake_case)]
pub fn ApiStatusDisplay() -> Element {
    let app_state = use_context::<AppState>();
    let api_status = app_state.api_status.read();
    let (flag_color, status_message, queue_info) = match &*api_status {
        Some(Ok(status)) => (
            "green",
            "API Online".to_string(),
            format!(
                "{} jobs in queue, {} jobs processing",
                status.queue_state.queued_jobs, status.queue_state.processing_jobs
            ),
        ),
        Some(Err(err)) => ("red", err.to_string(), "".to_string()),
        None => (
            "yellow",
            "Checking API status...".to_string(),
            "".to_string(),
        ),
    };

    // We only show the timestamp if a check has actually been performed.
    let last_checked = if api_status.is_some() {
        let now_ms = Date::now();
        let now_secs = (now_ms / 1000.0) as i64;
        let now_nanos = ((now_ms % 1000.0) * 1_000_000.0) as u32;

        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(now_secs, now_nanos)
            .unwrap_or_default()
            .format("%H:%M:%S")
            .to_string();

        format!("Last checked at {}", dt)
    } else {
        // If there's no status, we don't show a timestamp.
        "".to_string()
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
