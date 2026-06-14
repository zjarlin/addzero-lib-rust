use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    model::{AppScreen, MetaField, MetaModel},
    store::LowcodeStore,
};

#[derive(Clone)]
pub struct LowcodeApiState {
    pub store: LowcodeStore,
}

pub fn lowcode_router(state: LowcodeApiState) -> Router {
    Router::new()
        .route("/api/lowcode/models", get(list_models).post(create_model))
        .route(
            "/api/lowcode/models/{id}",
            get(get_model).put(update_model).delete(delete_model),
        )
        .route(
            "/api/lowcode/models/{model_id}/fields",
            get(list_fields).post(create_field),
        )
        .route(
            "/api/lowcode/fields/{id}",
            get(get_field).put(update_field).delete(delete_field),
        )
        .route("/api/lowcode/screens", get(list_screens).post(create_screen))
        .route(
            "/api/lowcode/screens/{id}",
            get(get_screen).delete(delete_screen),
        )
        // Form POST → redirect
        .route(
            "/api/lowcode/models/create-redirect",
            axum::routing::post(create_model_redirect),
        )
        .route(
            "/api/lowcode/models/{model_id}/fields/create-redirect",
            axum::routing::post(create_field_redirect),
        )
        .route(
            "/api/lowcode/fields/{id}/delete-redirect",
            axum::routing::post(delete_field_redirect),
        )
        .with_state(state)
}

// ── MetaModel ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateModelInput {
    name: String,
    label: String,
    #[serde(default)]
    description: String,
}

async fn list_models(State(s): State<LowcodeApiState>) -> Json<Vec<crate::model::MetaModelSummary>> {
    Json(s.store.list_models().await.unwrap_or_default())
}

async fn create_model(
    State(s): State<LowcodeApiState>,
    Json(input): Json<CreateModelInput>,
) -> Json<MetaModel> {
    let now = chrono::Utc::now().to_rfc3339();
    let model = MetaModel {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        label: input.label,
        description: input.description,
        created_at: now.clone(),
        updated_at: now,
    };
    Json(s.store.create_model(model).await.unwrap())
}

async fn get_model(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
) -> Json<Option<MetaModel>> {
    Json(s.store.get_model(&id).await.unwrap())
}

#[derive(Deserialize)]
struct UpdateModelInput {
    name: Option<String>,
    label: Option<String>,
    description: Option<String>,
}

async fn update_model(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateModelInput>,
) -> Json<serde_json::Value> {
    let Some(mut model) = s.store.get_model(&id).await.unwrap() else {
        return Json(serde_json::json!({ "error": "not found" }));
    };
    if let Some(v) = input.name {
        model.name = v;
    }
    if let Some(v) = input.label {
        model.label = v;
    }
    if let Some(v) = input.description {
        model.description = v;
    }
    model.updated_at = chrono::Utc::now().to_rfc3339();
    s.store.update_model(&model).await.unwrap();
    Json(serde_json::json!({ "ok": true }))
}

async fn delete_model(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    s.store.delete_model(&id).await.unwrap();
    Json(serde_json::json!({ "ok": true }))
}

// ── MetaField ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateFieldInput {
    name: String,
    label: String,
    field_type: String,
    #[serde(default)]
    relation_type: Option<String>,
    #[serde(default)]
    relation_model_id: Option<String>,
    #[serde(default)]
    is_required: bool,
    #[serde(default)]
    is_unique: bool,
    #[serde(default)]
    order: i32,
    #[serde(default)]
    default_value: Option<String>,
}

async fn list_fields(
    State(s): State<LowcodeApiState>,
    Path(model_id): Path<String>,
) -> Json<Vec<crate::model::MetaFieldView>> {
    Json(s.store.list_fields(&model_id).await.unwrap_or_default())
}

async fn create_field(
    State(s): State<LowcodeApiState>,
    Path(model_id): Path<String>,
    Json(input): Json<CreateFieldInput>,
) -> Json<MetaField> {
    let now = chrono::Utc::now().to_rfc3339();
    let field = MetaField {
        id: Uuid::new_v4().to_string(),
        model_id,
        name: input.name,
        label: input.label,
        field_type: input.field_type,
        relation_type: input.relation_type,
        relation_model_id: input.relation_model_id,
        is_required: input.is_required,
        is_unique: input.is_unique,
        order: input.order,
        default_value: input.default_value,
        created_at: now.clone(),
        updated_at: now,
    };
    Json(s.store.create_field(&field).await.unwrap())
}

async fn get_field(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
) -> Json<Option<MetaField>> {
    Json(s.store.get_field(&id).await.unwrap())
}

#[derive(Deserialize)]
struct UpdateFieldInput {
    name: Option<String>,
    label: Option<String>,
    field_type: Option<String>,
    relation_type: Option<String>,
    relation_model_id: Option<String>,
    is_required: Option<bool>,
    is_unique: Option<bool>,
    order: Option<i32>,
    default_value: Option<String>,
}

