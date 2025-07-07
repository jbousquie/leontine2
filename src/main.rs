mod api;
mod components;
mod config;
mod hooks;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let title = "Leontine - Audio Transcription";

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

            // Settings section
            section {
                class: "settings-section",
                components::settings::SettingsPanel {}
            }

            // API Status section
            section {
                class: "api-status-section",
                components::api_status::ApiStatus {}
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
