use crate::unsupported::BlockedCapability;
use az_derive_aliases::{apply, error};
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};

/// DuckMail、PKCE 和认证文件支持操作的 crate 内统一结果类型。
pub type CodexAuthSupportResult<T> = Result<T, CodexAuthSupportError>;

/// 安全认证支持辅助函数返回的错误。
#[apply(error)]
pub enum CodexAuthSupportError {
    /// 配置在任何网络 IO 前未通过校验。
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// 配置的基础 URL 无法解析。
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// 相对 API 路径无法拼接到配置的基础 URL。
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// 配置的 HTTP header 名称不合法。
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        name: String,
        #[source]
        source: InvalidHeaderName,
    },
    /// 配置的 HTTP header 值不合法。
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        name: String,
        #[source]
        source: InvalidHeaderValue,
    },
    /// 网络请求或响应体读取失败。
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// JSON 载荷序列化或反序列化失败。
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// 文件系统操作失败。
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    /// 请求完成但返回了非成功 HTTP 状态码。
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        url: String,
        status: u16,
        body: String,
    },
    /// 远端服务返回成功状态，但数据缺失或格式不正确。
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// token 无法解析；本 crate 只解码元数据，不校验 JWT。
    #[error("invalid token: {0}")]
    InvalidToken(String),
    /// 安全随机数生成失败。
    #[error("crypto random generation failed")]
    Crypto,
    /// 请求的能力属于刻意不支持的自动化流程。
    #[error("unsupported capability: {capability}")]
    UnsupportedCapability { capability: BlockedCapability },
}
