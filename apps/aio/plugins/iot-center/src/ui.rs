#![allow(non_snake_case)]

//! 物联网中心 SSR 页面。

use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;
use registry::ui::{
    badge::Badge,
    button::Button,
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
};

use crate::{
    contract::{
        ALARM_MODEL, DEVICE_MODEL, GATEWAY_MODEL, IotDashboardSnapshot, IotDeviceView,
        IotOnlineStatus, PRODUCT_MODEL, TELEMETRY_MODEL, UI_ACTION_PATH,
    },
    state::{run_iot_future, service},
};

struct PageSnapshot {
    dashboard: IotDashboardSnapshot,
    error: Option<String>,
}

/// 渲染物联网设备、产品、网关、遥测和告警工作台。
pub fn IotCenterPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let view = query_value(&context.active_route, "view").unwrap_or_else(|| "devices".to_string());
    let message = query_value(&context.active_route, "message");
    let route_error = query_value(&context.active_route, "error");

    rsx! {
        div { class: "space-y-5",
            Card {
                CardHeader {
                    div { class: "flex flex-wrap items-start justify-between gap-4",
                        div {
                            CardTitle { "物联网设备中心" }
                            CardDescription { "低代码模型驱动 · PostgreSQL 持久化 · 连接、心跳、数据三维在线判定" }
                        }
                        div { class: "flex flex-wrap gap-2",
                            Badge { "EngineStore" }
                            Badge { "MQTT Ready" }
                            Badge { "All in PG" }
                        }
                    }
                }
            }

            if let Some(message) = message {
                div { class: "rounded-xl border border-green-600/30 bg-green-600/10 p-3 text-sm text-green-800", "{message}" }
            }
            if let Some(error) = route_error.or(snapshot.error.clone()) {
                div { class: "rounded-xl border border-destructive bg-destructive/10 p-3 text-sm text-destructive", "{error}" }
            }

            if !snapshot.dashboard.template_ready {
                {render_template_empty()}
            } else {
                {render_summary(&snapshot.dashboard)}
                match view.as_str() {
                    "products" => render_products(&snapshot.dashboard),
                    "gateways" => render_gateways(&snapshot.dashboard),
                    "telemetry" => render_telemetry(&snapshot.dashboard),
                    "alarms" => render_alarms(&snapshot.dashboard),
                    _ => render_devices(&snapshot.dashboard),
                }
            }
        }
    }
}

fn load_snapshot() -> PageSnapshot {
    match service().and_then(|service| run_iot_future(async move { service.dashboard().await })) {
        Ok(dashboard) => PageSnapshot {
            dashboard,
            error: None,
        },
        Err(error) => PageSnapshot {
            dashboard: IotDashboardSnapshot::default(),
            error: Some(error.to_string()),
        },
    }
}

fn render_template_empty() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "初始化物联网低代码模板" }
                CardDescription { "将在共享 PostgreSQL 中创建产品、网关、设备、遥测、告警五个 EngineStore 动态模型。" }
            }
            CardContent {
                form { method: "post", action: UI_ACTION_PATH, class: "flex flex-wrap items-center gap-4",
                    input { r#type: "hidden", name: "action", value: "apply_template" }
                    label { class: "flex items-center gap-2 text-sm",
                        input { r#type: "checkbox", name: "seed_demo", value: "true", checked: true }
                        span { "写入演示产品、网关、设备和告警" }
                    }
                    Button { button_type: "submit", "初始化物联网系统" }
                }
            }
        }
    }
}

