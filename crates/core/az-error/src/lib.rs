//! addzero 生态系统的统一错误类型。
//!
//! 本 crate 提供一个统一的 [`AppError`] 枚举，涵盖后端服务中常见的错误场景，
//! 包括 HTTP 风格错误、I/O 失败以及 JSON（反）序列化问题。
//! 使用 [`AppResult<T>`] 作为可失败操作的标准返回类型。

use az_derive_aliases::{apply, error};

/// 统一应用错误类型。
///
/// 每个变体都携带面向人的错误消息，并可通过 [`AppError::status_code`] 映射到 HTTP
/// 状态码，通过 [`AppError::error_type`] 映射到机器可读的错误分类。
#[apply(error)]
pub enum AppError {
    /// 请求的资源不存在（HTTP 404）。
    #[error("not found: {0}")]
    NotFound(String),

    /// 输入没有通过校验规则（HTTP 422）。
    #[error("validation error: {0}")]
    Validation(String),

    /// 当前请求需要身份认证（HTTP 401）。
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// 已认证用户没有足够权限（HTTP 403）。
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// 检测到资源冲突（HTTP 409）。
    #[error("conflict: {0}")]
    Conflict(String),

    /// 发生非预期的服务端内部错误（HTTP 500）。
    #[error("internal error: {0}")]
    Internal(String),

    /// 请求格式错误或参数无效（HTTP 400）。
    #[error("bad request: {0}")]
    BadRequest(String),

    /// 操作超时（HTTP 504）。
    #[error("timeout: {0}")]
    Timeout(String),

    /// 发生 I/O 错误（HTTP 500）。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// 发生 JSON 序列化或反序列化错误（HTTP 500）。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppError {
    /// 返回该错误变体对应的 HTTP 状态码。
    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::Validation(_) => 422,
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::Conflict(_) => 409,
            Self::Internal(_) => 500,
            Self::BadRequest(_) => 400,
            Self::Timeout(_) => 504,
            Self::Io(_) => 500,
            Self::Json(_) => 500,
        }
    }

    /// 返回短小且机器可读的错误类型标识。
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::Validation(_) => "validation",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Conflict(_) => "conflict",
            Self::Internal(_) => "internal",
            Self::BadRequest(_) => "bad_request",
            Self::Timeout(_) => "timeout",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
        }
    }
}

/// 使用 [`AppError`] 作为错误类型的便捷返回别名。
pub type AppResult<T> = Result<T, AppError>;
