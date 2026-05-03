/// Axum router for the lowcode service.
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::editor::{EditorError, LayoutEditor};
use crate::schema::GridArea;
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
// Canvas / node operations — implemented in #78
// ---------------------------------------------------------------------------

/// POST /api/lowcode/layout/{id}/node — place a new component.
///
/// Body JSON: `{ "parent_path": "root", "component_type": "button", "grid_area": {...}, "props": {...} }`
async fn place_component(
    state: State<LowcodeState>,
    id: Path<Uuid>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let parent_path = body
        .get("parent_path")
        .and_then(|v| v.as_str())
        .unwrap_or("root");
    let component_type = match body.get("component_type").and_then(|v| v.as_str()) {
        Some(ct) => ct,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(
                    serde_json::json!({"error": "missing required field: component_type"}),
                ),
            );
        }
    };
    let grid_area: GridArea = match body.get("grid_area") {
        Some(v) => match serde_json::from_value(v.clone()) {
            Ok(ga) => ga,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({"error": format!("invalid grid_area: {e}")})),
                );
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "missing required field: grid_area"})),
            );
        }
    };
    let props = body
        .get("props")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // TODO: wire to PG repository once layout CRUD is implemented
    let mut layouts = state.layouts.write().await;
    let layout = match layouts.get_mut(&*id) {
        Some(l) => l,
        None => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"error": "layout not found"})),
            );
        }
    };

    match LayoutEditor::place_component(layout, parent_path, component_type, grid_area, props) {
        Ok(node_id) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({"id": node_id})),
        ),
        Err(e) => editor_error_response(e),
    }
}

/// PATCH /api/lowcode/layout/{id}/node/{*path} — update props.
///
/// Body JSON: `{ "props_patch": { ... } }` or just `{ ... }` as props_patch.
async fn update_props(
    state: State<LowcodeState>,
    Path((layout_id, node_path)): Path<(Uuid, String)>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let props_patch = body
        .get("props_patch")
        .cloned()
        .unwrap_or(body);

    // TODO: wire to PG repository once layout CRUD is implemented
    let mut layouts = state.layouts.write().await;
    let layout = match layouts.get_mut(&layout_id) {
        Some(l) => l,
        None => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"error": "layout not found"})),
            );
        }
    };

    match LayoutEditor::update_props(layout, &node_path, props_patch) {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({"status": "ok"})),
        ),
        Err(e) => editor_error_response(e),
    }
}

/// DELETE /api/lowcode/layout/{id}/node/{*path} — remove a component.
async fn delete_component(
    state: State<LowcodeState>,
    Path((layout_id, node_path)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    // TODO: wire to PG repository once layout CRUD is implemented
    let mut layouts = state.layouts.write().await;
    let layout = match layouts.get_mut(&layout_id) {
        Some(l) => l,
        None => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"error": "layout not found"})),
            );
        }
    };

    match LayoutEditor::delete_component(layout, &node_path) {
        Ok(node) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({"status": "deleted", "node": node})),
        ),
        Err(e) => editor_error_response(e),
    }
}

/// PATCH /api/lowcode/layout/{id}/node/{*path}/move — move component position.
///
/// Body JSON: `{ "grid_area": { "col_start": 1, "row_start": 1, "col_end": 3, "row_end": 3 } }`
async fn move_component(
    state: State<LowcodeState>,
    Path((layout_id, node_path)): Path<(Uuid, String)>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let new_area: GridArea = match body.get("grid_area") {
        Some(v) => match serde_json::from_value(v.clone()) {
            Ok(ga) => ga,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(
                        serde_json::json!({"error": format!("invalid grid_area: {e}")}),
                    ),
                );
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "missing required field: grid_area"})),
            );
        }
    };

    // TODO: wire to PG repository once layout CRUD is implemented
    let mut layouts = state.layouts.write().await;
    let layout = match layouts.get_mut(&layout_id) {
        Some(l) => l,
        None => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"error": "layout not found"})),
            );
        }
    };

    match LayoutEditor::move_component(layout, &node_path, new_area) {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({"status": "moved"})),
        ),
        Err(e) => editor_error_response(e),
    }
}

