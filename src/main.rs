mod api;
mod components;
mod config;
mod hooks;

use crate::config::DEFAULT_API_URL;
use crate::hooks::persistent::use_persistent;
use dioxus::prelude::*;

fn main() {
    // Initialize the logger for wasm-bindgen.
    // This allows `log::info!`, `log::error!`, etc. to work in the browser console.
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Logger initialized. Starting Leontine application...");

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let title = "Leontine - Audio Transcription";

    // --- Lifted State ---
    // The `api_url` state is owned by the parent `App` component.
    // It is the single source of truth for the entire application.
    let api_url = use_persistent("api_url", || DEFAULT_API_URL.to_string());

    rsx! {
        // Set up document head
        head {
            title { "{title}" }
            style {
                {include_str!("../assets/main.css")}
            }
            style {
                {include_str!("../assets/api-status.css")}
            }
        }

        // Application container
        div {
            class: "app-container",

            header {
                class: "app-header",
                h1 { "{title}" }
            }

            // --- Child Components ---
            // The shared `api_url` state is passed down to child components as a prop.
            section {
                class: "settings-section",
                // The `SettingsPanel` receives the signal to read and write the URL.
                components::settings::SettingsPanel { api_url: api_url }
            }

            section {
                class: "api-status-section",
                // The `ApiStatus` component receives the signal to read the URL.
                components::api_status::ApiStatus { api_url: api_url }
            }

            footer {
                class: "app-footer",
                p {
                    "Powered by "
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
