mod api;
mod components;
mod config;
mod hooks;
pub mod state;

use crate::config::{API_STATUS_CHECK_INTERVAL_MS, DEFAULT_API_URL};
use crate::hooks::persistent::use_persistent;
use crate::state::{ApiConnectionStatus, AppState, TranscriptionUiStatus};
use dioxus::prelude::*;
use gloo_timers::callback::Interval;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Logger initialized. Starting Leontine application...");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let title = "Leontine - Audio Transcription";

    // --- Global State Initialization ---
    // All shared state is created here and provided to the context.
    let mut app_state = AppState {
        api_url: use_persistent("api_url", || DEFAULT_API_URL.to_string()),
        active_job: use_persistent("leontine-active-job", || None),
        api_connection_status: use_signal(ApiConnectionStatus::default),
        job_state: use_signal(|| None),
        transcription_ui_status: use_signal(TranscriptionUiStatus::default),
    };

    use_context_provider(|| app_state);

    // This resource will fetch the API status. It automatically re-runs whenever
    // its dependencies change (in this case, when api_url changes).
    let mut api_status_resource = use_resource(move || async move {
        let api_url = app_state.api_url.get();
        if api_url.is_empty() {
            return ApiConnectionStatus::Unavailable(
                crate::api::ApiError::RequestFailed("API URL is not configured".to_string()),
                chrono::Utc::now(),
            );
        }
        match crate::api::get_status(&api_url).await {
            Ok(status) => ApiConnectionStatus::Available(status, chrono::Utc::now()),
            Err(err) => ApiConnectionStatus::Unavailable(err, chrono::Utc::now()),
        }
    });

    // When the resource finishes fetching, we update the global state.
    // This effect runs whenever api_status_resource changes.
    use_effect(move || {
        if let Some(status) = api_status_resource.value().read().clone() {
            app_state.api_connection_status.set(status);
        }
    });

    // Set up a simple interval timer to periodically refresh the resource,
    // which keeps the API status up-to-date.
    use_effect(move || {
        let timer = Interval::new(API_STATUS_CHECK_INTERVAL_MS as u32, move || {
            // `restart` will cause the `use_resource` to run its future again.
            api_status_resource.restart();
        });

        // The on_cleanup function is crucial to prevent memory leaks.
        // It runs when the component is unmounted.
        use_drop(move || {
            timer.cancel();
        });
    });

    rsx! {
        head {
            title { "{title}" }
            link { rel: "stylesheet", href: "https://unpkg.com/sakura.css/css/sakura.css" }
            style { {include_str!("../assets/main.css")} }
        }

        div {
            class: "app-container",
            header { class: "app-header", h1 { "{title}" } }

            section {
                class: "settings-section",
                components::settings::SettingsPanel {
                    api_url: use_context::<AppState>().api_url
                }
            }

            section {
                class: "api-status-section",
                components::api_status::ApiStatusDisplay {}
            }

            section {
                class: "transcription-section",
                components::transcription::TranscriptionPanel {
                    api_url: use_context::<AppState>().api_url
                }
            }

            footer {
                class: "app-footer",
                p {
                    "Powered by ",
                    a {
                        href: "https://github.com/jbousquie/whisper_api",
                        target: "_blank",
                        "WhisperX API"
                    }
                }
            }
        }
    }
}
