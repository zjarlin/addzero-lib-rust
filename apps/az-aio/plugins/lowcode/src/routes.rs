//! engine REST API 与 SSR action 路由。

use std::collections::BTreeMap;

use anyhow::{Context, anyhow};
use axum::{
    Router,
    extract::{RawQuery, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use az_aio_platform::core::api_error::{ApiError, ApiForm, ApiJson, ApiPath, parse_usize_param};
use az_engine::{EngineStore, FieldInput, HookInput, ModelInput, PageParams};
use serde_json::{Map, Value, json};

#[derive(Clone)]
pub struct LowcodeApiState {
    pub store: EngineStore,
}

/// 构建 lowcode 插件的新 engine API 路由。
pub fn engine_router(state: LowcodeApiState) -> Router {
    Router::new()
        .route("/api/engine/models", get(list_models).post(create_model))
        .route(
            "/api/engine/models/{model_name}",
            get(get_model).put(update_model).delete(delete_model),
        )
        .route(
            "/api/engine/models/{model_name}/fields",
            get(list_fields).post(create_field),
        )
        .route(
            "/api/engine/models/{model_name}/fields/{field_id}",
            get(get_field).put(update_field).delete(delete_field),
        )
        .route(
            "/api/engine/models/{model_name}/hooks",
            get(list_hooks).post(create_hook),
        )
        .route(
            "/api/engine/models/{model_name}/hooks/{hook_id}",
            get(get_hook).put(update_hook).delete(delete_hook),
        )
        .route(
            "/api/engine/models/{model_name}/records",
            get(list_records).post(insert_record),
        )
        .route(
            "/api/engine/models/{model_name}/records/{record_id}",
            get(get_record).put(update_record).delete(delete_record),
        )
        .route("/api/engine/ui-action", post(ui_action))
        .with_state(state)
}

async fn list_models(
    State(state): State<LowcodeApiState>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let page = match page_query_from_raw(raw_query.as_deref()) {
        Ok(page) => page,
        Err(error) => return ApiError::from(error).into_response(),
    };
    into_api_response(state.store.list_models(page).await)
}

async fn create_model(
    State(state): State<LowcodeApiState>,
    ApiJson(input): ApiJson<ModelInput>,
) -> Response {
    into_api_response(state.store.create_model(input).await)
}

async fn get_model(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
) -> Response {
    into_api_response(state.store.get_model(&model_name).await)
}

async fn delete_model(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
) -> Response {
    into_api_response(
        async {
            state.store.delete_model(&model_name).await?;
            Ok(json!({ "deleted": true }))
        }
        .await,
    )
}

async fn update_model(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
    ApiJson(input): ApiJson<ModelInput>,
) -> Response {
    into_api_response(state.store.update_model(&model_name, input).await)
}

async fn list_fields(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
) -> Response {
    into_api_response(state.store.list_fields(&model_name).await)
}

async fn create_field(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
    ApiJson(input): ApiJson<FieldInput>,
) -> Response {
    into_api_response(state.store.create_field(&model_name, input).await)
}

async fn delete_field(
    State(state): State<LowcodeApiState>,
    ApiPath((_model_name, field_id)): ApiPath<(String, String)>,
) -> Response {
    into_api_response(
        async {
            state.store.delete_field(&field_id).await?;
            Ok(json!({ "deleted": true }))
        }
        .await,
    )
}

async fn get_field(
    State(state): State<LowcodeApiState>,
    ApiPath((_model_name, field_id)): ApiPath<(String, String)>,
) -> Response {
    into_api_response(state.store.get_field(&field_id).await)
}

async fn update_field(
    State(state): State<LowcodeApiState>,
    ApiPath((_model_name, field_id)): ApiPath<(String, String)>,
    ApiJson(input): ApiJson<FieldInput>,
) -> Response {
    into_api_response(state.store.update_field(&field_id, input).await)
}

async fn list_hooks(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
) -> Response {
    into_api_response(state.store.list_hooks(&model_name).await)
}

async fn create_hook(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
    ApiJson(input): ApiJson<HookInput>,
) -> Response {
    into_api_response(state.store.create_hook(&model_name, input).await)
}

async fn delete_hook(
    State(state): State<LowcodeApiState>,
    ApiPath((_model_name, hook_id)): ApiPath<(String, String)>,
) -> Response {
    into_api_response(
        async {
            state.store.delete_hook(&hook_id).await?;
            Ok(json!({ "deleted": true }))
        }
        .await,
    )
}

async fn get_hook(
    State(state): State<LowcodeApiState>,
    ApiPath((_model_name, hook_id)): ApiPath<(String, String)>,
) -> Response {
    into_api_response(state.store.get_hook(&hook_id).await)
}

async fn update_hook(
    State(state): State<LowcodeApiState>,
    ApiPath((_model_name, hook_id)): ApiPath<(String, String)>,
    ApiJson(input): ApiJson<HookInput>,
) -> Response {
    into_api_response(state.store.update_hook(&hook_id, input).await)
}

async fn list_records(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let page = match page_query_from_raw(raw_query.as_deref()) {
        Ok(page) => page,
        Err(error) => return ApiError::from(error).into_response(),
    };
    into_api_response(state.store.executor().list_records(&model_name, page).await)
}

async fn insert_record(
    State(state): State<LowcodeApiState>,
    ApiPath(model_name): ApiPath<String>,
    ApiJson(payload): ApiJson<Value>,
) -> Response {
    into_api_response(
        state
            .store
            .executor()
            .insert_record(&model_name, payload)
            .await,
    )
}

async fn get_record(
    State(state): State<LowcodeApiState>,
    ApiPath((model_name, record_id)): ApiPath<(String, String)>,
) -> Response {
    into_api_response(
        state
            .store
            .executor()
            .get_record(&model_name, &record_id)
            .await,
    )
}

async fn update_record(
    State(state): State<LowcodeApiState>,
    ApiPath((model_name, record_id)): ApiPath<(String, String)>,
    ApiJson(payload): ApiJson<Value>,
) -> Response {
    into_api_response(
        state
            .store
            .executor()
            .update_record(&model_name, &record_id, payload)
            .await,
    )
}

async fn delete_record(
    State(state): State<LowcodeApiState>,
    ApiPath((model_name, record_id)): ApiPath<(String, String)>,
) -> Response {
    into_api_response(
        async {
            state
                .store
                .executor()
                .delete_record(&model_name, &record_id)
                .await?;
            Ok(json!({ "deleted": true }))
        }
        .await,
    )
}

async fn ui_action(
    State(state): State<LowcodeApiState>,
    ApiForm(form): ApiForm<BTreeMap<String, String>>,
) -> Response {
    let redirect = match apply_ui_action(&state.store, form).await {
        Ok(route) => route,
        Err(error) => {
            let route = format!("/lowcode?error={}", urlencoding::encode(&error.to_string()));
            format!("/?route={}", urlencoding::encode(&route))
        }
    };
    Redirect::to(&redirect).into_response()
}

async fn apply_ui_action(
    store: &EngineStore,
    form: BTreeMap<String, String>,
) -> anyhow::Result<String> {
    let action = form_value(&form, "action");
    match action.as_deref() {
        Some("create_model") => {
            let name = required_form_value(&form, "name")?;
            let display_name = match form_value(&form, "display_name") {
                Some(value) => value,
                None => name.clone(),
            };
            store
                .create_model(ModelInput {
                    name: name.clone(),
                    display_name,
                })
                .await?;
            Ok(lowcode_route(Some(&name), "fields"))
        }
        Some("delete_model") => {
            let model_name = required_form_value(&form, "model_name")?;
            store.delete_model(&model_name).await?;
            Ok(lowcode_route(None, "fields"))
        }
        Some("update_model") => {
            let model_name = required_form_value(&form, "model_name")?;
            let name = match form_value(&form, "name") {
                Some(value) => value,
                None => model_name.clone(),
            };
            store
                .update_model(
                    &model_name,
                    ModelInput {
                        name,
                        display_name: required_form_value(&form, "display_name")?,
                    },
                )
                .await?;
            Ok(lowcode_route(Some(&model_name), "fields"))
        }
        Some("create_field") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .create_field(&model_name, field_input_from_form(&form)?)
                .await?;
            Ok(lowcode_route(Some(&model_name), "fields"))
        }
        Some("update_field") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .update_field(
                    &required_form_value(&form, "field_id")?,
                    field_input_from_form(&form)?,
                )
                .await?;
            Ok(lowcode_route(Some(&model_name), "fields"))
        }
        Some("delete_field") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .delete_field(&required_form_value(&form, "field_id")?)
                .await?;
            Ok(lowcode_route(Some(&model_name), "fields"))
        }
        Some("create_hook") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .create_hook(&model_name, hook_input_from_form(&form, true)?)
                .await?;
            Ok(lowcode_route(Some(&model_name), "hooks"))
        }
        Some("update_hook") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .update_hook(
                    &required_form_value(&form, "hook_id")?,
                    hook_input_from_form(&form, false)?,
                )
                .await?;
            Ok(lowcode_route(Some(&model_name), "hooks"))
        }
        Some("delete_hook") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .delete_hook(&required_form_value(&form, "hook_id")?)
                .await?;
            Ok(lowcode_route(Some(&model_name), "hooks"))
        }
        Some("create_record") => {
            let model_name = required_form_value(&form, "model_name")?;
            let fields = store.list_fields(&model_name).await?;
            let payload = payload_from_form(&fields, &form, EmptyFieldPolicy::Omit)?;
            store
                .executor()
                .insert_record(&model_name, Value::Object(payload))
                .await?;
            Ok(lowcode_route(Some(&model_name), "records"))
        }
        Some("update_record") => {
            let model_name = required_form_value(&form, "model_name")?;
            let record_id = required_form_value(&form, "record_id")?;
            let fields = store.list_fields(&model_name).await?;
            let payload = payload_from_form(&fields, &form, EmptyFieldPolicy::Null)?;
            store
                .executor()
                .update_record(&model_name, &record_id, Value::Object(payload))
                .await?;
            Ok(lowcode_route(Some(&model_name), "records"))
        }
        Some("delete_record") => {
            let model_name = required_form_value(&form, "model_name")?;
            store
                .executor()
                .delete_record(&model_name, &required_form_value(&form, "record_id")?)
                .await?;
            Ok(lowcode_route(Some(&model_name), "records"))
        }
        Some(other) => Err(anyhow!("未知 engine UI action: {other}")),
        None => Err(anyhow!("缺少 engine UI action")),
    }
}

