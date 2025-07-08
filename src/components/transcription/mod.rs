//! Transcription panel component
//! Manages file selection, state, and actions for transcription.

use crate::api::{self, ApiError, TranscriptionJob};
use crate::document::eval;
use crate::hooks::persistent::UsePersistent;
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_elements::FileEngine;
use log::info;
use std::sync::Arc;

/// Represents the current state of the transcription UI, acting as a simple state machine.
#[derive(Clone, PartialEq, Debug)]
enum TranscriptionStatus {
    Idle,
    FileSelected,
    Transcribing,
    Completed(Result<TranscriptionJob, ApiError>),
}

// --- Component Props ---

#[derive(Props, Clone, PartialEq)]
pub struct TranscriptionPanelProps {
    pub api_url: UsePersistent<String>,
}

/// The main transcription panel, which combines file upload, state management, and action buttons.
#[component]
pub fn TranscriptionPanel(props: TranscriptionPanelProps) -> Element {
    // --- State Signals ---
    let mut status = use_signal(|| TranscriptionStatus::Idle);
    let mut selected_file: Signal<Option<Arc<dyn FileEngine>>> = use_signal(|| None);
    let mut is_dragging = use_signal(|| false);

    let api_url_prop = props.api_url;

    // --- Resource for the API call ---
    // This resource will re-run when `status` changes to `Transcribing`.
    use_resource(move || async move {
        if *status.read() != TranscriptionStatus::Transcribing {
            return;
        }

        info!("Transcription process triggered.");

        let file_to_upload = selected_file.read().clone();
        let api_url = api_url_prop.get();

        if let Some(file) = file_to_upload {
            // Call the API function to submit the file.
            let result = api::submit_transcription(&api_url, &file).await;
            // Update the status with the result of the API call.
            status.set(TranscriptionStatus::Completed(result));
        } else {
            // This is an inconsistent state; reset with an error.
            status.set(TranscriptionStatus::Completed(Err(
                ApiError::FileNotAvailable,
            )));
        }
    });

    // --- Event Handlers and Helpers ---

    // A helper to determine if the UI should be locked from user input.
    let is_locked = move || {
        matches!(
            *status.read(),
            TranscriptionStatus::Transcribing | TranscriptionStatus::Completed(_)
        )
    };

    // Handles file selection from either drag-drop or the file input.
    let mut handle_file_selection = move |file_engine: Arc<dyn FileEngine>| {
        if is_locked() {
            return;
        }
        if !file_engine.files().is_empty() {
            selected_file.set(Some(file_engine));
            status.set(TranscriptionStatus::FileSelected);
        }
    };

    // Resets the component to its initial state.
    let reset_state = move |_| {
        selected_file.set(None);
        status.set(TranscriptionStatus::Idle);
        // Clear the native file input's value to allow re-selecting the same file.
        let _ = eval(r#"document.getElementById('file-upload-input').value = '';"#);
    };

    // --- Dynamic CSS classes ---
    let mut upload_area_class = String::from("upload-area");
    if is_dragging() && !is_locked() {
        upload_area_class.push_str(" dragging");
    }
    if is_locked() {
        upload_area_class.push_str(" disabled");
    }

    rsx! {
        div {
            class: "transcription-panel",
            h2 { "Transcription" }
            div {
                class: "{upload_area_class}",

                // --- Drag-and-Drop Event Handlers ---
                ondragover: move |evt| {
                    if !is_locked() {
                        evt.prevent_default();
                        is_dragging.set(true);
                    }
                },
                ondragleave: move |evt| {
                    if !is_locked() {
                        evt.prevent_default();
                        is_dragging.set(false);
                    }
                },
                ondrop: move |evt| {
                    evt.prevent_default();
                    if !is_locked() {
                        is_dragging.set(false);
                        if let Some(file_engine) = evt.files() {
                            handle_file_selection(file_engine);
                        }
                    }
                },

                // --- Hidden file input for the button ---
                input {
                    r#type: "file",
                    id: "file-upload-input",
                    accept: "audio/*",
                    multiple: false,
                    disabled: is_locked(),
                    style: "display: none;",
                    onchange: move |evt| {
                         if let Some(file_engine) = evt.files() {
                            handle_file_selection(file_engine);
                        }
                    },
                },

                // --- Main UI Content (changes based on state) ---
                div {
                    class: "upload-content",

                    // Persistently show the selected file name if it exists
                    if let Some(file_engine) = selected_file() {
                        if let Some(file_name) = file_engine.files().first() {
                             p { "Selected file: ", strong { "{file_name}" } }
                        }
                    }

                    // Render different controls and messages based on the current status
                    match status() {
                        TranscriptionStatus::Idle => rsx!{
                            p { "Drag and drop an audio file here, or click the button below." }
                            button {
                                onclick: move |_| {
                                    let _ = eval(r#"document.getElementById('file-upload-input').click();"#);
                                },
                                "Select Audio File"
                            }
                        },
                        TranscriptionStatus::FileSelected => rsx!{
                            div {
                                class: "action-buttons",
                                button {
                                    class: "button-clear",
                                    onclick: reset_state,
                                    "Clear Selection"
                                }
                                button {
                                    class: "button-transcribe",
                                    onclick: move |_| status.set(TranscriptionStatus::Transcribing),
                                    "Transcribe Audio"
                                }
                            }
                        },
                        TranscriptionStatus::Transcribing => rsx!{
                            p { class: "transcribing-message", "Submitting job... Please wait." }
                        },
                        TranscriptionStatus::Completed(result) => {
                            // Render the result of the API call
                            let result_display = match result {
                                Ok(job) => rsx! {
                                    div { class: "success-message",
                                        p { "Job submitted successfully!" }
                                        p { "Job ID: ", code { "{job.job_id}" } }
                                        p { "Status: ", code { "{job.job_status}" } }
                                    }
                                },
                                Err(err) => rsx! {
                                    div { class: "error-message",
                                        p { "Failed to submit transcription job:" }
                                        p { code { "{err:?}" } }
                                    }
                                }
                            };
                            rsx! {
                                {result_display},
                                div {
                                    class: "action-buttons",
                                    button {
                                        class: "button-new",
                                        onclick: reset_state,
                                        "Start New Transcription"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