fn render_summary(snapshot: &IotDashboardSnapshot) -> Element {
    let summary = &snapshot.summary;
    rsx! {
        div { class: "grid gap-3 sm:grid-cols-2 xl:grid-cols-6",
            SummaryCard { label: "设备总数", value: summary.total.to_string(), tone: "slate" }
            SummaryCard { label: "已启用", value: summary.enabled.to_string(), tone: "blue" }
            SummaryCard { label: "在线", value: summary.online.to_string(), tone: "green" }
            SummaryCard { label: "心跳丢失", value: summary.heartbeat_lost.to_string(), tone: "yellow" }
            SummaryCard { label: "数据异常", value: summary.data_anomaly.to_string(), tone: "orange" }
            SummaryCard { label: "离线 / 未知", value: format!("{} / {}", summary.offline, summary.unknown), tone: "red" }
        }
    }
}

#[component]
fn SummaryCard(label: &'static str, value: String, tone: &'static str) -> Element {
    let class = match tone {
        "green" => "border-green-500/30 bg-green-500/10",
        "yellow" => "border-yellow-500/30 bg-yellow-500/10",
        "orange" => "border-orange-500/30 bg-orange-500/10",
        "red" => "border-red-500/30 bg-red-500/10",
        "blue" => "border-blue-500/30 bg-blue-500/10",
        _ => "border-slate-300 bg-white",
    };
    rsx! {
        div { class: "rounded-xl border p-4 {class}",
            div { class: "text-xs text-muted-foreground", "{label}" }
            div { class: "mt-1 text-2xl font-semibold", "{value}" }
        }
    }
}

