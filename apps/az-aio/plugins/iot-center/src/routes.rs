//! 物联网中心 REST API 与 SSR 表单操作。

use anyhow::{Context, Result, anyhow};
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use az_aio_platform::core::api_error::{ApiError, ApiForm, ApiJson, ApiResponse, ok_json};
use serde::Deserialize;

use crate::{
    contract::{
        APPLY_TEMPLATE_PATH, ApplyIotTemplateRequest, CreateIotDeviceRequest, DEVICES_PATH,
        IotDashboardSnapshot, IotDeviceView, IotTemplateApplyResult, STATUS_PATH, UI_ACTION_PATH,
    },
    service::IotService,
};

/// 物联网中心 API 状态。
#[derive(Clone)]
pub struct IotApiState {
    service: IotService,
}

impl IotApiState {
    /// 创建物联网 API 状态。
    pub fn new(service: IotService) -> Self {
        Self { service }
    }
}

/// 构建物联网中心路由。
pub fn iot_router(state: IotApiState) -> Router {
    Router::new()
        .route(STATUS_PATH, get(status_handler))
        .route(APPLY_TEMPLATE_PATH, post(apply_template_handler))
        .route(DEVICES_PATH, post(create_device_handler))
        .route(UI_ACTION_PATH, post(ui_action_handler))
        .with_state(state)
}

async fn status_handler(
    State(state): State<IotApiState>,
) -> Result<Json<ApiResponse<IotDashboardSnapshot>>, ApiError> {
    state
        .service
        .dashboard()
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn apply_template_handler(
    State(state): State<IotApiState>,
    ApiJson(request): ApiJson<ApplyIotTemplateRequest>,
) -> Result<Json<ApiResponse<IotTemplateApplyResult>>, ApiError> {
    state
        .service
        .apply_template(request)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn create_device_handler(
    State(state): State<IotApiState>,
    ApiJson(request): ApiJson<CreateIotDeviceRequest>,
) -> Result<Json<ApiResponse<IotDeviceView>>, ApiError> {
    state
        .service
        .create_device(request)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn ui_action_handler(
    State(state): State<IotApiState>,
    ApiForm(form): ApiForm<IotUiForm>,
) -> Response {
    let result = apply_ui_action(&state.service, form).await;
    let route = match result {
        Ok(message) => format!(
            "/iot?view=devices&message={}",
            urlencoding::encode(&message)
        ),
        Err(error) => format!(
            "/iot?view=devices&error={}",
            urlencoding::encode(&error.to_string())
        ),
    };
    let redirect = format!("/?route={}", urlencoding::encode(&route));
    Redirect::to(&redirect).into_response()
}

async fn apply_ui_action(service: &IotService, form: IotUiForm) -> Result<String> {
    match form.action.as_str() {
        "apply_template" => {
            let result = service
                .apply_template(ApplyIotTemplateRequest {
                    seed_demo: form.seed_demo.is_some(),
                })
                .await?;
            Ok(format!(
                "模板已就绪：新建 {} 个模型、{} 个字段、{} 条演示数据",
                result.created_models, result.created_fields, result.seeded_records
            ))
        }
        "create_device" => {
            let request = form.into_device_request()?;
            let device = service.create_device(request).await?;
            Ok(format!("设备 {} 已创建", device.device_code))
        }
        other => Err(anyhow!("invalid IoT UI action: {other}")),
    }
}

#[derive(Debug, Deserialize)]
struct IotUiForm {
    action: String,
    seed_demo: Option<String>,
    device_code: Option<String>,
    name: Option<String>,
    product_code: Option<String>,
    product_name: Option<String>,
    gateway_code: Option<String>,
    mqtt_client_id: Option<String>,
    location: Option<String>,
    enabled: Option<String>,
    expected_heartbeat_secs: Option<i64>,
    expected_data_secs: Option<i64>,
}

impl IotUiForm {
    fn into_device_request(self) -> Result<CreateIotDeviceRequest> {
        Ok(CreateIotDeviceRequest {
            device_code: required(self.device_code, "device_code")?,
            name: required(self.name, "name")?,
            product_code: required(self.product_code, "product_code")?,
            product_name: required(self.product_name, "product_name")?,
            gateway_code: self.gateway_code.unwrap_or_default(),
            mqtt_client_id: required(self.mqtt_client_id, "mqtt_client_id")?,
            location: self.location.unwrap_or_default(),
            enabled: self.enabled.is_some(),
            expected_heartbeat_secs: self.expected_heartbeat_secs.unwrap_or(30),
            expected_data_secs: self.expected_data_secs.unwrap_or(60),
        })
    }
}

fn required(value: Option<String>, field: &str) -> Result<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .with_context(|| format!("缺少表单字段: {field}"))
}
