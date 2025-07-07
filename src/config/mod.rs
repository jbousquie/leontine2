//! Configuration constants for the Leontine application

/// Default URL for the WhisperX API endpoint
pub const DEFAULT_API_URL: &str = "https://llm.iut-rodez.fr/leontine/api";

/// Path to the API status endpoint
pub const API_STATUS_ENDPOINT: &str = "/status";

/// Initial API status check delay in milliseconds (check once when the component mounts)
pub const API_STATUS_INITIAL_CHECK_MS: u64 = 100;

/// Interval between API status checks in milliseconds (check every 30 seconds)
pub const API_STATUS_CHECK_INTERVAL_MS: u64 = 30000;
