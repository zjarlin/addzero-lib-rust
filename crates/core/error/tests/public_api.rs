use az_error::api::{
    error_type_for_error, error_type_for_message, status_code_for_error, status_code_for_message,
};

#[test]
fn status_code_maps_common_boundary_messages() {
    assert_eq!(status_code_for_message("not found: user 42"), 404);
    assert_eq!(status_code_for_message("validation error: bad email"), 422);
    assert_eq!(status_code_for_message("unauthorized: missing token"), 401);
    assert_eq!(status_code_for_message("forbidden: admin only"), 403);
    assert_eq!(status_code_for_message("conflict: duplicate entry"), 409);
    assert_eq!(status_code_for_message("bad request: missing field"), 400);
    assert_eq!(status_code_for_message("timeout: upstream slow"), 504);
    assert_eq!(status_code_for_message("io error: pipe broke"), 500);
}

#[test]
fn error_type_maps_common_boundary_messages() {
    assert_eq!(error_type_for_message("not found: user 42"), "not_found");
    assert_eq!(
        error_type_for_message("validation error: bad email"),
        "validation"
    );
    assert_eq!(
        error_type_for_message("unauthorized: missing token"),
        "unauthorized"
    );
    assert_eq!(error_type_for_message("forbidden: admin only"), "forbidden");
    assert_eq!(
        error_type_for_message("conflict: duplicate entry"),
        "conflict"
    );
    assert_eq!(
        error_type_for_message("bad request: missing field"),
        "bad_request"
    );
    assert_eq!(error_type_for_message("timeout: upstream slow"), "timeout");
    assert_eq!(error_type_for_message("io error: pipe broke"), "internal");
}

#[test]
fn error_helpers_accept_any_error_object() {
    let error = anyhow::anyhow!("not found: config key");

    assert_eq!(status_code_for_error(error.as_ref()), 404);
    assert_eq!(error_type_for_error(error.as_ref()), "not_found");
}
