/// Axum router for the lowcode service.

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::state::LowcodeState;

// ---------------------------------------------------------------------------
// Layout CRUD handlers (skeleton — handlers are todo!())
// ---------------------------------------------------------------------------

async fn create_layout(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "create_layout")
}

async fn list_layouts(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "list_layouts")
}

async fn get_layout(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "get_layout")
}

async fn update_layout(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "update_layout")
}

async fn delete_layout(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "delete_layout")
}

// ---------------------------------------------------------------------------
// Canvas / node operations (skeleton — #78)
// ---------------------------------------------------------------------------

async fn place_component(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "place_component")
}

async fn update_props(
    _state: State<LowcodeState>,
    _path: Path<(Uuid, String)>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "update_props")
}

async fn delete_component(
    _state: State<LowcodeState>,
    _path: Path<(Uuid, String)>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "delete_component")
}

// ---------------------------------------------------------------------------
// Preview & render (skeleton — #81)
// ---------------------------------------------------------------------------

async fn preview_layout(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "preview_layout")
}

async fn render_layout(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "render_layout")
}

// ---------------------------------------------------------------------------
// Event handling (skeleton — #79)
// ---------------------------------------------------------------------------

async fn handle_event(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "handle_event")
}

// ---------------------------------------------------------------------------
// Scripting (skeleton — #80)
// ---------------------------------------------------------------------------

async fn validate_script(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "validate_script")
}

// ---------------------------------------------------------------------------
// Template CRUD (skeleton — #81)
// ---------------------------------------------------------------------------

async fn create_template(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "create_template")
}

async fn list_templates(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "list_templates")
}

async fn get_template(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "get_template")
}

async fn update_template(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "update_template")
}

async fn delete_template(_state: State<LowcodeState>, _id: Path<Uuid>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "delete_template")
}

// ---------------------------------------------------------------------------
// Component registry (skeleton — #77)
// ---------------------------------------------------------------------------

async fn list_components(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "list_components")
}

async fn register_component(_state: State<LowcodeState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "register_component")
}

// ---------------------------------------------------------------------------
// Router assembly
// ---------------------------------------------------------------------------

/// Builds the full lowcode API router.
pub fn lowcode_router(state: LowcodeState) -> Router {
    Router::new()
        .route("/api/lowcode/layout", post(create_layout).get(list_layouts))
        .route(
            "/api/lowcode/layout/{id}",
            get(get_layout).put(update_layout).delete(delete_layout),
        )
        .route(
            "/api/lowcode/layout/{id}/node",
            post(place_component),
        )
        .route(
            "/api/lowcode/layout/{id}/node/{*path}",
            patch(update_props).delete(delete_component),
        )
        .route("/api/lowcode/layout/{id}/preview", get(preview_layout))
        .route("/api/lowcode/layout/{id}/render", get(render_layout))
        .route("/api/lowcode/event", post(handle_event))
        .route("/api/lowcode/script/validate", post(validate_script))
        .route(
            "/api/lowcode/template",
            post(create_template).get(list_templates),
        )
        .route(
            "/api/lowcode/template/{id}",
            get(get_template).put(update_template).delete(delete_template),
        )
        .route(
            "/api/lowcode/component",
            get(list_components).post(register_component),
        )
        .with_state(state)
}
