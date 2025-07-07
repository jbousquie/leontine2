mod components;
mod config;
mod hooks;

use components::settings::SettingsPanel;
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
        }

        // Application container
        div {
            class: "app-container",

            header {
                class: "app-header",
                h1 { "{title}" }
            }

            // Settings section - our first component
            section {
                class: "settings-section",
                SettingsPanel {}
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
