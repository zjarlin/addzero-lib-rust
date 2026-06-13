use az_error::{AppError, AppResult};

#[test]
fn display_messages_match_error_variants() {
    assert_eq!(
        AppError::NotFound("user 42".into()).to_string(),
        "not found: user 42"
    );
    assert_eq!(
        AppError::Validation("email is invalid".into()).to_string(),
        "validation error: email is invalid"
    );
    assert_eq!(
        AppError::Unauthorized("missing token".into()).to_string(),
        "unauthorized: missing token"
    );
    assert_eq!(
        AppError::Forbidden("admin only".into()).to_string(),
        "forbidden: admin only"
    );
    assert_eq!(
        AppError::Conflict("duplicate entry".into()).to_string(),
        "conflict: duplicate entry"
    );
    assert_eq!(
        AppError::Internal("something broke".into()).to_string(),
        "internal error: something broke"
    );
    assert_eq!(
        AppError::BadRequest("missing field".into()).to_string(),
        "bad request: missing field"
    );
    assert_eq!(
        AppError::Timeout("upstream slow".into()).to_string(),
        "timeout: upstream slow"
    );
}

#[test]
fn source_error_conversions_keep_display_context() {
    let io_error = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
    let error: AppError = io_error.into();
    assert!(error.to_string().contains("io error:"));

    let json_error = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let error: AppError = json_error.into();
    assert!(error.to_string().contains("json error:"));
}

#[test]
fn status_code_maps_each_error_class() {
    assert_eq!(AppError::NotFound("".into()).status_code(), 404);
    assert_eq!(AppError::Validation("".into()).status_code(), 422);
    assert_eq!(AppError::Unauthorized("".into()).status_code(), 401);
    assert_eq!(AppError::Forbidden("".into()).status_code(), 403);
    assert_eq!(AppError::Conflict("".into()).status_code(), 409);
    assert_eq!(AppError::Internal("".into()).status_code(), 500);
    assert_eq!(AppError::BadRequest("".into()).status_code(), 400);
    assert_eq!(AppError::Timeout("".into()).status_code(), 504);

    let io_error: AppError = std::io::Error::other("x").into();
    assert_eq!(io_error.status_code(), 500);

    let json_error: AppError = serde_json::from_str::<serde_json::Value>("x")
        .unwrap_err()
        .into();
    assert_eq!(json_error.status_code(), 500);
}

#[test]
fn error_type_maps_each_error_class() {
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

    let io_error: AppError = std::io::Error::other("x").into();
    assert_eq!(io_error.error_type(), "io");

    let json_error: AppError = serde_json::from_str::<serde_json::Value>("x")
        .unwrap_err()
        .into();
    assert_eq!(json_error.error_type(), "json");
}

#[test]
fn from_conversions_preserve_error_variants() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let app_error: AppError = io_error.into();
    assert!(matches!(app_error, AppError::Io(_)));

    let json_error = serde_json::from_str::<i32>("bad").unwrap_err();
    let app_error: AppError = json_error.into();
    assert!(matches!(app_error, AppError::Json(_)));
}

#[test]
fn app_result_alias_uses_app_error() {
    let ok: AppResult<i32> = Ok(42);
    assert_eq!(ok.unwrap(), 42);

    let err: AppResult<i32> = Err(AppError::Internal("boom".into()));
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().to_string(), "internal error: boom");
}
