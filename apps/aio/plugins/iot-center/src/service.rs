//! 基于低代码 EngineStore 的物联网领域服务。

use std::{
    collections::BTreeSet,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use az_engine::{EngineStore, FieldInput, ModelInput, PageParams};
use serde_json::{Value, json};

use crate::contract::{
    ALARM_MODEL, ApplyIotTemplateRequest, CreateIotDeviceRequest, DEVICE_MODEL, GATEWAY_MODEL,
    IotAlarmView, IotDashboardSnapshot, IotDeviceView, IotGatewayView, IotOnlineStatus,
    IotProductView, IotStatusSummary, IotTelemetryView, IotTemplateApplyResult, PRODUCT_MODEL,
    TELEMETRY_MODEL,
};

#[derive(Clone, Copy)]
struct FieldDefinition {
    name: &'static str,
    display_name: &'static str,
    field_type: &'static str,
    required: bool,
}

#[derive(Clone, Copy)]
struct ModelDefinition {
    name: &'static str,
    display_name: &'static str,
    fields: &'static [FieldDefinition],
}

const PRODUCT_FIELDS: &[FieldDefinition] = &[
    field("code", "产品编码", "string", true),
    field("name", "产品名称", "string", true),
    field("category", "产品分类", "string", true),
    field("protocol", "接入协议", "string", true),
    field("enabled", "启用", "boolean", true),
];

const GATEWAY_FIELDS: &[FieldDefinition] = &[
    field("code", "网关编码", "string", true),
    field("name", "网关名称", "string", true),
    field("mqtt_client_id", "MQTT ClientId", "string", true),
    field("connected", "连接状态", "boolean", true),
    field("last_seen_at_ms", "最后消息", "datetime", false),
    field("last_heartbeat_at_ms", "最后心跳", "datetime", false),
    field("expected_heartbeat_secs", "心跳周期秒", "int", true),
    field("location", "安装位置", "string", false),
    field("enabled", "启用", "boolean", true),
];

const DEVICE_FIELDS: &[FieldDefinition] = &[
    field("device_code", "设备编码", "string", true),
    field("name", "设备名称", "string", true),
    field("product_code", "产品编码", "string", true),
    field("product_name", "产品名称", "string", true),
    field("gateway_code", "网关编码", "string", false),
    field("mqtt_client_id", "MQTT ClientId", "string", true),
    field("location", "安装位置", "string", false),
    field("enabled", "启用", "boolean", true),
    field("connected", "连接状态", "boolean", true),
    field("last_seen_at_ms", "最后消息", "datetime", false),
    field("last_heartbeat_at_ms", "最后心跳", "datetime", false),
    field("last_data_at_ms", "最后数据", "datetime", false),
    field("expected_heartbeat_secs", "心跳周期秒", "int", true),
    field("expected_data_secs", "数据周期秒", "int", true),
    field("offline_reason", "离线原因", "string", false),
];

const TELEMETRY_FIELDS: &[FieldDefinition] = &[
    field("device_code", "设备编码", "string", true),
    field("metric_code", "指标编码", "string", true),
    field("value", "指标值", "decimal", true),
    field("unit", "单位", "string", false),
    field("quality", "数据质量", "string", true),
    field("collected_at_ms", "采集时间", "datetime", true),
];

const ALARM_FIELDS: &[FieldDefinition] = &[
    field("device_code", "设备编码", "string", true),
    field("level", "告警等级", "string", true),
    field("message", "告警内容", "string", true),
    field("status", "处理状态", "string", true),
    field("occurred_at_ms", "发生时间", "datetime", true),
];

const MODEL_DEFINITIONS: &[ModelDefinition] = &[
    model(PRODUCT_MODEL, "物联网产品", PRODUCT_FIELDS),
    model(GATEWAY_MODEL, "边缘网关", GATEWAY_FIELDS),
    model(DEVICE_MODEL, "物联网设备", DEVICE_FIELDS),
    model(TELEMETRY_MODEL, "设备遥测", TELEMETRY_FIELDS),
    model(ALARM_MODEL, "设备告警", ALARM_FIELDS),
];

const fn field(
    name: &'static str,
    display_name: &'static str,
    field_type: &'static str,
    required: bool,
) -> FieldDefinition {
    FieldDefinition {
        name,
        display_name,
        field_type,
        required,
    }
}

const fn model(
    name: &'static str,
    display_name: &'static str,
    fields: &'static [FieldDefinition],
) -> ModelDefinition {
    ModelDefinition {
        name,
        display_name,
        fields,
    }
}

/// 使用共享 EngineStore 管理物联网低代码模型和记录。
#[derive(Clone)]
pub struct IotService {
    store: EngineStore,
}

impl IotService {
    /// 创建物联网领域服务。
    pub fn new(store: EngineStore) -> Self {
        Self { store }
    }

    /// 初始化五个低代码模型，并按需写入演示数据。
    pub async fn apply_template(
        &self,
        request: ApplyIotTemplateRequest,
    ) -> Result<IotTemplateApplyResult> {
        let mut created_models = 0;
        let mut created_fields = 0;
        for definition in MODEL_DEFINITIONS {
            if self.store.get_model(definition.name).await?.is_none() {
                let input = ModelInput {
                    name: definition.name.to_string(),
                    display_name: definition.display_name.to_string(),
                };
                self.store.create_model(input).await?;
                created_models += 1;
            }
            let existing_names = self
                .store
                .list_fields(definition.name)
                .await?
                .into_iter()
                .map(|item| item.name)
                .collect::<BTreeSet<_>>();
            for (order_index, field) in definition.fields.iter().enumerate() {
                if existing_names.contains(field.name) {
                    continue;
                }
                let input = FieldInput {
                    name: field.name.to_string(),
                    display_name: field.display_name.to_string(),
                    field_type: field.field_type.to_string(),
                    is_required: field.required,
                    expression: None,
                    dependency_json: None,
                    order_index: order_index as i32,
                };
                self.store.create_field(definition.name, input).await?;
                created_fields += 1;
            }
        }
        let seeded_records = if request.seed_demo {
            self.seed_demo_records().await?
        } else {
            0
        };
        Ok(IotTemplateApplyResult {
            created_models,
            created_fields,
            seeded_records,
            model_names: MODEL_DEFINITIONS
                .iter()
                .map(|definition| definition.name.to_string())
                .collect(),
        })
    }

    /// 返回页面和 API 共用的物联网聚合快照。
    pub async fn dashboard(&self) -> Result<IotDashboardSnapshot> {
        if !self.template_ready().await? {
            return Ok(IotDashboardSnapshot::default());
        }
        let products = self.list_records(PRODUCT_MODEL).await?;
        let gateways = self.list_records(GATEWAY_MODEL).await?;
        let devices = self.list_records(DEVICE_MODEL).await?;
        let telemetry = self.list_records(TELEMETRY_MODEL).await?;
        let alarms = self.list_records(ALARM_MODEL).await?;
        let now_ms = timestamp_ms();
        let devices = devices
            .into_iter()
            .map(|record| device_view(record.id, &record.payload, now_ms))
            .collect::<Vec<_>>();
        let summary = summarize_devices(&devices);
        Ok(IotDashboardSnapshot {
            template_ready: true,
            summary,
            products: products
                .iter()
                .map(|record| product_view(&record.payload))
                .collect(),
            gateways: gateways
                .iter()
                .map(|record| gateway_view(&record.payload))
                .collect(),
            devices,
            telemetry: telemetry
                .iter()
                .map(|record| telemetry_view(&record.payload))
                .collect(),
            alarms: alarms
                .iter()
                .map(|record| alarm_view(&record.payload))
                .collect(),
        })
    }

    /// 新建设备并校验设备编码和 MQTT ClientId 唯一性。
    pub async fn create_device(&self, request: CreateIotDeviceRequest) -> Result<IotDeviceView> {
        validate_device_request(&request)?;
        let existing = self.list_records(DEVICE_MODEL).await?;
        if existing
            .iter()
            .any(|record| text(&record.payload, "device_code") == request.device_code)
        {
            bail!("duplicate device code: {}", request.device_code);
        }
        if existing
            .iter()
            .any(|record| text(&record.payload, "mqtt_client_id") == request.mqtt_client_id)
        {
            bail!("duplicate MQTT ClientId: {}", request.mqtt_client_id);
        }
        let payload = json!({
            "device_code": request.device_code,
            "name": request.name,
            "product_code": request.product_code,
            "product_name": request.product_name,
            "gateway_code": request.gateway_code,
            "mqtt_client_id": request.mqtt_client_id,
            "location": request.location,
            "enabled": request.enabled,
            "connected": false,
            "last_seen_at_ms": 0,
            "last_heartbeat_at_ms": 0,
            "last_data_at_ms": 0,
            "expected_heartbeat_secs": request.expected_heartbeat_secs,
            "expected_data_secs": request.expected_data_secs,
            "offline_reason": "尚未接入",
        });
        let record = self
            .store
            .executor()
            .insert_record(DEVICE_MODEL, payload)
            .await?;
        Ok(device_view(record.id, &record.payload, timestamp_ms()))
    }

    async fn template_ready(&self) -> Result<bool> {
        for definition in MODEL_DEFINITIONS {
            if self.store.get_model(definition.name).await?.is_none() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn list_records(&self, model_name: &str) -> Result<Vec<az_engine::DataRecordView>> {
        let page = self
            .store
            .executor()
            .list_records(model_name, PageParams { o: 0, s: 500 })
            .await?;
        Ok(page.d)
    }

    async fn seed_demo_records(&self) -> Result<usize> {
        let now = timestamp_ms();
        let datasets = demo_datasets(now);
        let mut inserted = 0;
        for (model_name, records) in datasets {
            let existing = self.list_records(model_name).await?;
            if !existing.is_empty() {
                continue;
            }
            for payload in records {
                self.store
                    .executor()
                    .insert_record(model_name, payload)
                    .await
                    .with_context(|| format!("写入物联网演示记录失败: {model_name}"))?;
                inserted += 1;
            }
        }
        Ok(inserted)
    }
}

/// 按连接、心跳和数据新鲜度计算设备业务状态。
pub fn evaluate_online_status(
    connected: bool,
    last_seen_at_ms: i64,
    last_heartbeat_at_ms: i64,
    last_data_at_ms: i64,
    expected_heartbeat_secs: i64,
    expected_data_secs: i64,
    now_ms: i64,
) -> IotOnlineStatus {
    if last_seen_at_ms <= 0 {
        return IotOnlineStatus::Unknown;
    }
    if !connected {
        return IotOnlineStatus::Offline;
    }
    if is_expired(
        last_heartbeat_at_ms,
        expected_heartbeat_secs.saturating_mul(3),
        now_ms,
    ) {
        return IotOnlineStatus::HeartbeatLost;
    }
    if is_expired(
        last_data_at_ms,
        expected_data_secs.saturating_mul(2),
        now_ms,
    ) {
        return IotOnlineStatus::DataAnomaly;
    }
    IotOnlineStatus::Online
}

fn is_expired(timestamp: i64, threshold_secs: i64, now_ms: i64) -> bool {
    let threshold_ms = threshold_secs.max(1).saturating_mul(1_000);
    timestamp <= 0 || now_ms.saturating_sub(timestamp) > threshold_ms
}

fn validate_device_request(request: &CreateIotDeviceRequest) -> Result<()> {
    for (field, value) in [
        ("deviceCode", request.device_code.as_str()),
        ("name", request.name.as_str()),
        ("productCode", request.product_code.as_str()),
        ("productName", request.product_name.as_str()),
        ("mqttClientId", request.mqtt_client_id.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("invalid device request: {field} 不能为空");
        }
    }
    if request.expected_heartbeat_secs <= 0 || request.expected_data_secs <= 0 {
        bail!("invalid device request: 心跳和数据周期必须大于 0");
    }
    Ok(())
}

fn device_view(record_id: String, payload: &Value, now_ms: i64) -> IotDeviceView {
    let connected = boolean(payload, "connected");
    let last_seen_at_ms = integer(payload, "last_seen_at_ms");
    let last_heartbeat_at_ms = integer(payload, "last_heartbeat_at_ms");
    let last_data_at_ms = integer(payload, "last_data_at_ms");
    let status = evaluate_online_status(
        connected,
        last_seen_at_ms,
        last_heartbeat_at_ms,
        last_data_at_ms,
        integer(payload, "expected_heartbeat_secs"),
        integer(payload, "expected_data_secs"),
        now_ms,
    );
    IotDeviceView {
        record_id,
        device_code: text(payload, "device_code"),
        name: text(payload, "name"),
        product_code: text(payload, "product_code"),
        product_name: text(payload, "product_name"),
        gateway_code: text(payload, "gateway_code"),
        mqtt_client_id: text(payload, "mqtt_client_id"),
        location: text(payload, "location"),
        enabled: boolean(payload, "enabled"),
        connected,
        status,
        last_seen_at_ms,
        last_heartbeat_at_ms,
        last_data_at_ms,
        offline_reason: text(payload, "offline_reason"),
    }
}

fn summarize_devices(devices: &[IotDeviceView]) -> IotStatusSummary {
    let mut summary = IotStatusSummary {
        total: devices.len(),
        enabled: devices.iter().filter(|device| device.enabled).count(),
        ..IotStatusSummary::default()
    };
    for device in devices {
        match device.status {
            IotOnlineStatus::Online => summary.online += 1,
            IotOnlineStatus::HeartbeatLost => summary.heartbeat_lost += 1,
            IotOnlineStatus::DataAnomaly => summary.data_anomaly += 1,
            IotOnlineStatus::Offline => summary.offline += 1,
            IotOnlineStatus::Unknown => summary.unknown += 1,
        }
    }
    summary
}

fn product_view(payload: &Value) -> IotProductView {
    IotProductView {
        code: text(payload, "code"),
        name: text(payload, "name"),
        category: text(payload, "category"),
        protocol: text(payload, "protocol"),
        enabled: boolean(payload, "enabled"),
    }
}

fn gateway_view(payload: &Value) -> IotGatewayView {
    IotGatewayView {
        code: text(payload, "code"),
        name: text(payload, "name"),
        mqtt_client_id: text(payload, "mqtt_client_id"),
        connected: boolean(payload, "connected"),
        location: text(payload, "location"),
        last_heartbeat_at_ms: integer(payload, "last_heartbeat_at_ms"),
    }
}

fn telemetry_view(payload: &Value) -> IotTelemetryView {
    IotTelemetryView {
        device_code: text(payload, "device_code"),
        metric_code: text(payload, "metric_code"),
        value: number(payload, "value"),
        unit: text(payload, "unit"),
        quality: text(payload, "quality"),
        collected_at_ms: integer(payload, "collected_at_ms"),
    }
}

fn alarm_view(payload: &Value) -> IotAlarmView {
    IotAlarmView {
        device_code: text(payload, "device_code"),
        level: text(payload, "level"),
        message: text(payload, "message"),
        status: text(payload, "status"),
        occurred_at_ms: integer(payload, "occurred_at_ms"),
    }
}

fn text(payload: &Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn integer(payload: &Value, key: &str) -> i64 {
    payload.get(key).and_then(Value::as_i64).unwrap_or_default()
}

fn number(payload: &Value, key: &str) -> f64 {
    payload.get(key).and_then(Value::as_f64).unwrap_or_default()
}

fn boolean(payload: &Value, key: &str) -> bool {
    payload.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn timestamp_ms() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as i64,
        Err(_) => 0,
    }
}

fn demo_datasets(now: i64) -> Vec<(&'static str, Vec<Value>)> {
    vec![
        (
            PRODUCT_MODEL,
            vec![
                json!({"code":"temperature_sensor","name":"温湿度传感器","category":"环境监测","protocol":"MQTT","enabled":true}),
                json!({"code":"energy_meter","name":"智能电表","category":"能源管理","protocol":"Modbus TCP","enabled":true}),
                json!({"code":"edge_controller","name":"边缘控制器","category":"边缘计算","protocol":"OPC UA","enabled":true}),
            ],
        ),
        (
            GATEWAY_MODEL,
            vec![
                json!({"code":"gw-shanghai-01","name":"上海车间网关","mqtt_client_id":"factory-shanghai-gw-01","connected":true,"last_seen_at_ms":now-8_000,"last_heartbeat_at_ms":now-8_000,"expected_heartbeat_secs":30,"location":"上海一号车间","enabled":true}),
                json!({"code":"gw-tianjin-02","name":"天津产线网关","mqtt_client_id":"factory-tianjin-gw-02","connected":false,"last_seen_at_ms":now-900_000,"last_heartbeat_at_ms":now-900_000,"expected_heartbeat_secs":30,"location":"天津二号产线","enabled":true}),
            ],
        ),
        (
            DEVICE_MODEL,
            vec![
                demo_device(
                    "TH-001",
                    "一号车间温湿度",
                    "temperature_sensor",
                    "温湿度传感器",
                    "gw-shanghai-01",
                    "factory-shanghai-th-001",
                    "上海一号车间",
                    true,
                    now - 5_000,
                    now - 5_000,
                    now - 10_000,
                    300,
                    600,
                    "",
                ),
                demo_device(
                    "TH-002",
                    "仓储区温湿度",
                    "temperature_sensor",
                    "温湿度传感器",
                    "gw-shanghai-01",
                    "factory-shanghai-th-002",
                    "上海仓储区",
                    true,
                    now - 20_000,
                    now - 1_200_000,
                    now - 20_000,
                    300,
                    600,
                    "心跳超时",
                ),
                demo_device(
                    "EM-101",
                    "一号产线智能电表",
                    "energy_meter",
                    "智能电表",
                    "gw-shanghai-01",
                    "factory-shanghai-em-101",
                    "上海一号产线",
                    true,
                    now - 15_000,
                    now - 15_000,
                    now - 1_500_000,
                    300,
                    600,
                    "数据采集超时",
                ),
                demo_device(
                    "EM-102",
                    "二号产线智能电表",
                    "energy_meter",
                    "智能电表",
                    "gw-tianjin-02",
                    "factory-tianjin-em-102",
                    "天津二号产线",
                    false,
                    now - 900_000,
                    now - 900_000,
                    now - 900_000,
                    300,
                    600,
                    "MQTT Last Will 离线",
                ),
                demo_device(
                    "EC-201",
                    "包装线边缘控制器",
                    "edge_controller",
                    "边缘控制器",
                    "gw-shanghai-01",
                    "factory-shanghai-ec-201",
                    "上海包装线",
                    true,
                    now - 6_000,
                    now - 6_000,
                    now - 6_000,
                    300,
                    600,
                    "",
                ),
                json!({"device_code":"NEW-301","name":"待接入压力传感器","product_code":"temperature_sensor","product_name":"温湿度传感器","gateway_code":"","mqtt_client_id":"factory-unbound-new-301","location":"待分配","enabled":false,"connected":false,"last_seen_at_ms":0,"last_heartbeat_at_ms":0,"last_data_at_ms":0,"expected_heartbeat_secs":30,"expected_data_secs":60,"offline_reason":"尚未接入"}),
            ],
        ),
        (
            TELEMETRY_MODEL,
            vec![
                json!({"device_code":"TH-001","metric_code":"temperature","value":23.6,"unit":"℃","quality":"good","collected_at_ms":now-10_000}),
                json!({"device_code":"TH-001","metric_code":"humidity","value":48.2,"unit":"%RH","quality":"good","collected_at_ms":now-10_000}),
                json!({"device_code":"EM-101","metric_code":"active_power","value":18.7,"unit":"kW","quality":"stale","collected_at_ms":now-300_000}),
            ],
        ),
        (
            ALARM_MODEL,
            vec![
                json!({"device_code":"TH-002","level":"warning","message":"心跳超过 3 个周期未刷新","status":"open","occurred_at_ms":now-180_000}),
                json!({"device_code":"EM-101","level":"major","message":"业务遥测超过 2 个采集周期未刷新","status":"open","occurred_at_ms":now-240_000}),
            ],
        ),
    ]
}

/// 集中构造演示设备，参数逐项对应低代码设备模型字段。
#[allow(clippy::too_many_arguments)]
fn demo_device(
    device_code: &str,
    name: &str,
    product_code: &str,
    product_name: &str,
    gateway_code: &str,
    mqtt_client_id: &str,
    location: &str,
    connected: bool,
    last_seen_at_ms: i64,
    last_heartbeat_at_ms: i64,
    last_data_at_ms: i64,
    expected_heartbeat_secs: i64,
    expected_data_secs: i64,
    offline_reason: &str,
) -> Value {
    json!({
        "device_code":device_code,
        "name":name,
        "product_code":product_code,
        "product_name":product_name,
        "gateway_code":gateway_code,
        "mqtt_client_id":mqtt_client_id,
        "location":location,
        "enabled":true,
        "connected":connected,
        "last_seen_at_ms":last_seen_at_ms,
        "last_heartbeat_at_ms":last_heartbeat_at_ms,
        "last_data_at_ms":last_data_at_ms,
        "expected_heartbeat_secs":expected_heartbeat_secs,
        "expected_data_secs":expected_data_secs,
        "offline_reason":offline_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_false_online_states_separately() {
        let now = 1_000_000;

        // 关键断言：连接仍在但心跳过期时必须识别为假在线，而不是绿色在线。
        assert_eq!(
            evaluate_online_status(true, now - 1_000, now - 100_000, now - 1_000, 30, 60, now),
            IotOnlineStatus::HeartbeatLost
        );
        // 关键断言：心跳正常但业务数据过期时必须识别采集异常。
        assert_eq!(
            evaluate_online_status(true, now - 1_000, now - 1_000, now - 130_000, 30, 60, now),
            IotOnlineStatus::DataAnomaly
        );
        assert_eq!(
            evaluate_online_status(false, now - 1_000, now - 1_000, now - 1_000, 30, 60, now),
            IotOnlineStatus::Offline
        );
        assert_eq!(
            evaluate_online_status(false, 0, 0, 0, 30, 60, now),
            IotOnlineStatus::Unknown
        );
    }

    #[test]
    fn template_contains_five_lowcode_models() {
        let names = MODEL_DEFINITIONS
            .iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>();

        // 关键断言：物联网插件的正式业务模型必须全部进入低代码引擎。
        assert_eq!(
            names,
            [
                PRODUCT_MODEL,
                GATEWAY_MODEL,
                DEVICE_MODEL,
                TELEMETRY_MODEL,
                ALARM_MODEL
            ]
        );
    }
}
