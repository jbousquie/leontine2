//! API client module for interacting with the WhisperX API

use crate::config::API_STATUS_ENDPOINT;
use gloo_net::http::Request;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

/// Error type for API operations
#[derive(Debug)]
pub enum ApiError {
    /// Network or request error
    RequestFailed(String),
    /// Server returned a non-200 response
    HttpError(u16, String),
    /// Error parsing the response JSON
    ParseError(String),
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

/// Fetches the API status from the server
///
/// # Arguments
///
/// * `api_url` - The base URL of the API
///
/// # Returns
///
/// The API status response or an error
pub async fn get_status(api_url: &str) -> Result<ApiStatus, ApiError> {
    if api_url.is_empty() {
        warn!("API URL is empty, cannot check status");
        return Err(ApiError::RequestFailed(
            "API URL is not configured".to_string(),
        ));
    }

    let url = format!("{}{}", api_url, API_STATUS_ENDPOINT);
    info!("Fetching API status from: {}", url);

    let response = match Request::get(&url).send().await {
        Ok(resp) => {
            info!(
                "API status response received with status: {}",
                resp.status()
            );
            resp
        }
        Err(err) => {
            error!("Failed to send request: {}", err);
            return Err(ApiError::RequestFailed(err.to_string()));
        }
    };

    if !response.ok() {
        let status = response.status();
        let text = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to get error text: {}", e);
                "Unknown error".to_string()
            }
        };
        error!("API returned error status {}: {}", status, text);
        return Err(ApiError::HttpError(status, text));
    }

    match response.json::<ApiStatus>().await {
        Ok(status) => {
            info!(
                "API status parsed successfully: {:?} jobs queued, {:?} jobs processing",
                status.queue_state.queued_jobs, status.queue_state.processing_jobs
            );
            Ok(status)
        }
        Err(err) => {
            error!("Failed to parse API status response: {}", err);
            Err(ApiError::ParseError(err.to_string()))
        }
    }
}
