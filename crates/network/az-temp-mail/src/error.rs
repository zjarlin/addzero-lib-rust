use az_derive_aliases::{apply, error};
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};

/// 临时邮箱操作的统一结果类型。
pub type TempMailResult<T> = Result<T, TempMailError>;

/// 临时邮箱客户端返回的错误。
#[apply(error)]
pub enum TempMailError {
    /// 客户端配置字段组合不一致。
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// 配置的 worker 基础 URL 不是合法 URL。
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// 请求路径无法拼接到基础 URL。
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// 配置的 header 名称不合法。
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        /// 调用方输入的 header 名称。
        name: String,
        /// `reqwest` 返回的解析错误。
        #[source]
        source: InvalidHeaderName,
    },
    /// 配置的 header 值不合法。
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        /// 调用方输入的 header 名称。
        name: String,
        /// `reqwest` 返回的解析错误。
        #[source]
        source: InvalidHeaderValue,
    },
    /// 生成可用响应前 HTTP 传输失败。
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// JSON 载荷解码失败。
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// worker 返回了非成功 HTTP 状态码。
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// 最终请求 URL。
        url: String,
        /// HTTP 状态码。
        status: u16,
        /// 以 UTF-8 lossy 方式解码的响应体。
        body: String,
    },
    /// worker 返回了语法合法但缺少必要数据的 JSON。
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}
