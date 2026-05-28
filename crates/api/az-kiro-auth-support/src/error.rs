use crate::unsupported::BlockedCapability;
use az_derive_aliases::{apply, error};

/// Kiro 认证支持操作的 crate 内统一结果类型。
pub type KiroAuthSupportResult<T> = Result<T, KiroAuthSupportError>;

/// Kiro 设备流程、解析和生成辅助函数返回的错误。
#[apply(error)]
pub enum KiroAuthSupportError {
    /// 配置在任何网络 IO 前未通过校验。
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// 配置的基础 URL 无法解析。
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// 相对 API 路径无法拼接到配置的基础 URL。
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// 网络请求或响应体读取失败。
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// JSON 载荷序列化或反序列化失败。
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// 请求完成但返回了非成功 HTTP 状态码。
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// 最终请求 URL。
        url: String,
        /// HTTP 状态码。
        status: u16,
        /// 以 UTF-8 lossy 方式解码的响应体。
        body: String,
    },
    /// 远端服务返回成功状态，但数据缺失或格式不正确。
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// 安全随机数生成失败。
    #[error("crypto random generation failed")]
    Crypto,
    /// 请求的能力属于刻意不支持的自动化流程。
    #[error("unsupported capability: {capability}")]
    UnsupportedCapability { capability: BlockedCapability },
}