async fn update_field(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateFieldInput>,
) -> Json<serde_json::Value> {
    let Some(mut field) = s.store.get_field(&id).await.unwrap() else {
        return Json(serde_json::json!({ "error": "not found" }));
    };
    if let Some(v) = input.name {
        field.name = v;
    }
    if let Some(v) = input.label {
        field.label = v;
    }
    if let Some(v) = input.field_type {
        field.field_type = v;
    }
    if input.relation_type.is_some() {
        field.relation_type = input.relation_type;
    }
    if input.relation_model_id.is_some() {
        field.relation_model_id = input.relation_model_id;
    }
    if let Some(v) = input.is_required {
        field.is_required = v;
    }
    if let Some(v) = input.is_unique {
        field.is_unique = v;
    }
    if let Some(v) = input.order {
        field.order = v;
    }
    if input.default_value.is_some() {
        field.default_value = input.default_value;
    }
    field.updated_at = chrono::Utc::now().to_rfc3339();
    s.store.update_field(&field).await.unwrap();
    Json(serde_json::json!({ "ok": true }))
}

async fn delete_field(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    s.store.delete_field(&id).await.unwrap();
    Json(serde_json::json!({ "ok": true }))
}

// ── AppScreen ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateScreenInput {
    name: String,
    label: String,
    layout: String,
    model_id: String,
    #[serde(default)]
    config_json: String,
}

async fn list_screens(
    State(s): State<LowcodeApiState>,
) -> Json<Vec<crate::model::AppScreenSummary>> {
    Json(s.store.list_screens().await.unwrap_or_default())
}

async fn get_screen(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
) -> Json<Option<AppScreen>> {
    Json(s.store.get_screen(&id).await.unwrap())
}

async fn create_screen(
    State(s): State<LowcodeApiState>,
    Json(input): Json<CreateScreenInput>,
) -> Json<AppScreen> {
    let now = chrono::Utc::now().to_rfc3339();
    let screen = AppScreen {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        label: input.label,
        layout: input.layout,
        model_id: input.model_id,
        config_json: input.config_json,
        created_at: now.clone(),
        updated_at: now,
    };
    Json(s.store.create_screen(&screen).await.unwrap())
}

async fn delete_screen(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    s.store.delete_screen(&id).await.unwrap();
    Json(serde_json::json!({ "ok": true }))
}

// ── Form POST → redirect ───────────────────────────────────────

#[derive(Deserialize)]
struct CreateModelForm {
    name: String,
    label: String,
    #[serde(default)]
    description: String,
}

async fn create_model_redirect(
    State(s): State<LowcodeApiState>,
    axum::extract::Form(form): axum::extract::Form<CreateModelForm>,
) -> axum::response::Redirect {
    let now = chrono::Utc::now().to_rfc3339();
    let model = MetaModel {
        id: Uuid::new_v4().to_string(),
        name: form.name,
        label: form.label,
        description: form.description,
        created_at: now.clone(),
        updated_at: now,
    };
    let _ = s.store.create_model(model.clone()).await;
    axum::response::Redirect::to(&format!("/?route=/lowcode&model={}", model.id))
}

#[derive(Deserialize)]
struct CreateFieldForm {
    name: String,
    label: String,
    field_type: String,
    #[serde(default)]
    order: i32,
    #[serde(default)]
    default_value: Option<String>,
}

async fn create_field_redirect(
    State(s): State<LowcodeApiState>,
    Path(model_id): Path<String>,
    axum::extract::Form(form): axum::extract::Form<CreateFieldForm>,
) -> axum::response::Redirect {
    let now = chrono::Utc::now().to_rfc3339();
    let field = MetaField {
        id: Uuid::new_v4().to_string(),
        model_id: model_id.clone(),
        name: form.name,
        label: form.label,
        field_type: form.field_type,
        relation_type: None,
        relation_model_id: None,
        is_required: false,
        is_unique: false,
        order: form.order,
        default_value: form.default_value,
        created_at: now.clone(),
        updated_at: now,
    };
    let _ = s.store.create_field(&field).await;
    axum::response::Redirect::to(&format!("/?route=/lowcode&model={model_id}"))
}

#[derive(Deserialize)]
struct DeleteFieldForm {
    redirect_model: String,
}

async fn delete_field_redirect(
    State(s): State<LowcodeApiState>,
    Path(id): Path<String>,
    axum::extract::Form(form): axum::extract::Form<DeleteFieldForm>,
) -> axum::response::Redirect {
    let _ = s.store.delete_field(&id).await;
    axum::response::Redirect::to(&format!("/?route=/lowcode&model={}", form.redirect_model))
}
