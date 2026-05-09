use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use thiserror::Error;

/// Result alias for Cloudflare Temp Email operations.
pub type TempMailResult<T> = Result<T, TempMailError>;

/// Errors returned by the Cloudflare Temp Email client.
#[derive(Debug, Error)]
pub enum TempMailError {
    /// Client configuration is internally inconsistent.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// The configured worker base URL is not a valid URL.
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// A request path could not be joined against the base URL.
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// A configured header name is invalid.
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        /// Header name from caller input.
        name: String,
        /// Parser error from `reqwest`.
        #[source]
        source: InvalidHeaderName,
    },
    /// A configured header value is invalid.
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        /// Header name from caller input.
        name: String,
        /// Parser error from `reqwest`.
        #[source]
        source: InvalidHeaderValue,
    },
    /// The HTTP transport failed before a usable response was produced.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// A JSON payload failed to decode.
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// The worker returned a non-success HTTP status.
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// Final request URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// Response body, decoded lossily as UTF-8.
        body: String,
    },
    /// The worker returned syntactically valid JSON with missing required data.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}
