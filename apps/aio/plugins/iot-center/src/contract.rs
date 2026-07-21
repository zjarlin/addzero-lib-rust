//! 物联网中心共享契约。

pub use crate::dictionary::IotOnlineStatus;
use serde::{Deserialize, Serialize};

pub const ROUTE: &str = "/iot";
pub const STATUS_PATH: &str = "/api/iot/status";
pub const APPLY_TEMPLATE_PATH: &str = "/api/iot/templates/default/apply";
pub const DEVICES_PATH: &str = "/api/iot/devices";
pub const UI_ACTION_PATH: &str = "/api/iot/ui-action";
pub const OP_TEMPLATE_APPLY: &str = "iot.templates.default.apply";
pub const OP_DEVICES_CREATE: &str = "iot.devices.create";

pub const PRODUCT_MODEL: &str = "iot_product";
pub const GATEWAY_MODEL: &str = "iot_gateway";
pub const DEVICE_MODEL: &str = "iot_device";
pub const TELEMETRY_MODEL: &str = "iot_telemetry";
pub const ALARM_MODEL: &str = "iot_alarm";

/// 初始化物联网低代码模板的请求。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyIotTemplateRequest {
    #[serde(default)]
    pub seed_demo: bool,
}

/// 物联网低代码模板初始化结果。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotTemplateApplyResult {
    pub created_models: usize,
    pub created_fields: usize,
    pub seeded_records: usize,
    pub model_names: Vec<String>,
}

/// 新建设备请求。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIotDeviceRequest {
    pub device_code: String,
    pub name: String,
    pub product_code: String,
    pub product_name: String,
    pub gateway_code: String,
    pub mqtt_client_id: String,
    pub location: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_heartbeat_period_secs")]
    pub expected_heartbeat_secs: i64,
    #[serde(default = "default_data_period_secs")]
    pub expected_data_secs: i64,
}

/// 页面展示所需的设备投影。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotDeviceView {
    pub record_id: String,
    pub device_code: String,
    pub name: String,
    pub product_code: String,
    pub product_name: String,
    pub gateway_code: String,
    pub mqtt_client_id: String,
    pub location: String,
    pub enabled: bool,
    pub connected: bool,
    pub status: IotOnlineStatus,
    pub last_seen_at_ms: i64,
    pub last_heartbeat_at_ms: i64,
    pub last_data_at_ms: i64,
    pub offline_reason: String,
}

/// 设备状态统计。
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotStatusSummary {
    pub total: usize,
    pub enabled: usize,
    pub online: usize,
    pub heartbeat_lost: usize,
    pub data_anomaly: usize,
    pub offline: usize,
    pub unknown: usize,
}

/// 产品列表投影。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotProductView {
    pub code: String,
    pub name: String,
    pub category: String,
    pub protocol: String,
    pub enabled: bool,
}

/// 网关列表投影。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotGatewayView {
    pub code: String,
    pub name: String,
    pub mqtt_client_id: String,
    pub connected: bool,
    pub location: String,
    pub last_heartbeat_at_ms: i64,
}

/// 遥测列表投影。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotTelemetryView {
    pub device_code: String,
    pub metric_code: String,
    pub value: f64,
    pub unit: String,
    pub quality: String,
    pub collected_at_ms: i64,
}

/// 告警列表投影。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotAlarmView {
    pub device_code: String,
    pub level: String,
    pub message: String,
    pub status: String,
    pub occurred_at_ms: i64,
}

/// 物联网中心聚合快照。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IotDashboardSnapshot {
    pub template_ready: bool,
    pub summary: IotStatusSummary,
    pub products: Vec<IotProductView>,
    pub gateways: Vec<IotGatewayView>,
    pub devices: Vec<IotDeviceView>,
    pub telemetry: Vec<IotTelemetryView>,
    pub alarms: Vec<IotAlarmView>,
}

fn default_true() -> bool {
    true
}

fn default_heartbeat_period_secs() -> i64 {
    30
}

fn default_data_period_secs() -> i64 {
    60
}
