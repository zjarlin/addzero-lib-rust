//! Unified error types for the addzero ecosystem.
//!
//! This crate provides a single [`AppError`] enum that covers common error
//! scenarios encountered across backend services, including HTTP-style errors,
//! I/O failures, and JSON (de)serialization issues. Use [`AppResult<T>`] as
//! the standard return type for fallible operations.

/// Unified application error type.
///
/// Every variant carries a human-readable message and maps to an HTTP status
/// code via [`AppError::status_code`] as well as a machine-readable error
/// kind via [`AppError::error_type`].
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The requested resource was not found (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),

    /// Input failed validation rules (HTTP 422).
    #[error("validation error: {0}")]
    Validation(String),

    /// Authentication is required (HTTP 401).
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// The authenticated user lacks permission (HTTP 403).
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// A resource conflict was detected (HTTP 409).
    #[error("conflict: {0}")]
    Conflict(String),

    /// An unexpected internal server error occurred (HTTP 500).
    #[error("internal error: {0}")]
    Internal(String),

    /// The request was malformed or invalid (HTTP 400).
    #[error("bad request: {0}")]
    BadRequest(String),

    /// The operation timed out (HTTP 504).
    #[error("timeout: {0}")]
    Timeout(String),

    /// An I/O error occurred (HTTP 500).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialization / deserialization error occurred (HTTP 500).
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppError {
    /// Returns the corresponding HTTP status code for this error variant.
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

    /// Returns a short, machine-readable error type identifier.
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

/// Convenient result alias using [`AppError`] as the error type.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Display messages ──────────────────────────────────────────────

    #[test]
    fn test_display_not_found() {
        let err = AppError::NotFound("user 42".into());
        assert_eq!(err.to_string(), "not found: user 42");
    }

    #[test]
    fn test_display_validation() {
        let err = AppError::Validation("email is invalid".into());
        assert_eq!(err.to_string(), "validation error: email is invalid");
    }

    #[test]
    fn test_display_unauthorized() {
        let err = AppError::Unauthorized("missing token".into());
        assert_eq!(err.to_string(), "unauthorized: missing token");
    }

    #[test]
    fn test_display_forbidden() {
        let err = AppError::Forbidden("admin only".into());
        assert_eq!(err.to_string(), "forbidden: admin only");
    }

    #[test]
    fn test_display_conflict() {
        let err = AppError::Conflict("duplicate entry".into());
        assert_eq!(err.to_string(), "conflict: duplicate entry");
    }

    #[test]
    fn test_display_internal() {
        let err = AppError::Internal("something broke".into());
        assert_eq!(err.to_string(), "internal error: something broke");
    }

    #[test]
    fn test_display_bad_request() {
        let err = AppError::BadRequest("missing field".into());
        assert_eq!(err.to_string(), "bad request: missing field");
    }

    #[test]
    fn test_display_timeout() {
        let err = AppError::Timeout("upstream slow".into());
        assert_eq!(err.to_string(), "timeout: upstream slow");
    }

    #[test]
    fn test_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        let err: AppError = io_err.into();
        assert!(err.to_string().contains("io error:"));
    }

    #[test]
    fn test_display_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: AppError = json_err.into();
        assert!(err.to_string().contains("json error:"));
    }

    // ── status_code ───────────────────────────────────────────────────

    #[test]
    fn test_status_codes() {
        assert_eq!(AppError::NotFound("".into()).status_code(), 404);
        assert_eq!(AppError::Validation("".into()).status_code(), 422);
        assert_eq!(AppError::Unauthorized("".into()).status_code(), 401);
        assert_eq!(AppError::Forbidden("".into()).status_code(), 403);
        assert_eq!(AppError::Conflict("".into()).status_code(), 409);
        assert_eq!(AppError::Internal("".into()).status_code(), 500);
        assert_eq!(AppError::BadRequest("".into()).status_code(), 400);
        assert_eq!(AppError::Timeout("".into()).status_code(), 504);

        let io_err: AppError = std::io::Error::new(std::io::ErrorKind::Other, "x").into();
        assert_eq!(io_err.status_code(), 500);

        let json_err: AppError = serde_json::from_str::<serde_json::Value>("x")
            .unwrap_err()
            .into();
        assert_eq!(json_err.status_code(), 500);
    }

    // ── error_type ────────────────────────────────────────────────────

    #[test]
    fn test_error_types() {
        assert_eq!(AppError::NotFound("".into()).error_type(), "not_found");
        assert_eq!(AppError::Validation("".into()).error_type(), "validation");
        assert_eq!(
            AppError::Unauthorized("".into()).error_type(),
            "unauthorized"
        );
        assert_eq!(AppError::Forbidden("".into()).error_type(), "forbidden");
        assert_eq!(AppError::Conflict("".into()).error_type(), "conflict");
        assert_eq!(AppError::Internal("".into()).error_type(), "internal");
        assert_eq!(AppError::BadRequest("".into()).error_type(), "bad_request");
        assert_eq!(AppError::Timeout("".into()).error_type(), "timeout");

        let io_err: AppError = std::io::Error::new(std::io::ErrorKind::Other, "x").into();
        assert_eq!(io_err.error_type(), "io");

        let json_err: AppError = serde_json::from_str::<serde_json::Value>("x")
            .unwrap_err()
            .into();
        assert_eq!(json_err.error_type(), "json");
    }

    // ── From conversions ──────────────────────────────────────────────

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<i32>("bad").unwrap_err();
        let app_err: AppError = json_err.into();
        assert!(matches!(app_err, AppError::Json(_)));
    }

    // ── AppResult alias ───────────────────────────────────────────────

    #[test]
    fn test_app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_app_result_err() {
        let result: AppResult<i32> = Err(AppError::Internal("boom".into()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "internal error: boom");
    }
}
