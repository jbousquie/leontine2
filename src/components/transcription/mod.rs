//! Transcription panel component
//! Manages file selection, state, and actions for transcription.

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
    // This resource will re-run when `status` changes.
    let _transcription_resource = use_resource(move || async move {
        // We only want to run the logic when the status changes to `Transcribing`.
        if *status.read() != TranscriptionStatus::Transcribing {
            return;
        }

        info!("Transcription process triggered.");

        let file_to_upload = selected_file.read().clone();
        let api_url = api_url_prop.get();

        if let Some(file) = file_to_upload {
            if let Some(file_name) = file.files().first() {
                info!(
                    "Preparing to transcribe file '{}' using API at '{}'",
                    file_name, api_url
                );
                // --- TODO: Implement the actual file upload and API call here ---
            }
        } else {
            // This is an inconsistent state. If we are 'Transcribing' but have no file, reset.
            status.set(TranscriptionStatus::Idle);
        }
    });

    // --- Event Handlers ---
    let is_transcribing = move || *status.read() == TranscriptionStatus::Transcribing;

    let mut handle_file_selection = move |file_engine: Arc<dyn FileEngine>| {
        if is_transcribing() {
            return;
        }
        if !file_engine.files().is_empty() {
            selected_file.set(Some(file_engine));
            status.set(TranscriptionStatus::FileSelected);
        }
    };

    let on_drop = move |evt: DragEvent| {
        evt.prevent_default();
        if is_transcribing() {
            return;
        }
        is_dragging.set(false);
        if let Some(file_engine) = evt.files() {
            handle_file_selection(file_engine);
        }
    };

    let on_file_change = move |evt: FormEvent| {
        if is_transcribing() {
            return;
        }
        if let Some(file_engine) = evt.files() {
            handle_file_selection(file_engine);
        }
    };

    let on_clear_or_cancel = move |_| {
        // Reset Dioxus state
        selected_file.set(None);
        status.set(TranscriptionStatus::Idle);

        // Reset the native file input's value to allow re-selection of the same file.
        let _ = eval(r#"document.getElementById('file-upload-input').value = '';"#);

        // Note: For a real implementation, we would also need to cancel the running `use_resource` future.
        // Dioxus 0.6 will make this easier with `use_future` and `use_coroutine`.
    };

    // --- Dynamic CSS classes for the upload area ---
    let mut upload_area_class = String::from("upload-area");
    if is_dragging() && !is_transcribing() {
        upload_area_class.push_str(" dragging");
    }
    if is_transcribing() {
        // This can be used to style the area as disabled, e.g., with lower opacity.
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
                    if !is_transcribing() {
                        evt.prevent_default();
                        is_dragging.set(true);
                    }
                },
                ondragleave: move |evt| {
                    if !is_transcribing() {
                        evt.prevent_default();
                        is_dragging.set(false);
                    }
                },
                ondrop: on_drop,

                // --- Hidden file input for the button ---
                input {
                    r#type: "file",
                    id: "file-upload-input",
                    accept: "audio/*",
                    multiple: false,
                    disabled: is_transcribing(),
                    style: "display: none;",
                    onchange: on_file_change,
                },

                // --- UI Content based on State ---
                div {
                    class: "upload-content",

                    // Show selected file name when available
                    if let Some(file_engine) = selected_file() {
                        if let Some(file_name) = file_engine.files().first() {
                             p { "Selected file: ", strong { "{file_name}" } }
                        }
                    }

                    // Show content and buttons based on the current status
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
                                    onclick: on_clear_or_cancel,
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
                            p { class: "transcribing-message", "Transcribing... Please wait." }
                            button {
                                class: "button-cancel",
                                onclick: on_clear_or_cancel,
                                "Cancel"
                            }
                        },
                    }
                }
            }
        }
    }
}
