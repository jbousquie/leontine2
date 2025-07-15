//! Settings panel component
//! Allows users to configure the API URL. The state is lifted to the parent component.

use crate::hooks::persistent::UsePersistent;
use crate::state::{ApiConnectionStatus, AppState};
use dioxus::prelude::*;

/// Settings panel component for managing API configuration.
/// It receives the shared `api_url` state as a prop from its parent.
#[component]
pub fn SettingsPanel(mut api_url: UsePersistent<String>) -> Element {
    // The `input_value` signal holds the temporary state of the input field.
    // This separation prevents unnecessary side effects during typing.
    let mut input_value = use_signal(|| api_url.get());

    // Track if the input has been modified from the saved value.
    let mut is_modified = use_signal(|| false);

    // `use_effect` to reactively check if the input value differs from the persisted one.
    // This runs whenever `input_value` or `api_url` changes.
    use_effect(move || {
        is_modified.set(input_value.read().as_str() != api_url.get().as_str());
    });

    let app_state = use_context::<AppState>();
    let api_connection_status = app_state.api_connection_status.read();

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
                    // On every keystroke, update the local `input_value` signal.
                    // This does NOT modify the persistent state yet.
                    oninput: move |evt| {
                        input_value.set(evt.value());
                    }
                }

                // Conditionally render the "Save URL" button or "Saved" message
                // based on whether the input has been modified.
                if is_modified() {
                    button {
                        // When the user clicks "Save", commit the change to the persistent state.
                        // This updates the signal in the parent `App` component and writes to localStorage.
                        onclick: move |_| {
                            api_url.set(input_value.read().clone());
                            log::info!("New API URL saved: {}", api_url.get());
                        },
                        "Save URL"
                    }
                } else {
                    span {
                        class: "saved-message",
                        "Saved"
                    }
                }
                match &*api_connection_status {
                    ApiConnectionStatus::Pending => rsx! {
                        span { class: "status-text yellow", "Checking API status..." }
                    },
                    ApiConnectionStatus::Available(_, _) => rsx! {
                        span { class: "status-text green", "API is online" }
                    },
                    ApiConnectionStatus::Unavailable(_, _) => rsx! {
                        span { class: "status-text red", "API is unreachable" }
                    },
                }
            }
        }
    }
}
