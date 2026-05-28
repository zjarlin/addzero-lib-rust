use az_derive_aliases::{apply, error};

/// Gmail 验证码操作的统一结果类型。
pub type GmailCodeResult<T> = Result<T, GmailCodeError>;

/// Gmail API 调用和验证码解析辅助函数返回的错误。
#[apply(error)]
pub enum GmailCodeError {
    /// 客户端配置在发送请求前未通过校验。
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// 配置的 Gmail API 基础 URL 无法解析。
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    /// Gmail API 路径无法拼接到基础 URL。
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    /// 网络传输失败或响应体读取失败。
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// JSON 序列化或反序列化失败。
    #[error("failed to process json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// Gmail 返回了非成功 HTTP 状态码。
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        /// 最终请求 URL。
        url: String,
        /// HTTP 状态码。
        status: u16,
        /// 以 UTF-8 lossy 方式解码的响应体。
        body: String,
    },
    /// Gmail 邮件正文 part 包含非法 base64url 内容。
    #[error("failed to decode Gmail message body for part `{part_id}`: {source}")]
    BodyDecode {
        /// Gmail 邮件 part ID，或为根正文合成的 ID。
        part_id: String,
        /// Base64 解码器源错误。
        #[source]
        source: base64::DecodeError,
    },
    /// 解码后的正文 part 不是合法 UTF-8。
    #[error("Gmail message body for part `{part_id}` is not valid UTF-8: {source}")]
    BodyUtf8 {
        /// Gmail 邮件 part ID，或为根正文合成的 ID。
        part_id: String,
        /// UTF-8 解码器源错误。
        #[source]
        source: std::string::FromUtf8Error,
    },
}