fn field_input_from_form(form: &BTreeMap<String, String>) -> anyhow::Result<FieldInput> {
    Ok(FieldInput {
        name: required_form_value(form, "name")?,
        display_name: required_form_value(form, "display_name")?,
        field_type: required_form_value(form, "field_type")?,
        is_required: form_bool(form, "is_required"),
        expression: form_value(form, "expression"),
        dependency_json: form_value(form, "dependency_json"),
        order_index: form_i32(form, "order_index")?,
    })
}

fn hook_input_from_form(
    form: &BTreeMap<String, String>,
    default_active: bool,
) -> anyhow::Result<HookInput> {
    let is_active = if default_active {
        form_bool_default_true(form, "is_active")
    } else {
        form_bool(form, "is_active")
    };
    Ok(HookInput {
        trigger_event: required_form_value(form, "trigger_event")?,
        script_content: required_form_value(form, "script_content")?,
        is_active,
        order_index: form_i32(form, "order_index")?,
    })
}

fn payload_from_form(
    fields: &[az_engine::MetaField],
    form: &BTreeMap<String, String>,
    empty_policy: EmptyFieldPolicy,
) -> anyhow::Result<Map<String, Value>> {
    let mut payload = Map::new();
    for field in fields {
        if field.field_type == "computed" {
            continue;
        }
        let key = format!("payload_{}", field.name);
        if field.field_type == "boolean" {
            payload.insert(field.name.clone(), Value::Bool(form_bool(form, &key)));
            continue;
        }
        let Some(raw) = form_value(form, &key) else {
            continue;
        };
        if raw.is_empty() && !field.is_required {
            if empty_policy == EmptyFieldPolicy::Null {
                payload.insert(field.name.clone(), Value::Null);
            }
            continue;
        }
        let value = match field.field_type.as_str() {
            "string" => Value::String(raw),
            "int" | "datetime" => Value::Number(
                raw.parse::<i64>()
                    .with_context(|| format!("字段 {} 需要整数", field.name))?
                    .into(),
            ),
            "decimal" => json!(
                raw.parse::<f64>()
                    .with_context(|| format!("字段 {} 需要数字", field.name))?
            ),
            "json" => serde_json::from_str(&raw)
                .with_context(|| format!("字段 {} 需要合法 JSON", field.name))?,
            _ => Value::String(raw),
        };
        payload.insert(field.name.clone(), value);
    }
    Ok(payload)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmptyFieldPolicy {
    Omit,
    Null,
}

fn into_api_response<T: serde::Serialize>(result: anyhow::Result<T>) -> Response {
    az_aio_platform::core::api_error::into_api_response(result)
}

fn page_query_from_raw(raw_query: Option<&str>) -> anyhow::Result<PageParams> {
    let mut page = PageParams::default();
    for pair in raw_query.unwrap_or_default().split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = urlencoding::decode(raw_key)
            .with_context(|| format!("分页参数名解码失败: {raw_key}"))?;
        let value = urlencoding::decode(raw_value)
            .with_context(|| format!("分页参数值解码失败: {raw_key}"))?;
        apply_page_query_pair(&mut page, &key, &value)?;
    }
    Ok(page)
}

