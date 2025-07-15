//! Global application state management.
//! This module defines the central `AppState` struct that holds all shared signals
//! and is provided to the entire application via Dioxus context.

use crate::api::{ApiError, ApiStatus, JobState, TranscriptionJob};
use crate::hooks::persistent::UsePersistent;
use chrono::{self, prelude::*};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;

/// Represents the possible UI states for the transcription panel.
/// This is kept in the global state so that other components could potentially
/// react to it, and to centralize all state logic.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum TranscriptionUiStatus {
    #[default]
    Idle,
    FileSelected,
    Submitting,
    Monitoring,
    Completed(String),
    Error(String),
}

/// Represents the connection status of the WhisperX API endpoint.
/// This provides a clearer state machine than `Option<Result<...>>`.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum ApiConnectionStatus {
    /// The application has not yet attempted to connect, or a check is in progress.
    #[default]
    Pending,
    /// A successful connection has been made, and the API status is available.
    Available(ApiStatus, DateTime<Utc>),
    /// An attempt to connect to the API failed.
    Unavailable(ApiError, DateTime<Utc>),
}

/// The global application state.
///
/// This struct holds all the shared signals that are needed by various components
/// throughout the application. It is created once in the `App` component and
/// provided down to all children via context.
///
/// It derives `Clone` and `Copy` because signals themselves are just cheap copyable
/// references to the actual data, making it efficient to pass this entire struct around.
#[derive(Clone, Copy)]
pub struct AppState {
    // --- Persisted State ---
    /// The API URL, persisted in local storage.
    pub api_url: UsePersistent<String>,
    /// The currently active transcription job, persisted in local storage.
    pub active_job: UsePersistent<Option<TranscriptionJob>>,

    // --- Volatile State ---
    /// The last known connection status of the API server.
    pub api_connection_status: Signal<ApiConnectionStatus>,
    /// The last known state of the active transcription job.
    pub job_state: Signal<Option<Result<JobState, ApiError>>>,
    /// The current status of the transcription panel's UI.
    pub transcription_ui_status: Signal<TranscriptionUiStatus>,
}
