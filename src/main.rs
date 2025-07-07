mod components;
mod config;

use components::settings::SettingsPanel;
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let title = "Leontine - Audio Transcription";

    rsx! {
        head {
            title { "{title}" }
            link {
                rel: "stylesheet",
                href: "/assets/main.css"
            }
        }

        // Main application layout
        main {
            class: "app-container",

            header {
                class: "app-header",
                h1 { "{title}" }
            }

            // Settings section - our first implemented component
            section {
                class: "settings-section",
                SettingsPanel {}
            }
        }
    }
}