/// PATCH /api/lowcode/layout/{id}/node/{*path}/reparent — move to another container.
///
/// Body JSON: `{ "new_parent_path": "0", "grid_area": {...} }`
async fn reparent_component(
    state: State<LowcodeState>,
    Path((layout_id, node_path)): Path<(Uuid, String)>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let new_parent_path = match body.get("new_parent_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(
                    serde_json::json!({"error": "missing required field: new_parent_path"}),
                ),
            );
        }
    };
    let new_area: GridArea = match body.get("grid_area") {
        Some(v) => match serde_json::from_value(v.clone()) {
            Ok(ga) => ga,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(
                        serde_json::json!({"error": format!("invalid grid_area: {e}")}),
                    ),
                );
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "missing required field: grid_area"})),
            );
        }
    };

    // TODO: wire to PG repository once layout CRUD is implemented
    let mut layouts = state.layouts.write().await;
    let layout = match layouts.get_mut(&layout_id) {
        Some(l) => l,
        None => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"error": "layout not found"})),
            );
        }
    };

    match LayoutEditor::reparent_component(layout, &node_path, &new_parent_path, new_area) {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({"status": "reparented"})),
        ),
        Err(e) => editor_error_response(e),
    }
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
// Component registry (#77 — implemented)
// ---------------------------------------------------------------------------

/// GET /api/lowcode/component → JSON array of registered component info.
async fn list_components(state: State<LowcodeState>) -> impl IntoResponse {
    let reg = state.registry.read().await;
    let info = reg.list_info();
    axum::Json(info)
}

/// POST /api/lowcode/component → register a new component type.
///
/// Accepts JSON body: `{ "type_key": "...", "category": "...", "props_schema": {...} }`
async fn register_component(
    state: State<LowcodeState>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let type_key = body.get("type_key").and_then(|v| v.as_str());
    let category = body.get("category").and_then(|v| v.as_str());
    let props_schema = body.get("props_schema").cloned();

    let (Some(type_key), Some(category), Some(props_schema)) = (type_key, category, props_schema)
    else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(
                serde_json::json!({"error": "missing required fields: type_key, category, props_schema"}),
            ),
        );
    };

    let entry = crate::registry::ComponentEntry {
        type_key: type_key.to_string(),
        category: category.to_string(),
        props_schema,
        renderer: Box::new(|node| {
            // Default passthrough renderer for user-registered components
            format!(
                r#"<div class="lc-component lc-{}">{}</div>"#,
                node.type_key, node.props
            )
        }),
    };

    let mut reg = state.registry.write().await;
    reg.register(entry);
    (
        StatusCode::CREATED,
        axum::Json(serde_json::json!({"type_key": type_key, "status": "registered"})),
    )
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

/// Map `EditorError` to an HTTP status + JSON error body.
fn editor_error_response(err: EditorError) -> (StatusCode, axum::Json<serde_json::Value>) {
    let (status, msg) = match &err {
        EditorError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
        EditorError::InvalidPath(m) => (StatusCode::BAD_REQUEST, m.clone()),
        EditorError::GridConflict(m) => (StatusCode::CONFLICT, m.clone()),
        EditorError::InvalidProps(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
    };
    (status, axum::Json(serde_json::json!({"error": msg})))
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
        .route("/api/lowcode/layout/{id}/node", post(place_component))
        .route(
            "/api/lowcode/layout/{id}/node/{*path}",
            patch(update_props).delete(delete_component),
        )
        .route(
            "/api/lowcode/layout/{id}/node/{*path}/move",
            patch(move_component),
        )
        .route(
            "/api/lowcode/layout/{id}/node/{*path}/reparent",
            patch(reparent_component),
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
            get(get_template)
                .put(update_template)
                .delete(delete_template),
        )
        .route(
            "/api/lowcode/component",
            get(list_components).post(register_component),
        )
        .with_state(state)
}
