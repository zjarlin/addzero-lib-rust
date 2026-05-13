/// Axum router for the lowcode service.
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::edge::AzEdgeSpec;
use crate::editor::{EditorError, LayoutEditor};
use crate::events::EventContext;
use crate::schema::GridArea;
use crate::scripting::{ScriptError, ValidateResponse};
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
                axum::Json(serde_json::json!({"error": "missing required field: component_type"})),
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
    let props = body.get("props").cloned().unwrap_or(serde_json::json!({}));

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
    let props_patch = body.get("props_patch").cloned().unwrap_or(body);

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
                axum::Json(serde_json::json!({"error": "missing required field: new_parent_path"})),
            );
        }
    };
    let new_area: GridArea = match body.get("grid_area") {
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

/// POST /api/lowcode/event — dispatch a component event through the handler
/// registry.
///
/// Body JSON:
/// ```json
/// {
///   "handler_type": "navigate",
///   "config": { "url": "/dashboard" },
///   "context": {
///     "component_path": "root/button1",
///     "event_type": "click",
///     "component_props": {}
///   }
/// }
/// ```
async fn handle_event(
    state: State<LowcodeState>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let handler_type = match body.get("handler_type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "missing required field: handler_type"})),
            );
        }
    };
    let config = body.get("config").cloned().unwrap_or(serde_json::json!({}));

    let ctx: EventContext = match body.get("context") {
        Some(v) => match serde_json::from_value(v.clone()) {
            Ok(c) => c,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({"error": format!("invalid context: {e}")})),
                );
            }
        },
        None => EventContext {
            component_path: String::new(),
            event_type: String::new(),
            component_props: serde_json::json!({}),
            form_data: None,
            trigger_value: None,
        },
    };

    match state
        .handler_registry
        .dispatch(&ctx, handler_type, &config, 5_000)
        .await
    {
        Ok(result) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "side_effects": result.side_effects })),
        ),
        Err(e) => {
            let status = match &e {
                crate::events::EventError::HandlerNotFound(_) => StatusCode::NOT_FOUND,
                crate::events::EventError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
                _ => StatusCode::BAD_REQUEST,
            };
            (
                status,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Scripting (skeleton — #80)
// ---------------------------------------------------------------------------

/// POST /api/lowcode/script/validate — validate script syntax.
///
/// Body JSON: `{ "script": "..." }`
/// Returns 200 + `{"valid": true}` or 422 + `{"valid": false, "error": "...", "line": N, "col": N}`.
async fn validate_script(
    state: State<LowcodeState>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let script = match body.get("script").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "missing required field: script"})),
            );
        }
    };

    match state.script_engine.validate(script) {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(
                serde_json::to_value(ValidateResponse {
                    valid: true,
                    error: None,
                    line: None,
                    col: None,
                })
                .unwrap_or_default(),
            ),
        ),
        Err(ScriptError::SyntaxError {
            line,
            column,
            message,
        }) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(
                serde_json::to_value(ValidateResponse {
                    valid: false,
                    error: Some(message),
                    line: Some(line),
                    col: Some(column),
                })
                .unwrap_or_default(),
            ),
        ),
        Err(other) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(serde_json::json!({
                "valid": false,
                "error": other.to_string()
            })),
        ),
    }
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
// Az-edge REST contract generation
// ---------------------------------------------------------------------------

/// POST /api/lowcode/edge/rest-contract — generate a REST interface contract
/// from an `az-edge` card specification.
async fn generate_edge_rest_contract(
    axum::extract::Json(spec): axum::extract::Json<AzEdgeSpec>,
) -> impl IntoResponse {
    match spec.rest_contract() {
        Ok(contract) => (StatusCode::OK, axum::Json(serde_json::json!(contract))),
        Err(error) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
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
            "/api/lowcode/edge/rest-contract",
            post(generate_edge_rest_contract),
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generate_edge_rest_contract_should_return_schema_json() {
        let response = generate_edge_rest_contract(axum::Json(AzEdgeSpec {
            title: "Weather bridge".into(),
            variant: crate::edge::AzEdgeVariant::Curl,
            method: crate::edge::AzEdgeHttpMethod::Post,
            path: "/api/edge/weather".into(),
            template: "curl https://api.example.com/weather?q={{city}}".into(),
            inputs: vec![crate::edge::AzEdgeParam {
                name: "city".into(),
                ty: crate::edge::AzEdgeParamType::String,
                required: true,
                description: None,
                default_value: None,
            }],
            outputs: vec![crate::edge::AzEdgeParam {
                name: "temperature".into(),
                ty: crate::edge::AzEdgeParamType::Number,
                required: true,
                description: None,
                default_value: None,
            }],
            timeout_secs: Some(10),
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["path"], "/api/edge/weather");
        assert_eq!(json["method"], "POST");
        assert_eq!(json["variant"], "curl");
        assert_eq!(
            json["request_schema"]["required"],
            serde_json::json!(["city"])
        );
    }

    #[tokio::test]
    async fn generate_edge_rest_contract_should_reject_unknown_placeholder() {
        let response = generate_edge_rest_contract(axum::Json(AzEdgeSpec {
            title: "Broken bridge".into(),
            variant: crate::edge::AzEdgeVariant::Python,
            method: crate::edge::AzEdgeHttpMethod::Post,
            path: "/api/edge/broken".into(),
            template: "print({{missing}})".into(),
            inputs: vec![],
            outputs: vec![],
            timeout_secs: None,
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(
            json["error"]
                .as_str()
                .unwrap_or_default()
                .contains("unknown template placeholder")
        );
    }
}