fn render_devices(snapshot: &IotDashboardSnapshot) -> Element {
    rsx! {
        div { class: "space-y-4",
            Card {
                CardHeader {
                    CardTitle { "新建设备" }
                    CardDescription { "平台生成并校验唯一 MQTT ClientId；新设备初始状态为 Unknown。" }
                }
                CardContent {
                    form { method: "post", action: UI_ACTION_PATH, class: "grid gap-3 md:grid-cols-3 xl:grid-cols-4",
                        input { r#type: "hidden", name: "action", value: "create_device" }
                        TextField { label: "设备编码", name: "device_code", placeholder: "TH-003", required: true }
                        TextField { label: "设备名称", name: "name", placeholder: "三号温湿度", required: true }
                        TextField { label: "产品编码", name: "product_code", placeholder: "temperature_sensor", required: true }
                        TextField { label: "产品名称", name: "product_name", placeholder: "温湿度传感器", required: true }
                        TextField { label: "网关编码", name: "gateway_code", placeholder: "gw-shanghai-01" }
                        TextField { label: "MQTT ClientId", name: "mqtt_client_id", placeholder: "factory-shanghai-th-003", required: true }
                        TextField { label: "安装位置", name: "location", placeholder: "上海一号车间" }
                        div { class: "flex items-end gap-3",
                            label { class: "flex items-center gap-2 pb-2 text-sm",
                                input { r#type: "checkbox", name: "enabled", value: "true", checked: true }
                                span { "启用" }
                            }
                            Button { button_type: "submit", "新建设备" }
                        }
                    }
                }
            }

            div { class: "grid gap-4 lg:grid-cols-2 2xl:grid-cols-3",
                for device in &snapshot.devices {
                    {device_card(device)}
                }
            }
        }
    }
}

fn device_card(device: &IotDeviceView) -> Element {
    let status_class = status_badge_class(device.status);
    let dot_class = status_dot_class(device.status);
    let model_href = lowcode_records_href(DEVICE_MODEL);
    rsx! {
        Card { class: "overflow-hidden",
            CardHeader {
                div { class: "flex items-start justify-between gap-3",
                    div { class: "min-w-0",
                        div { class: "flex items-center gap-2",
                            span { class: "h-2.5 w-2.5 shrink-0 rounded-full {dot_class}" }
                            CardTitle { class: "truncate", "{device.name}" }
                        }
                        CardDescription { "{device.device_code} · {device.location}" }
                    }
                    span { class: "shrink-0 rounded-full px-2 py-1 text-xs font-medium {status_class}", "{device.status.label()}" }
                }
            }
            CardContent { class: "space-y-3 text-sm",
                div { class: "grid grid-cols-2 gap-3",
                    InfoItem { label: "所属产品", value: device.product_name.clone() }
                    InfoItem { label: "接入网关", value: fallback(&device.gateway_code) }
                    InfoItem { label: "MQTT 连接", value: if device.connected { "connected".to_string() } else { "disconnected".to_string() } }
                    InfoItem { label: "启用状态", value: if device.enabled { "enabled".to_string() } else { "disabled".to_string() } }
                    InfoItem { label: "最后心跳", value: age_text(device.last_heartbeat_at_ms) }
                    InfoItem { label: "最后数据", value: age_text(device.last_data_at_ms) }
                }
                if !device.offline_reason.is_empty() {
                    div { class: "rounded-md bg-muted px-3 py-2 text-xs text-muted-foreground", "原因：{device.offline_reason}" }
                }
                div { class: "flex items-center justify-between border-t pt-3",
                    code { class: "text-xs text-muted-foreground", "{device.mqtt_client_id}" }
                    a { class: "text-sm font-medium text-primary hover:underline", href: model_href, "低代码记录 →" }
                }
            }
        }
    }
}

fn render_products(snapshot: &IotDashboardSnapshot) -> Element {
    let model_href = lowcode_records_href(PRODUCT_MODEL);
    rsx! {
        div { class: "space-y-4",
            SectionHeader { title: "产品模板", description: "协议和设备分类由低代码产品模型管理。", model_href }
            div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-3",
                for product in &snapshot.products {
                    Card {
                        CardHeader { CardTitle { "{product.name}" } CardDescription { "{product.code}" } }
                        CardContent { class: "flex flex-wrap gap-2",
                            Badge { "{product.category}" }
                            Badge { "{product.protocol}" }
                            Badge { if product.enabled { "enabled" } else { "disabled" } }
                        }
                    }
                }
            }
        }
    }
}

fn render_gateways(snapshot: &IotDashboardSnapshot) -> Element {
    let model_href = lowcode_records_href(GATEWAY_MODEL);
    rsx! {
        div { class: "space-y-4",
            SectionHeader { title: "边缘网关", description: "网关连接和心跳与下挂设备健康分别管理。", model_href }
            div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-3",
                for gateway in &snapshot.gateways {
                    Card {
                        CardHeader {
                            CardTitle { "{gateway.name}" }
                            CardDescription { "{gateway.code} · {gateway.location}" }
                        }
                        CardContent { class: "space-y-2 text-sm",
                            InfoItem { label: "连接", value: if gateway.connected { "connected".to_string() } else { "disconnected".to_string() } }
                            InfoItem { label: "最后心跳", value: age_text(gateway.last_heartbeat_at_ms) }
                            code { class: "text-xs text-muted-foreground", "{gateway.mqtt_client_id}" }
                        }
                    }
                }
            }
        }
    }
}

fn render_telemetry(snapshot: &IotDashboardSnapshot) -> Element {
    let model_href = lowcode_records_href(TELEMETRY_MODEL);
    rsx! {
        div { class: "space-y-4",
            SectionHeader { title: "数据采集", description: "只有通过解析和质量校验的业务数据才刷新数据新鲜度。", model_href }
            Table {
                TableHeader { TableRow { TableHead { "设备" } TableHead { "指标" } TableHead { "值" } TableHead { "质量" } TableHead { "采集时间" } } }
                TableBody {
                    for item in &snapshot.telemetry {
                        TableRow {
                            TableCell { code { "{item.device_code}" } }
                            TableCell { "{item.metric_code}" }
                            TableCell { "{item.value} {item.unit}" }
                            TableCell { Badge { "{item.quality}" } }
                            TableCell { "{age_text(item.collected_at_ms)}" }
                        }
                    }
                }
            }
        }
    }
}

