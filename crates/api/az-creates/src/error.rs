use az_derive_aliases::{apply, error, impl_from_match};
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};

/// `az-creates` 对外统一使用的结果类型。
pub type CreatesResult<T> = Result<T, CreatesError>;

/// 第三方 API 门面层的统一错误类型。
///
/// 该类型把 Maven、音乐、天眼查等子客户端的公共失败形状收敛到同一边界，
/// 同时通过 `#[source]` / `#[from]` 保留底层 `reqwest`、`serde_json` 和 header 解析错误链。
#[apply(error)]
pub enum CreatesError {
    /// 调用方传入的配置不完整或语义非法。
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// 基础地址无法解析为 URL。
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// 请求路径无法安全拼到基础地址后。
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// 默认或签名请求头名称不是合法 HTTP header name。
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        /// 原始 header 名称。
        name: String,
        /// reqwest/header 层返回的具体错误。
        #[source]
        source: InvalidHeaderName,
    },
    /// 默认或签名请求头值不是合法 HTTP header value。
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        /// 对应的 header 名称。
        name: String,
        /// reqwest/header 层返回的具体错误。
        #[source]
        source: InvalidHeaderValue,
    },
    /// 网络传输、超时、TLS 或请求构造失败。
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// 响应体不是当前 API 契约期望的 JSON 形状。
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// 上游返回非 2xx 状态码，保留 URL、状态码和响应体便于排查。
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// 实际请求 URL。
        url: String,
        /// HTTP 状态码。
        status: u16,
        /// 上游响应体文本。
        body: String,
    },
    /// 华为云等签名流程生成认证头失败。
    #[error("signature error: {0}")]
    Signature(String),
    /// 上游返回成功状态以外的业务错误，或成功响应缺少必要数据。
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

impl_from_match!(az_music::MusicError => CreatesError {
    az_music::MusicError::InvalidConfig(message) => Self::InvalidConfig(message),
    az_music::MusicError::InvalidBaseUrl(url) => Self::InvalidBaseUrl(url),
    az_music::MusicError::InvalidPath(path) => Self::InvalidPath(path),
    az_music::MusicError::InvalidHeaderName { name, source } => Self::InvalidHeaderName { name, source },
    az_music::MusicError::InvalidHeaderValue { name, source } => Self::InvalidHeaderValue { name, source },
    az_music::MusicError::Transport(error) => Self::Transport(error),
    az_music::MusicError::Json(error) => Self::Json(error),
    az_music::MusicError::HttpStatus { url, status, body } => Self::HttpStatus { url, status, body },
    az_music::MusicError::InvalidResponse(message) => Self::InvalidResponse(message),
});

impl_from_match!(az_maven::MavenError => CreatesError {
    az_maven::MavenError::InvalidConfig(message) => Self::InvalidConfig(message),
    az_maven::MavenError::InvalidBaseUrl(url) => Self::InvalidBaseUrl(url),
    az_maven::MavenError::InvalidPath(path) => Self::InvalidPath(path),
    az_maven::MavenError::InvalidHeaderName { name, source } => Self::InvalidHeaderName { name, source },
    az_maven::MavenError::InvalidHeaderValue { name, source } => Self::InvalidHeaderValue { name, source },
    az_maven::MavenError::Transport(error) => Self::Transport(error),
    az_maven::MavenError::Json(error) => Self::Json(error),
    az_maven::MavenError::HttpStatus { url, status, body } => Self::HttpStatus { url, status, body },
    az_maven::MavenError::Signature(message) => Self::Signature(message),
    az_maven::MavenError::InvalidResponse(message) => Self::InvalidResponse(message),
});
