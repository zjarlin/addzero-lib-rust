use az_derive_aliases::{apply, error};

/// Crate-local result type for Kiro auth support operations.
pub type KiroAuthSupportResult<T> = Result<T, KiroAuthSupportError>;

/// Errors returned by Kiro device-flow, parsing, and generation helpers.
#[apply(error)]
pub enum KiroAuthSupportError {
    /// Configuration failed validation before any network I/O was attempted.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// The configured base URL cannot be parsed.
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// A relative API path could not be joined with the configured base URL.
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// A network request or response-body read failed.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// A JSON payload failed to serialize or deserialize.
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// A request completed with a non-success HTTP status.
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// Final request URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// Response body, decoded lossily as UTF-8.
        body: String,
    },
    /// The remote service returned a success status with missing or malformed data.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// Secure random generation failed.
    #[error("crypto random generation failed")]
    Crypto,
    /// The requested capability belongs to an intentionally unsupported automation flow.
    #[error("unsupported capability: {0}")]
    UnsupportedCapability(&'static str),
}
