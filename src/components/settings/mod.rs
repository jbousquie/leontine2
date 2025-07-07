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

    // Track if the input has been modified from the saved value
    let mut is_modified = use_signal(|| false);

    // Update is_modified when input changes
    use_effect(move || {
        is_modified.set(input_value() != api_url.get());
        (|| {})()
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
                    // Capture all input changes with a single handler
                    oninput: move |evt| {
                        input_value.set(evt.value().clone());
                        is_modified.set(evt.value() != api_url.get());
                    }
                }

                // Show either the Save button or the Saved message
                if is_modified() {
                    button {
                        onclick: move |_| {
                            // Commit the input value to the application state and localStorage
                            api_url.set(input_value().clone());
                            log::info!("URL saved: {}", api_url.get());
                            is_modified.set(false);
                        },
                        "Save URL"
                    }
                } else {
                    span {
                        class: "saved-message",
                        "Saved"
                    }
                }
            }
        }
    }
}
