use crate::model::SmsOrderStatus;
use az_derive_aliases::{apply, error, from_copy_eq};

/// Result alias for SMS provider operations.
pub type SmsResult<T> = Result<T, SmsError>;

/// Errors that can occur while calling an SMS provider.
#[apply(error)]
pub enum SmsError {
    /// Configuration is incomplete or internally inconsistent.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// A request object failed local validation before network I/O.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// The configured provider base URL is invalid.
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),

    /// The provider endpoint could not be built.
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),

    /// HTTP transport failed before a provider response could be parsed.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),

    /// JSON serialization or deserialization failed.
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),

    /// The provider returned an HTTP error or non-JSON provider error body.
    #[error("provider error{status}: {message}")]
    ProviderError {
        /// HTTP status code when one was available.
        status: ProviderStatus,
        /// Provider response body or normalized provider message.
        message: String,
    },

    /// The selected provider does not expose this operation through its API.
    #[error("{provider} does not support {operation}")]
    UnsupportedOperation {
        /// Provider name.
        provider: &'static str,
        /// Unsupported operation name.
        operation: &'static str,
    },

    /// Waiting for an SMS exceeded the requested timeout.
    #[error("timed out waiting for SMS on order {order_id} after {timeout_secs}s")]
    Timeout {
        /// Provider order ID.
        order_id: u64,
        /// Timeout in seconds.
        timeout_secs: u64,
    },

    /// The order entered a terminal state before an SMS arrived.
    #[error("order {order_id} closed before SMS arrived: {status:?}")]
    OrderClosed {
        /// Provider order ID.
        order_id: u64,
        /// Terminal provider status.
        status: SmsOrderStatus,
    },
}

/// Optional provider HTTP status displayed without leaking formatting logic into
/// error construction sites.
#[apply(from_copy_eq)]
pub struct ProviderStatus(pub Option<u16>);

impl std::fmt::Display for ProviderStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(status) => write!(formatter, " HTTP {status}"),
            None => Ok(()),
        }
    }
}
