use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use thiserror::Error;

/// Crate-local result type for DuckMail, PKCE, and auth-file support operations.
pub type CodexAuthSupportResult<T> = Result<T, CodexAuthSupportError>;

/// Errors returned by the safe auth support helpers.
#[derive(Debug, Error)]
pub enum CodexAuthSupportError {
    /// Configuration failed validation before any network I/O was attempted.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// The configured base URL cannot be parsed.
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// A relative API path could not be joined with the configured base URL.
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// A configured HTTP header name is invalid.
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        name: String,
        #[source]
        source: InvalidHeaderName,
    },
    /// A configured HTTP header value is invalid.
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        name: String,
        #[source]
        source: InvalidHeaderValue,
    },
    /// A network request or response-body read failed.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// A JSON payload failed to serialize or deserialize.
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// A filesystem operation failed.
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    /// A request completed with a non-success HTTP status.
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        url: String,
        status: u16,
        body: String,
    },
    /// The remote service returned a success status with missing or malformed data.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// A token could not be parsed. This crate only decodes metadata; it does not verify JWTs.
    #[error("invalid token: {0}")]
    InvalidToken(String),
    /// Secure random generation failed.
    #[error("crypto random generation failed")]
    Crypto,
    /// The requested capability belongs to the intentionally unsupported automation flow.
    #[error("unsupported capability: {0}")]
    UnsupportedCapability(&'static str),
}