fn render_alarms(snapshot: &IotDashboardSnapshot) -> Element {
    let model_href = lowcode_records_href(ALARM_MODEL);
    rsx! {
        div { class: "space-y-4",
            SectionHeader { title: "告警中心", description: "心跳丢失和数据异常独立告警，保留可解释原因。", model_href }
            Table {
                TableHeader { TableRow { TableHead { "设备" } TableHead { "等级" } TableHead { "告警" } TableHead { "状态" } TableHead { "发生时间" } } }
                TableBody {
                    for alarm in &snapshot.alarms {
                        TableRow {
                            TableCell { code { "{alarm.device_code}" } }
                            TableCell { Badge { "{alarm.level}" } }
                            TableCell { "{alarm.message}" }
                            TableCell { "{alarm.status}" }
                            TableCell { "{age_text(alarm.occurred_at_ms)}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SectionHeader(title: &'static str, description: &'static str, model_href: String) -> Element {
    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-start justify-between gap-4",
                    div { CardTitle { "{title}" } CardDescription { "{description}" } }
                    a { class: "text-sm font-medium text-primary hover:underline", href: model_href, "打开低代码记录 →" }
                }
            }
        }
    }
}

#[component]
fn TextField(
    label: &'static str,
    name: &'static str,
    placeholder: &'static str,
    #[props(default)] required: bool,
) -> Element {
    rsx! {
        label { class: "grid gap-1 text-sm",
            span { class: "font-medium", "{label}" }
            input { class: "az-input", name, placeholder, required }
        }
    }
}

#[component]
fn InfoItem(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "min-w-0",
            div { class: "text-xs text-muted-foreground", "{label}" }
            div { class: "truncate font-medium", "{value}" }
        }
    }
}

fn status_badge_class(status: IotOnlineStatus) -> &'static str {
    match status {
        IotOnlineStatus::Online => "bg-green-500/15 text-green-700",
        IotOnlineStatus::HeartbeatLost => "bg-yellow-500/15 text-yellow-800",
        IotOnlineStatus::DataAnomaly => "bg-orange-500/15 text-orange-800",
        IotOnlineStatus::Offline => "bg-red-500/15 text-red-700",
        IotOnlineStatus::Unknown => "bg-slate-500/15 text-slate-700",
    }
}

fn status_dot_class(status: IotOnlineStatus) -> &'static str {
    match status {
        IotOnlineStatus::Online => "bg-green-500",
        IotOnlineStatus::HeartbeatLost => "bg-yellow-500",
        IotOnlineStatus::DataAnomaly => "bg-orange-500",
        IotOnlineStatus::Offline => "bg-red-500",
        IotOnlineStatus::Unknown => "bg-slate-400",
    }
}

fn age_text(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "从未".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default();
    let seconds = now.saturating_sub(timestamp_ms) / 1_000;
    match seconds {
        0..=59 => format!("{seconds} 秒前"),
        60..=3_599 => format!("{} 分钟前", seconds / 60),
        3_600..=86_399 => format!("{} 小时前", seconds / 3_600),
        _ => format!("{} 天前", seconds / 86_400),
    }
}

fn fallback(value: &str) -> String {
    if value.is_empty() {
        "未绑定".to_string()
    } else {
        value.to_string()
    }
}

fn lowcode_records_href(model_name: &str) -> String {
    let route = format!("/lowcode?model={model_name}&tab=records");
    format!("/?route={}", urlencoding::encode(&route))
}

fn query_value(route: &str, key: &str) -> Option<String> {
    let query = route.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (pair_key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if pair_key != key {
            return None;
        }
        Some(
            urlencoding::decode(value)
                .map(|value| value.into_owned())
                .unwrap_or_else(|_| value.to_string()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowcode_link_opens_dynamic_device_records() {
        let href = lowcode_records_href(DEVICE_MODEL);

        // 关键断言：物联网插件必须回到真实低代码模型记录页，而不是维护第二套 CRUD。
        assert_eq!(
            href,
            "/?route=%2Flowcode%3Fmodel%3Diot_device%26tab%3Drecords"
        );
    }
}
