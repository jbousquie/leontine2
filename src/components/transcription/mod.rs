//! Transcription panel component
//! Manages file selection, state, and actions for transcription.

use crate::api::{self, ApiError, JobState, JobStatus, TranscriptionJob};
use crate::document::eval;
use crate::hooks::persistent::use_persistent;
use crate::hooks::persistent::UsePersistent;
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_elements::FileEngine;
use gloo_timers::callback::Interval;
use log::{error, info};
use std::sync::Arc;

/// Represents the current state of the transcription UI, acting as a simple state machine.
#[derive(Clone, PartialEq, Debug)]
enum TranscriptionUiStatus {
    Idle,
    FileSelected,
    Submitting,
    Monitoring,
    Completed(String),
    Error(String),
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
    let mut ui_status = use_signal(|| TranscriptionUiStatus::Idle);
    let mut selected_file: Signal<Option<Arc<dyn FileEngine>>> = use_signal(|| None);
    let mut is_dragging = use_signal(|| false);

    // --- Persistent and Polled State ---
    let mut active_job: UsePersistent<Option<TranscriptionJob>> =
        use_persistent("leontine-active-job", || None);
    let mut job_state: Signal<Option<Result<JobState, ApiError>>> = use_signal(|| None);
    let refresh_trigger = use_signal(|| 0);
    let mut interval_timer: Signal<Option<Interval>> = use_signal(|| None);

    let api_url_prop = props.api_url;

    // --- Effect to check for an existing job on load ---
    use_effect(move || {
        if active_job.get().is_some() && *ui_status.peek() == TranscriptionUiStatus::Idle {
            info!("Found active job on load, starting monitoring.");
            ui_status.set(TranscriptionUiStatus::Monitoring);
        }
    });

    // --- Resource for the initial transcription submission ---
    use_resource(move || async move {
        if *ui_status.read() != TranscriptionUiStatus::Submitting {
            return;
        }

        info!("Transcription submission process triggered.");
        let file_to_upload = selected_file.read().clone();
        let api_url = api_url_prop.get();

        if let Some(file) = file_to_upload {
            let result = api::submit_transcription(&api_url, &file).await;
            match result {
                Ok(job) => {
                    info!("Job submitted successfully: {}", job.job_id);
                    active_job.set(Some(job));
                    ui_status.set(TranscriptionUiStatus::Monitoring);
                }
                Err(e) => {
                    error!("Job submission failed: {:?}", e);
                    ui_status.set(TranscriptionUiStatus::Error(e.to_string()));
                }
            }
        } else {
            ui_status.set(TranscriptionUiStatus::Error(
                "File not available for submission.".to_string(),
            ));
        }
    });

    // --- Resource for polling the job status ---
    let job_status_resource = use_resource(move || async move {
        refresh_trigger.into_value();

        if let Some(job) = active_job.get() {
            let api_url = api_url_prop.get();
            let result = api::get_job_status(&api_url, &job.job_id).await;
            let mut should_clear_job = false;

            match &result {
                Ok(state) => match state.status {
                    JobStatus::Completed => {
                        let result_data = state.data.clone().unwrap_or_else(|| {
                            "Transcription completed, but no data was returned.".to_string()
                        });
                        ui_status.set(TranscriptionUiStatus::Completed(result_data));
                        should_clear_job = true;
                    }
                    JobStatus::Failed => {
                        let error_message = state.data.as_deref().unwrap_or("No details provided.");
                        error!("Job {} failed on server: {}", job.job_id, error_message);
                        ui_status.set(TranscriptionUiStatus::Error(format!(
                            "Job failed: {}",
                            error_message
                        )));
                        should_clear_job = true;
                    }
                    _ => { /* Queued or Processing, just update state and continue */ }
                },
                Err(ApiError::HttpError(404, _)) => {
                    error!(
                        "Job {} not found on server. Clearing local job.",
                        job.job_id
                    );
                    ui_status.set(TranscriptionUiStatus::Error(
                        "The previous job was not found on the server. It may have expired."
                            .to_string(),
                    ));
                    should_clear_job = true;
                }
                Err(_) => { /* Other errors will just be displayed, and polling will continue */ }
            }

            job_state.set(Some(result));
            return should_clear_job;
        }

        false
    });

    // --- Effect to clear the job based on the resource result ---
    use_effect(move || {
        if let Some(should_clear) = job_status_resource.value().read().as_ref() {
            if *should_clear {
                info!("Clearing active job from persistent storage.");
                active_job.set(None);
            }
        }
    });

    // --- Store the previous UI status to detect changes ---
    let mut previous_status = use_signal(|| TranscriptionUiStatus::Idle);