fn apply_page_query_pair(page: &mut PageParams, key: &str, value: &str) -> anyhow::Result<()> {
    match key {
        "o" => {
            page.o = parse_usize_param("o", value)?;
        }
        "s" => {
            page.s = parse_usize_param("s", value)?;
        }
        _ => {}
    }
    Ok(())
}

fn required_form_value(form: &BTreeMap<String, String>, key: &str) -> anyhow::Result<String> {
    form_value(form, key).ok_or_else(|| anyhow!("缺少表单字段: {key}"))
}

fn form_value(form: &BTreeMap<String, String>, key: &str) -> Option<String> {
    form.get(key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn form_i32(form: &BTreeMap<String, String>, key: &str) -> anyhow::Result<i32> {
    let Some(value) = form_value(form, key) else {
        return Ok(0);
    };
    value
        .parse::<i32>()
        .with_context(|| format!("表单字段 {key} 需要整数"))
}

fn form_bool(form: &BTreeMap<String, String>, key: &str) -> bool {
    matches!(
        form.get(key).map(String::as_str),
        Some("1" | "true" | "on" | "yes")
    )
}

fn form_bool_default_true(form: &BTreeMap<String, String>, key: &str) -> bool {
    !matches!(
        form.get(key).map(String::as_str),
        Some("0" | "false" | "off" | "no")
    )
}

fn lowcode_route(model_name: Option<&str>, tab: &str) -> String {
    let route = match model_name {
        Some(model_name) => format!(
            "/lowcode?model={}&tab={tab}",
            urlencoding::encode(model_name)
        ),
        None => format!("/lowcode?tab={tab}"),
    };
    format!("/?route={}", urlencoding::encode(&route))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_route_uses_new_lowcode_path() {
        let route = lowcode_route(Some("order"), "records");

        // SSR action 必须回到编码后的 route=/lowcode?... 路由。
        assert_eq!(route, "/?route=%2Flowcode%3Fmodel%3Dorder%26tab%3Drecords");
    }

    #[test]
    fn page_query_defaults_to_contract_page_size() {
        let page = page_query_from_raw(Some("o=20&s=50")).unwrap();

        // 列表接口继续使用 o/s 入参并回传 p。
        assert_eq!(page, PageParams { o: 20, s: 50 });
    }

    #[test]
    fn field_form_builds_update_input() {
        let form = BTreeMap::from([
            ("name".to_string(), "amount".to_string()),
            ("display_name".to_string(), "金额".to_string()),
            ("field_type".to_string(), "decimal".to_string()),
            ("is_required".to_string(), "1".to_string()),
            ("order_index".to_string(), "7".to_string()),
        ]);

        let input = field_input_from_form(&form).unwrap();

        // UI action 的字段编辑表单必须能落到真实 update_field 入参。
        assert_eq!(input.name, "amount");
        assert_eq!(input.field_type, "decimal");
        assert!(input.is_required);
        assert_eq!(input.order_index, 7);
    }

    #[test]
    fn hook_form_builds_inactive_update_input() {
        let form = BTreeMap::from([
            ("trigger_event".to_string(), "after_update".to_string()),
            ("script_content".to_string(), "let x = 1;".to_string()),
            ("order_index".to_string(), "3".to_string()),
        ]);

        let input = hook_input_from_form(&form, false).unwrap();

        // 未勾选启用时，更新钩子必须能真实写回 inactive。
        assert_eq!(input.trigger_event, "after_update");
        assert!(!input.is_active);
        assert_eq!(input.order_index, 3);
    }
}
