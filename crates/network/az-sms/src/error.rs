use crate::model::SmsOrderStatus;
use az_derive_aliases::{apply, error, from_copy_eq_display};

/// SMS provider 操作的统一结果类型。
pub type SmsResult<T> = Result<T, SmsError>;

/// 调用 SMS provider 时可能返回的错误。
#[apply(error)]
pub enum SmsError {
    /// 配置不完整或字段组合不一致。
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// 请求对象在网络 IO 前未通过本地校验。
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// 配置的 provider 基础 URL 不合法。
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),

    /// provider 端点无法构造。
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),

    /// 在解析 provider 响应前 HTTP 传输失败。
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),

    /// JSON 序列化或反序列化失败。
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),

    /// provider 返回了 HTTP 错误或非 JSON 错误正文。
    #[error("provider error{status}: {message}")]
    ProviderError {
        /// 可用时的 HTTP 状态码。
        status: ProviderStatus,
        /// provider 响应正文或归一化后的 provider 消息。
        message: String,
    },

    /// 所选 provider 的 API 不暴露该操作。
    #[error("{provider} does not support {operation}")]
    UnsupportedOperation {
        /// provider 名称。
        provider: &'static str,
        /// 不受支持的操作名称。
        operation: &'static str,
    },

    /// 等待短信超过了指定超时时间。
    #[error("timed out waiting for SMS on order {order_id} after {timeout_secs}s")]
    Timeout {
        /// provider 订单 ID。
        order_id: u64,
        /// 秒级超时时间。
        timeout_secs: u64,
    },

    /// 订单在短信到达前进入终态。
    #[error("order {order_id} closed before SMS arrived: {status:?}")]
    OrderClosed {
        /// provider 订单 ID。
        order_id: u64,
        /// provider 终态状态。
        status: SmsOrderStatus,
    },
}

/// 可选的 provider HTTP 状态码展示包装，避免把格式化逻辑泄漏到错误构造点。
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
            ProviderStatus { status: Some(503) }.to_string(),
            " HTTP 503"
        );
    }
}
