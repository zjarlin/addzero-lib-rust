use thiserror::Error;

/// Result alias for Gmail verification-code operations.
pub type GmailCodeResult<T> = Result<T, GmailCodeError>;

/// Errors returned by Gmail API and verification-code parsing helpers.
#[derive(Debug, Error)]
pub enum GmailCodeError {
    /// Client configuration failed validation before a request was sent.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// The configured Gmail API base URL cannot be parsed.
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// A Gmail API path could not be joined against the base URL.
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// Network transport failed or the response body could not be read.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// JSON serialization or deserialization failed.
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// Gmail returned a non-success HTTP status.
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// Final request URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// Response body decoded lossily as UTF-8.
        body: String,
    },
    /// A Gmail message body part had invalid base64url content.
    #[error("failed to decode Gmail message body for part `{part_id}`: {source}")]
    BodyDecode {
        /// Gmail message part id, or a synthesized id for the root body.
        part_id: String,
        /// Base64 decoder source error.
        #[source]
        source: base64::DecodeError,
    },
    /// A decoded body part was not valid UTF-8.
    #[error("Gmail message body for part `{part_id}` is not valid UTF-8: {source}")]
    BodyUtf8 {
        /// Gmail message part id, or a synthesized id for the root body.
        part_id: String,
        /// UTF-8 decoder source error.
        #[source]
        source: std::string::FromUtf8Error,
    },
}
