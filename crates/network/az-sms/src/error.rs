use crate::SmsOrderStatus;
use az_derive_aliases::{apply, plain_copy_eq};
use thiserror::Error;

/// Result alias for SMS provider operations.
pub type SmsResult<T> = Result<T, SmsError>;

/// Errors that can occur while calling an SMS provider.
#[derive(Debug, Error)]
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
#[apply(plain_copy_eq)]
pub struct ProviderStatus(pub Option<u16>);

impl std::fmt::Display for ProviderStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(status) => write!(formatter, " HTTP {status}"),
            None => Ok(()),
        }
    }
}

impl From<Option<u16>> for ProviderStatus {
    fn from(value: Option<u16>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_status_formats_only_when_present() {
        let err = SmsError::ProviderError {
            status: ProviderStatus(Some(400)),
            message: "bad country".to_owned(),
        };
        assert_eq!(err.to_string(), "provider error HTTP 400: bad country");

        let err = SmsError::ProviderError {
            status: ProviderStatus(None),
            message: "no free phones".to_owned(),
        };
        assert_eq!(err.to_string(), "provider error: no free phones");
    }
}