    // Create a dedicated polling mechanism that doesn't trigger reactivity loops
    use_effect(move || {
        let current_status = ui_status.read().clone();

        // Only act when the status changes
        if current_status != *previous_status.read() {
            let is_monitoring = current_status == TranscriptionUiStatus::Monitoring;

            // Cancel existing timer if there is one
            if interval_timer.with(|timer| timer.is_some()) {
                if let Some(timer) = interval_timer.write().take() {
                    info!("Cancelling existing timer due to status change");
                    timer.cancel();
                }
            }

            // Create new timer only if we're in monitoring state
            if is_monitoring {
                info!("Starting polling timer for monitoring");

                // Create the timer outside of any reactive context
                let timer_fn = {
                    let mut refresh_clone = refresh_trigger.clone();
                    move || {
                        // Increment the trigger counter to initiate a new poll
                        refresh_clone += 1;
                    }
                };

                let new_timer = Interval::new(5000, timer_fn);
                *interval_timer.write() = Some(new_timer);
            }

            // Update previous status
            previous_status.set(current_status);
        }
    });

    // --- Final cleanup on unmount ---
    use_drop(move || {
        if let Some(timer) = interval_timer.write().take() {
            info!("Clearing timer on component drop.");
            timer.cancel();
        }
    });

    // --- Event Handlers and Helpers ---
    let is_locked = move || {
        !matches!(
            ui_status(),
            TranscriptionUiStatus::Idle | TranscriptionUiStatus::FileSelected
        )
    };

    let mut handle_file_selection = move |file_engine: Arc<dyn FileEngine>| {
        if is_locked() {
            return;
        }
        if !file_engine.files().is_empty() {
            selected_file.set(Some(file_engine));
            ui_status.set(TranscriptionUiStatus::FileSelected);
        }
    };

    let reset_state = move |_| {
        selected_file.set(None);
        ui_status.set(TranscriptionUiStatus::Idle);
        active_job.set(None);
        job_state.set(None);
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
                ondragover: move |evt| { if !is_locked() { evt.prevent_default(); is_dragging.set(true); } },
                ondragleave: move |evt| { if !is_locked() { evt.prevent_default(); is_dragging.set(false); } },
                ondrop: move |evt| {
                    evt.prevent_default();
                    if !is_locked() {
                        is_dragging.set(false);
                        if let Some(file_engine) = evt.files() { handle_file_selection(file_engine); }
                    }
                },
                input {
                    r#type: "file",
                    id: "file-upload-input",
                    accept: "audio/*",
                    multiple: false,
                    disabled: is_locked(),
                    style: "display: none;",
                    onchange: move |evt| { if let Some(file_engine) = evt.files() { handle_file_selection(file_engine); } },
                },
                div {
                    class: "upload-content",
                    if let Some(file_engine) = selected_file() {
                        if let Some(file_name) = file_engine.files().first() {
                            p { "Selected file: ", strong { "{file_name}" } }
                        }
                    }
                    match ui_status() {
                        TranscriptionUiStatus::Idle => rsx! {
                            p { "Drag and drop an audio file here, or click the button below." }
                            button { onclick: move |_| { let _ = eval(r#"document.getElementById('file-upload-input').click();"#); }, "Select Audio File" }
                        },
                        TranscriptionUiStatus::FileSelected => rsx! {
                            div {
                                class: "action-buttons",
                                button { class: "button-clear", onclick: reset_state, "Clear Selection" }
                                button { class: "button-transcribe", onclick: move |_| ui_status.set(TranscriptionUiStatus::Submitting), "Transcribe Audio" }
                            }
                        },
                        TranscriptionUiStatus::Submitting => rsx! { p { class: "transcribing-message", "Submitting job... Please wait." } },
                        TranscriptionUiStatus::Monitoring => {
                            let status_message = if let Some(Ok(state)) = job_state() {
                                match state.status {
                                    JobStatus::Queued => format!("Job is queued at position {}.", state.queue_position.unwrap_or(0)),
                                    JobStatus::Processing => "Job is being processed...".to_string(),
                                    _ => "Waiting for status update...".to_string(),
                                }
                            } else if let Some(Err(_)) = job_state() { "Error polling job status... Retrying.".to_string() }
                            else { "Checking job status...".to_string() };
                            rsx! { p { class: "transcribing-message", "{status_message}" } }
                        },
                        TranscriptionUiStatus::Completed(result) => rsx! {
                            div { class: "success-message",
                                p { "Transcription successful!" }
                                p { code { "{result}" } }
                            }
                            button { class: "button-new", onclick: reset_state, "Start New Transcription" }
                        },
                        TranscriptionUiStatus::Error(error_message) => rsx! {
                            div { class: "error-message", p { "{error_message}" } }
                            button { class: "button-new", onclick: reset_state, "Start New Transcription" }
                        }
                    }
                }
            }
        }
    }
}
