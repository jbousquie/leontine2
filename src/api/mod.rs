//! API client module for interacting with the WhisperX API

use crate::config::{API_STATUS_ENDPOINT, API_TRANSCRIPTION_ENDPOINT};
use crate::dioxus_elements::FileEngine;
use gloo_net::http::Request;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use web_sys::js_sys::{Array, Uint8Array};
use web_sys::wasm_bindgen::JsValue;
use web_sys::{Blob, FormData};

/// Error type for API operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    /// Network or request error from `gloo_net` or JS interop.
    RequestFailed(String),
    /// Server returned a non-200 response.
    HttpError(u16, String),
    /// Error parsing the response JSON.
    ParseError(String),
    /// The file from the file engine was not available or couldn't be read.
    FileNotAvailable,
}

// --- From Trait Implementations ---

/// Allows for the use of `?` on `gloo_net::Error` to convert it into our `ApiError`.
impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        ApiError::RequestFailed(err.to_string())
    }
}

/// Allows for the use of `?` on `serde_json::Error` to convert it into our `ApiError`.
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::ParseError(err.to_string())
    }
}

/// Allows for the use of `?` on `JsValue` errors (e.g., from `FormData` creation)
/// to convert them into our `ApiError`.
impl From<JsValue> for ApiError {
    fn from(err: JsValue) -> Self {
        ApiError::RequestFailed(format!("{:?}", err))
    }
}

/// Represents the JSON response from a successful asynchronous transcription submission.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TranscriptionJob {
    pub job_id: String,
    pub job_status: String,
    pub queued_at: String,
    pub message: String,
    pub url: String,
}

/// Represents the parameters for a transcription job, to be serialized as JSON.
#[derive(Serialize)]
struct TranscriptionParams {
    sync: bool,
}

/// API status response structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiStatus {
    pub server: ServerConfig,
    pub processing: ProcessingConfig,
    pub resources: ResourcesConfig,
    pub security: SecurityConfig,
    pub queue_state: QueueState,
    pub error: Option<String>,
}

/// Server configuration section of the API status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
    pub timeout: u32,
    pub keepalive: u32,
    pub worker_number: u32,
}

/// Processing configuration section of the API status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessingConfig {
    pub concurrent_mode: bool,
    pub max_concurrent_jobs: u32,
    pub device: String,
    pub device_index: String,
    pub default_output_format: String,
    pub default_sync_mode: bool,
    pub sync_timeout: u32,
}

/// Resources configuration section of the API status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcesConfig {
    pub max_file_size: u64,
    pub job_retention_hours: u32,
    pub cleanup_interval_hours: u32,
}

/// Security configuration section of the API status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub authorization_enabled: bool,
}

/// Queue state section of the API status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueState {
    pub queued_jobs: u32,
    pub processing_jobs: u32,
}

/// Fetches the API status from the server.
pub async fn get_status(api_url: &str) -> Result<ApiStatus, ApiError> {
    if api_url.is_empty() {
        warn!("API URL is empty, cannot check status");
        return Err(ApiError::RequestFailed(
            "API URL is not configured".to_string(),
        ));
    }

    let url = format!("{}{}", api_url, API_STATUS_ENDPOINT);
    info!("Fetching API status from: {}", url);

    let response = Request::get(&url).send().await?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        error!("API returned error status {}: {}", status, text);
        return Err(ApiError::HttpError(status, text));
    }

    let status: ApiStatus = response.json().await?;

    info!(
        "API status parsed successfully: {:?} jobs queued, {:?} jobs processing",
        status.queue_state.queued_jobs, status.queue_state.processing_jobs
    );
    Ok(status)
}

/// Submits an audio file for asynchronous transcription.
pub async fn submit_transcription(
    api_url: &str,
    file_engine: &Arc<dyn FileEngine>,
) -> Result<TranscriptionJob, ApiError> {
    if api_url.is_empty() {
        return Err(ApiError::RequestFailed(
            "API URL is not configured".to_string(),
        ));
    }

    let file_name = file_engine
        .files()
        .first()
        .cloned()
        .ok_or(ApiError::FileNotAvailable)?;

    let file_bytes = file_engine
        .read_file(&file_name)
        .await
        .ok_or(ApiError::FileNotAvailable)?;

    // Create a JS-compatible byte array (Uint8Array) from the Rust byte slice.
    let uint8_array = Uint8Array::from(file_bytes.as_slice());

    // The web_sys::Blob constructor needs a sequence (JS Array) of blob parts.
    let array = Array::new();
    array.push(&uint8_array.into());
    let blob = Blob::new_with_blob_sequence(&array)?;

    // --- Correctly structure the form data ---
    let params = TranscriptionParams { sync: false };
    let params_json = serde_json::to_string(&params)?;

    let form_data = FormData::new()?;
    form_data.append_with_blob_and_filename("file", &blob, &file_name)?;
    form_data.append_with_str("params", &params_json)?;
    // --- End of correction ---

    let url = format!("{}{}", api_url, API_TRANSCRIPTION_ENDPOINT);
    info!("Submitting transcription to: {}", url);

    let response = Request::post(&url).body(form_data)?.send().await?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError::HttpError(status, text));
    }

    let job: TranscriptionJob = response.json().await?;
    info!("Transcription job submitted successfully: {:?}", job);
    Ok(job)
}
