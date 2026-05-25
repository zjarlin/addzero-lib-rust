use crate::model::SmsOrderStatus;
use az_derive_aliases::{apply, error, from_copy_eq_display};

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
#[apply(from_copy_eq_display)]
#[display(
    "{}",
    status.map_or(String::new(), |status| format!(" HTTP {status}"))
)]
pub struct ProviderStatus {
    pub(crate) status: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::ProviderStatus;

    #[test]
    fn provider_status_display_keeps_optional_prefix() {
        assert_eq!(ProviderStatus { status: None }.to_string(), "");
        assert_eq!(
            ProviderStatus {
                status: Some(503)
            }
            .to_string(),
            " HTTP 503"
        );
    }
}
