//! Settings panel component
//! Allows users to configure and persist the API URL

use crate::config::DEFAULT_API_URL;
use dioxus::prelude::*;

/// Settings panel component for managing API configuration
#[component]
pub fn SettingsPanel() -> Element {
    // In Dioxus 0.6, we use use_signal for reactive state
    let mut api_url = use_signal(|| DEFAULT_API_URL.to_string());
    let mut input_value = use_signal(|| api_url().clone());
    let mut show_saved = use_signal(|| false);

    // Reset the saved message after a delay
    use_effect(move || {
        if show_saved() {
            let timeout = gloo::timers::callback::Timeout::new(2000, move || {
                show_saved.set(false);
            });

            // Return cleanup function
            return (move || {
                drop(timeout);
            })();
        }
    });

    rsx! {
        div {
            class: "settings-panel",
            h2 { "Settings" }

            div {
                class: "settings-content",
                label {
                    r#for: "api-url",
                    "API URL:"
                }
                input {
                    id: "api-url",
                    r#type: "text",
                    value: "{input_value}",
                    onchange: move |evt| {
                        input_value.set(evt.value().clone());
                    }
                }
                button {
                    onclick: move |_| {
                        api_url.set(input_value().clone());
                        // TODO: Add proper localStorage implementation later
                        log::info!("URL saved: {}", api_url());
                        show_saved.set(true);
                    },
                    "Save URL"
                }

                // Show saved message
                if show_saved() {
                    span {
                        class: "saved-message",
                        "Saved!"
                    }
                }
            }
        }
    }
}
