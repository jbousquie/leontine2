//! Transcription panel component
//! Contains both the file upload area and the transcription status area
use crate::dioxus_elements::FileEngine;
use crate::document::eval;
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use std::sync::Arc;

/// The main transcription panel, which combines the file upload area and a message/status area.
#[component]
pub fn TranscriptionPanel() -> Element {
    // This signal holds the state of the selected file across child components.
    // Dioxus file events use Arc for thread-safety.
    let selected_file: Signal<Option<Arc<dyn FileEngine>>> = use_signal(|| None);
    // This signal tracks if a file is being dragged over the drop zone for UI feedback.
    let is_dragging = use_signal(|| false);

    rsx! {
        div {
            class: "transcription-panel",
            h2 { "Transcription" }
            // --- Child Components ---
            UploadArea {
                selected_file: selected_file,
                is_dragging: is_dragging,
            }
            MessageArea {
                selected_file: selected_file
            }
        }
    }
}

// --- Component Props ---

#[derive(Props, Clone, PartialEq)]
struct UploadAreaProps {
    selected_file: Signal<Option<Arc<dyn FileEngine>>>,
    is_dragging: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
struct MessageAreaProps {
    selected_file: Signal<Option<Arc<dyn FileEngine>>>,
}

// --- Private Child Components ---

/// A component for handling file uploads via drag-and-drop or a file input button.
#[component]
fn UploadArea(props: UploadAreaProps) -> Element {
    let mut selected_file = props.selected_file;
    let mut is_dragging = props.is_dragging;
    let dragging_class = if is_dragging() { "dragging" } else { "" };

    // --- Event Handlers ---

    let on_file_change = move |evt: FormEvent| {
        if let Some(file_engine) = evt.files() {
            if !file_engine.files().is_empty() {
                selected_file.set(Some(file_engine.clone()));
            }
        }
    };

    let on_drop = move |evt: DragEvent| {
        // Prevent the browser's default behavior for dropped files.
        evt.prevent_default();
        is_dragging.set(false);
        if let Some(file_engine) = evt.files() {
            if !file_engine.files().is_empty() {
                selected_file.set(Some(file_engine.clone()));
            }
        }
    };

    rsx! {
        // The main container for the drop zone
        div {
            class: "upload-area {dragging_class}",
            // Event handlers for drag-and-drop functionality
            ondragover: move |evt| {
                // Prevent the browser's default behavior to allow for a drop.
                evt.prevent_default();
                is_dragging.set(true);
            },
            ondragleave: move |evt| {
                // It's good practice to prevent default on all dnd events.
                evt.prevent_default();
                is_dragging.set(false);
            },
            ondrop: on_drop,

            // Hidden file input, triggered by the button
            input {
                r#type: "file",
                id: "file-upload-input",
                accept: "audio/*",
                multiple: false,
                style: "display: none;",
                onchange: on_file_change,
            },

            // --- Conditional Rendering ---
            // Render content based on whether a file has been selected
            if let Some(file_engine) = selected_file() {
                if let Some(file_name) = file_engine.files().first() {
                    div {
                        class: "upload-content",
                        p { "Selected file: ", strong { "{file_name}" } }
                        // Button to clear the selection and return to the initial state
                        button {
                            onclick: move |_| selected_file.set(None),
                            "Clear Selection"
                        }
                    }
                }
            } else {
                // Initial state: prompt the user to select a file
                div {
                    class: "upload-content",
                    p { "Drag and drop an audio file here, or click the button below." }
                    button {
                        onclick: move |_| {
                            // This JavaScript snippet programmatically clicks the hidden file input.
                            // We use `let _ =` to explicitly ignore the `Result`.
                            let _ = eval(r#"document.getElementById('file-upload-input').click();"#);
                        },
                        "Select Audio File"
                    }
                }
            }
        }
    }
}

/// A component to display contextual messages to the user.
#[component]
fn MessageArea(props: MessageAreaProps) -> Element {
    rsx! {
        div {
            // This area will be used later to display transcription progress and results.
            if props.selected_file.read().is_none() {
                p {
                    class: "status-message",
                    "Please select a file to begin transcription."
                }
            } else {
                // A placeholder message for when a file is ready
                p {
                    class: "status-message",
                    "File ready. The next step will be to add a 'Transcribe' button."
                }
            }
        }
    }
}
