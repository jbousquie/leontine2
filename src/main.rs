mod api;
mod components;
mod config;
mod hooks;
pub mod state;
use crate::config::DEFAULT_API_URL;
use crate::hooks::persistent::use_persistent;
use crate::state::{AppState, TranscriptionUiStatus};
use dioxus::prelude::*;

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
    let app_state = AppState {
        // Persisted state
        api_url: use_persistent("api_url", || DEFAULT_API_URL.to_string()),
        active_job: use_persistent("leontine-active-job", || None),
        // Volatile state
        api_status: use_signal(|| None),
        job_state: use_signal(|| None),
        transcription_ui_status: use_signal(TranscriptionUiStatus::default),
    };

    use_context_provider(|| app_state);

    // Set up periodic API status checking
    let api_url = app_state.api_url.get();
    let api_status = app_state.api_status;

    // Check API status on component mount
    use_effect(move || {
        to_owned![api_url, api_status];
        wasm_bindgen_futures::spawn_local(async move {
            if !api_url.is_empty() {
                match crate::api::get_status(&api_url).await {
                    Ok(status) => api_status.set(Some(Ok(status))),
                    Err(err) => api_status.set(Some(Err(err))),
                }
            }
        });
    });

    // API status polling setup
    let mut timer_handle = use_signal(|| None::<gloo_timers::callback::Interval>);

    // Initial API status check
    {
        let api_url_str = app_state.api_url.get().clone();
        let mut api_status_ref = app_state.api_status;

        use_effect(move || {
            let url = api_url_str.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if !url.is_empty() {
                    match crate::api::get_status(&url).await {
                        Ok(status) => api_status_ref.set(Some(Ok(status))),
                        Err(err) => api_status_ref.set(Some(Err(err))),
                    }
                }
            });
        });
    }

    // Set up periodic API status polling
    {
        let api_url_str = app_state.api_url.get().clone();
        let mut api_status_ref = app_state.api_status;

        use_effect(move || {
            // Create timer outside of reactive context
            let timer_fn = {
                let url = api_url_str.clone();
                let mut status = api_status_ref;
                move || {
                    let url_clone = url.clone();
                    let mut status_clone = status;
                    wasm_bindgen_futures::spawn_local(async move {
                        if !url_clone.is_empty() {
                            match crate::api::get_status(&url_clone).await {
                                Ok(s) => status_clone.set(Some(Ok(s))),
                                Err(e) => status_clone.set(Some(Err(e))),
                            }
                        }
                    });
                }
            };

            let handle = gloo_timers::callback::Interval::new(30_000, timer_fn);
            timer_handle.set(Some(handle));

            // Return nothing - we'll handle cleanup separately
        });
    }

    // Cleanup timer when component is unmounted
    use_drop(move || {
        if let Some(handle) = timer_handle.take() {
            handle.cancel();
        }
    });

    rsx! {head {
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
