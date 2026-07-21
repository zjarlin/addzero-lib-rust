//! addzero 生态系统的错误处理辅助入口。
//!
//! 本 crate 不再定义统一 `AppError` enum。内部可失败操作默认使用
//! [`anyhow::Result`]，边界层按 HTTP、CLI 或插件协议各自映射响应格式。

/// 按错误文本推断 HTTP 状态码。
///
/// 该函数只服务最终边界映射；业务内部不要依赖字符串分类做控制流。
#[must_use]
pub fn status_code_for_error(error: &(dyn std::error::Error + 'static)) -> u16 {
    status_code_for_message(&error.to_string())
}

/// 按错误文本推断短错误类型。
///
/// 该函数只服务最终边界映射；业务内部不要依赖字符串分类做控制流。
#[must_use]
pub fn error_type_for_error(error: &(dyn std::error::Error + 'static)) -> &'static str {
    error_type_for_message(&error.to_string())
}

/// 按消息前缀推断 HTTP 状态码。
#[must_use]
pub fn status_code_for_message(message: &str) -> u16 {
    match error_type_for_message(message) {
        "not_found" => 404,
        "validation" => 422,
        "unauthorized" => 401,
        "forbidden" => 403,
        "conflict" => 409,
        "bad_request" => 400,
        "timeout" => 504,
        _ => 500,
    }
}

/// 按消息前缀推断短错误类型。
#[must_use]
pub fn error_type_for_message(message: &str) -> &'static str {
    let normalized = message.trim().to_ascii_lowercase();
    if normalized.starts_with("not found") || normalized.contains(" was not found") {
        "not_found"
    } else if normalized.starts_with("validation") {
        "validation"
    } else if normalized.starts_with("unauthorized")
        || normalized.contains("未登录")
        || normalized.contains("登录已失效")
    {
        "unauthorized"
    } else if normalized.starts_with("forbidden") {
        "forbidden"
    } else if normalized.starts_with("conflict") || normalized.contains("duplicate") {
        "conflict"
    } else if normalized.starts_with("bad request") || normalized.starts_with("invalid ") {
        "bad_request"
    } else if normalized.starts_with("timeout") || normalized.contains("timed out") {
        "timeout"
    } else {
        "internal"
    }
}
