#![cfg(any())]
use az_aio_platform::plugin::api::NativeRenderContext;
use adui_dioxus::Card;
use dioxus::prelude::*;

use crate::{
    backend::model::GatewayRouteSummary,
    ui::state::{EdgeGatewayPageSnapshot, load_snapshot},
};

const MAX_RECENT_USAGE: usize = 6;

#[allow(non_snake_case)]
pub fn EdgeGatewayPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let selected_id = query_value(&context.active_route, "routeId");
    let selected_route = selected_route(&snapshot, selected_id.as_deref());
    let routes_url = api_url(&context.api_base_url, "/api/edge-gateway/routes");
    let assets_url = api_url(&context.api_base_url, "/api/edge-gateway/assets");
    let usage_url = api_url(&context.api_base_url, "/api/edge-gateway/assets/usage");
    let route_action_url = api_url(&context.api_base_url, "/api/edge-gateway/ui-route");
    let status_url = api_url(&context.api_base_url, "/api/edge-gateway/status");
    let weather_curl = weather_curl_preview();
    let saved = context.active_route.contains("saved=route");
    let error = query_value(&context.active_route, "error");

    rsx! {
        div { class: "adui-space adui-space-vertical", style: "display:grid;gap:22px;",
            
            Card {
                span { class: "adui-avatar", "EDGE" }
                div {
                    p { class: "adui-typography-secondary", "Edge API Studio · Toasty PG" }
                    h1 { "接口、路由与脚本资产管理台" }
                    p { "在线管理 GET/POST 路由、Bearer token 资产、脚本草稿和调用观测；定义持久化到 edge-gateway Toasty PostgreSQL。" }
                }
                div { class: "adui-space", style: "display:flex;flex-wrap:wrap;gap:8px;",
                    a { class: "adui-btn adui-btn-solid adui-btn-primary", href: "#route-editor", "新建/保存路由" }
                    a { class: "adui-btn adui-btn-outlined adui-btn-default", href: routes_url.clone(), "JSON API" }
                }
            }

            if saved {
                div { class: "adui-alert adui-alert-success", "路由定义已保存到 Toasty PG。" }
            }
            if let Some(error) = error {
                div { class: "adui-alert adui-alert-error", "保存失败：{error}" }
            }
            if let Some(error) = &snapshot.error {
                div { class: "adui-alert adui-alert-error", "运行告警：{error}" }
            }

            div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:16px;",
                MetricCard { label: "PG Store", value: connected_text(snapshot.status.store_connected), detail: snapshot.status.table_prefix.clone() }
                MetricCard { label: "Managed Routes", value: snapshot.route_definitions.len().to_string(), detail: "GET / POST definitions".to_string() }
                MetricCard { label: "Callable Assets", value: snapshot.callable_assets.len().to_string(), detail: "Bearer token gated".to_string() }
                MetricCard { label: "Usage Events", value: snapshot.usage_records.len().to_string(), detail: "recent asset calls".to_string() }
            }

            div { class: "adui-row", style: "display:grid;grid-template-columns:minmax(240px,300px) minmax(0,1fr) minmax(260px,340px);gap:16px;align-items:start;",
                aside { class: "adui-space adui-space-vertical", style: "display:grid;gap:16px;",
                    Card {
                        span { "01" }
                        div {
                            h2 { "路由库" }
                            p { "在线接口定义" }
                        }
                    }
                    a { class: route_link_class(selected_id.is_none()), href: "/?route=/gateway", "+ 新路由草稿" }
                    if snapshot.route_definitions.is_empty() {
                        Card {
                            strong { "还没有自定义路由" }
                            p { "先保存右侧表单，路由定义会进入 Toasty PG。" }
                        }
                    } else {
                        nav { class: "adui-menu adui-menu-inline",
                            for route in &snapshot.route_definitions {
                                a { class: route_link_class(selected_id.as_deref() == Some(route.id.as_str())), href: format!("/?route=/gateway&routeId={}", route.id),
                                    span { class: method_class(&route.method), "{route.method}" }
                                    strong { "{route.name}" }
                                    code { "{route.route}" }
                                }
                            }
                        }
                    }

                    Card {
                        span { "02" }
                        div {
                            h2 { "内置资产" }
                            p { "可被外部调用" }
                        }
                    }
                    div { class: "adui-space adui-space-vertical", style: "display:grid;gap:10px;",
                        for asset in &snapshot.callable_assets {
                            Card {
                                strong { "{asset.name}" }
                                code { "{asset.method} {asset.route}" }
                                p { "provider={asset.provider}" }
                            }
                        }
                    }
                }

                main { class: "adui-space adui-space-vertical", style: "display:grid;gap:16px;", id: "route-editor",
                    Card {
                        div {
                            p { class: "adui-typography-secondary", "Route Contract" }
                            h2 { "{editor_title(&selected_route)}" }
                            p { "将接口路径、GET/POST 方法、认证要求和脚本代码作为资产保存。当前版本保存脚本草稿，不直接执行任意代码。" }
                        }
                        div { class: "adui-space", style: "display:flex;flex-wrap:wrap;gap:8px;",
                            span { class: method_class(&selected_route.method), "{selected_route.method}" }
                            span { class: status_class(&selected_route.status), "{selected_route.status}" }
                            span { "auth={auth_text(selected_route.auth_required)}" }
                        }
                    }

                    form { class: "adui-form", method: "post", action: route_action_url,
                        input { r#type: "hidden", name: "id", value: "{selected_route.id}" }
                        div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px;",
                            label { class: "adui-form-item",
                                span { "接口名称" }
                                input { class: "adui-input", name: "name", value: "{selected_route.name}", placeholder: "订单查询 / Weather Proxy" }
                            }
                            label { class: "adui-form-item",
                                span { "路由路径" }
                                input { class: "adui-input", name: "route", value: "{selected_route.route}", placeholder: "/api/edge/orders/query" }
                            }
                            label { class: "adui-form-item",
                                span { "请求方法" }
                                select { class: "adui-select", name: "method",
                                    option { value: "GET", selected: selected_route.method == "GET", "GET" }
                                    option { value: "POST", selected: selected_route.method == "POST", "POST" }
                                }
                            }
                            label { class: "adui-form-item",
                                span { "状态" }
                                select { class: "adui-select", name: "status",
                                    option { value: "draft", selected: selected_route.status == "draft", "draft" }
                                    option { value: "active", selected: selected_route.status == "active", "active" }
                                    option { value: "disabled", selected: selected_route.status == "disabled", "disabled" }
                                }
                            }
                            label { class: "adui-form-item",
                                span { "认证" }
                                select { class: "adui-select", name: "auth_required",
                                    option { value: "true", selected: selected_route.auth_required, "Bearer token required" }
                                    option { value: "false", selected: !selected_route.auth_required, "Public / no auth" }
                                }
                            }
                            label { class: "adui-form-item",
                                span { "脚本语言" }
                                input { class: "adui-input", name: "script_language", value: "{selected_route.script_language}", placeholder: "javascript / json-template / wasm" }
                            }
                        }

                        div { class: "adui-row", style: "display:grid;grid-template-columns:minmax(0,1fr) minmax(260px,360px);gap:12px;",
                            label { class: "adui-form-item",
                                span { "在线脚本代码" }
                                textarea { class: "adui-input", name: "script_code", spellcheck: "false", "{selected_route.script_code}" }
                            }
                            div { class: "adui-space adui-space-vertical", style: "display:grid;gap:12px;",
                                label { class: "adui-form-item",
                                    span { "请求示例 JSON" }
                                    textarea { class: "adui-input", name: "request_example", spellcheck: "false", "{selected_route.request_example}" }
                                }
                                label { class: "adui-form-item",
                                    span { "响应模板 JSON" }
                                    textarea { class: "adui-input", name: "response_template", spellcheck: "false", "{selected_route.response_template}" }
                                }
                                label { class: "adui-form-item",
                                    span { "备注" }
                                    textarea { class: "adui-input", name: "notes", "{selected_route.notes}" }
                                }
                            }
                        }

                        div { class: "adui-space", style: "display:flex;flex-wrap:wrap;gap:8px;",
                            button { class: "adui-btn adui-btn-solid adui-btn-primary", r#type: "submit", "保存到 Toasty" }
                            a { class: "adui-btn adui-btn-outlined adui-btn-default", href: status_url, "运行态状态" }
                            a { class: "adui-btn adui-btn-outlined adui-btn-default", href: assets_url, "资产目录" }
                        }
                    }
                }

                aside { class: "adui-space adui-space-vertical", style: "display:grid;gap:16px;",
                    Card {
                        span { "03" }
                        div {
                            h2 { "调用调试" }
                            p { "curl / usage" }
                        }
                    }
                    Card {
                        h3 { "当前路由 curl" }
                        pre { "{curl_preview(&selected_route)}" }
                    }
                    Card {
                        h3 { "天气资产快速测试" }
                        pre { "{weather_curl}" }
                    }
                    Card {
                        div { class: "adui-space", style: "display:flex;align-items:center;justify-content:space-between;gap:8px;",
                            h3 { "最近调用" }
                            a { href: usage_url, "usage json" }
                        }
                        if snapshot.usage_records.is_empty() {
                            p { class: "adui-empty-description", "还没有调用流水。" }
                        } else {
                            for record in snapshot.usage_records.iter().rev().take(MAX_RECENT_USAGE) {
                                div { class: "adui-space", style: "display:flex;align-items:center;gap:8px;",
                                    span { class: status_code_class(record.status_code), "{record.status_code}" }
                                    code { "{record.asset_id}" }
                                    small { "{record.duration_ms}ms" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct EditorRoute {
    id: String,
    route: String,
    method: String,
    name: String,
    status: String,
    auth_required: bool,
    script_language: String,
    script_code: String,
    request_example: String,
    response_template: String,
    notes: String,
}

#[component]
fn MetricCard(label: String, value: String, detail: String) -> Element {
    rsx! {
        Card {
            span { "{label}" }
            strong { "{value}" }
            code { "{detail}" }
        }
    }
}

fn selected_route(snapshot: &EdgeGatewayPageSnapshot, selected_id: Option<&str>) -> EditorRoute {
    selected_id
        .and_then(|id| snapshot.route_definitions.iter().find(|route| route.id == id))
        .or_else(|| snapshot.route_definitions.first())
        .map(editor_route_from_summary)
        .unwrap_or_else(blank_editor_route)
}

fn editor_route_from_summary(route: &GatewayRouteSummary) -> EditorRoute {
    EditorRoute {
        id: route.id.clone(),
        route: route.route.clone(),
        method: route.method.clone(),
        name: route.name.clone(),
        status: route.status.clone(),
        auth_required: route.auth_required,
        script_language: route.script_language.clone(),
        script_code: route.script_code.clone(),
        request_example: route.request_example.clone(),
        response_template: route.response_template.clone(),
        notes: route.notes.clone(),
    }
}

fn blank_editor_route() -> EditorRoute {
    EditorRoute {
        id: String::new(),
        route: "/api/edge-gateway/custom/hello".to_string(),
        method: "POST".to_string(),
        name: "Hello Edge API".to_string(),
        status: "draft".to_string(),
        auth_required: true,
        script_language: "javascript".to_string(),
        script_code: "export default async function handle(request) {\n  return { ok: true, input: request.body };\n}".to_string(),
        request_example: r#"{"name":"az-aio"}"#.to_string(),
        response_template: r#"{"ok":true,"input":"object"}"#.to_string(),
        notes: "Script is stored as an edge asset draft. Execution requires a sandbox runner phase.".to_string(),
    }
}

fn editor_title(route: &EditorRoute) -> &str {
    if route.id.is_empty() {
        "新建接口资产"
    } else {
        &route.name
    }
}

fn curl_preview(route: &EditorRoute) -> String {
    let token = if route.auth_required {
        " \\\n  -H 'Authorization: Bearer edge-demo-weather-token'"
    } else {
        ""
    };
    let body = if route.method == "POST" {
        format!(" \\\n  -H 'Content-Type: application/json' \\\n  -d '{}'", route.request_example.replace('\n', ""))
    } else {
        String::new()
    };
    format!(
        "curl -X {} http://127.0.0.1:18081{}{}{}",
        route.method, route.route, token, body
    )
}

fn weather_curl_preview() -> String {
    "curl -X POST http://127.0.0.1:18081/api/edge-gateway/assets/weather/current \\\n  -H 'Authorization: Bearer edge-demo-weather-token' \\\n  -H 'Content-Type: application/json' \\\n  -d '{\"latitude\":31.2304,\"longitude\":121.4737,\"timezone\":\"Asia/Shanghai\"}'"
        .to_string()
}

fn query_value(route: &str, key: &str) -> Option<String> {
    let query = route.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (pair_key, value) = pair.split_once('=')?;
        (pair_key == key).then(|| value.to_string())
    })
}

fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}{path}")
    }
}

fn connected_text(value: bool) -> &'static str {
    if value { "已连接" } else { "未连接" }
}

fn auth_text(value: bool) -> &'static str {
    if value { "required" } else { "public" }
}

fn route_link_class(active: bool) -> &'static str {
    if active {
        "adui-menu-item adui-menu-item-selected"
    } else {
        "adui-menu-item"
    }
}

fn method_class(method: &str) -> &'static str {
    match method {
        "GET" => "adui-tag",
        "POST" => "adui-tag",
        _ => "adui-tag",
    }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "active" => "adui-tag",
        "disabled" => "adui-tag",
        _ => "adui-tag",
    }
}

fn status_code_class(status_code: u16) -> &'static str {
    if status_code < 400 {
        "adui-tag"
    } else {
        "adui-tag"
    }
}
