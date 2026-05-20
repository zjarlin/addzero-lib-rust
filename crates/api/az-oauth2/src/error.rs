use thiserror::Error;

/// Result alias for OAuth2 operations.
pub type OAuth2Result<T> = Result<T, OAuth2Error>;

/// Errors returned by OAuth2 helpers.
#[derive(Debug, Error)]
pub enum OAuth2Error {
    /// Client configuration failed validation before network I/O.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// A configured endpoint URL cannot be parsed.
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// A relative path could not be joined against a configured endpoint URL.
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// Network transport failed or a response body could not be read.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// JSON serialization or deserialization failed.
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// Local listener or callback response I/O failed.
    #[error("local callback I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Secure random generation failed.
    #[error("crypto random generation failed")]
    Crypto,
    /// The provider returned a non-success HTTP status without a structured OAuth error body.
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// Final request URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// Response body decoded lossily as UTF-8.
        body: String,
    },
    /// The OAuth provider returned a structured OAuth error response.
    #[error("oauth provider error `{error}`: {description}")]
    ProviderError {
        /// OAuth error code.
        error: String,
        /// Optional provider-supplied description.
        description: String,
    },
    /// The provider returned a syntactically valid response with required fields missing.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// The loopback callback did not contain a usable authorization code.
    #[error("invalid authorization callback: {0}")]
    InvalidCallback(String),
    /// The callback state did not match the state generated for this flow.
    #[error("oauth state mismatch: expected `{expected}`, got `{actual}`")]
    StateMismatch {
        /// Generated state.
        expected: String,
        /// Callback state.
        actual: String,
    },
}
