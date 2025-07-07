//! Settings panel component
//! Allows users to configure and persist the API URL

use crate::config::DEFAULT_API_URL;
use crate::hooks::persistent::use_persistent;
use dioxus::prelude::*;

/// Settings panel component for managing API configuration
#[component]
pub fn SettingsPanel() -> Element {
    // Two-signal pattern for form handling:
    // api_url: The committed value used by the application (persisted value)
    // input_value: The current value in the input field (changes with each keystroke)
    // This separation prevents unnecessary side effects during typing and enables validation before saving
    let mut api_url = use_persistent("api_url", || DEFAULT_API_URL.to_string());
    let mut input_value = use_signal(|| api_url.get());
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
                        // Commit the input value to the application state and localStorage
                        api_url.set(input_value().clone());
                        log::info!("URL saved: {}", api_url.get());
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
