use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

use crate::ui::state::load_snapshot;

const MAX_LIST_ROWS: usize = 12;

#[allow(non_snake_case)]
pub fn EdgeGatewayPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let flow_count = snapshot.flows.len();
    let status_url = api_url(&context.api_base_url, "/api/edge-gateway/status");
    let flows_url = api_url(&context.api_base_url, "/api/edge-gateway/flows");
    let example_url = api_url(&context.api_base_url, "/api/edge-gateway/example");

    rsx! {
        section { class: "native-plugin-page native-plugin-page--edge-gateway",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Operations / Network" }
                h1 { "Edge Gateway" }
                p { "边缘网关流、执行路由与 PostgreSQL 流定义表。" }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "运行态" }
                    dl {
                        dt { "路由" }
                        dd { "{context.active_route}" }
                        dt { "状态接口" }
                        dd { a { href: status_url.clone(), "{status_url}" } }
                        dt { "流列表接口" }
                        dd { a { href: flows_url.clone(), "{flows_url}" } }
                        dt { "DATABASE_URL" }
                        dd { "{configured_text(snapshot.status.database_configured)}" }
                        dt { "流表连接" }
                        dd { "{connected_text(snapshot.status.store_connected)}" }
                        dt { "表前缀" }
                        dd { code { "{snapshot.status.table_prefix}" } }
                    }
                    if let Some(error) = &snapshot.error {
                        p { class: "native-plugin-page__error", "{error}" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "网关流" }
                    p { "{flow_count} 条来自 edge-gateway Toasty store 的流定义。" }
                    if !snapshot.status.store_connected {
                        p { class: "native-plugin-page__empty", "未连接数据库，当前不读取网关流。" }
                    } else if snapshot.flows.is_empty() {
                        p { class: "native-plugin-page__empty", "数据库当前没有网关流记录。" }
                    } else {
                        table {
                            thead {
                                tr {
                                    th { "名称" }
                                    th { "路由" }
                                    th { "状态" }
                                    th { "ID" }
                                }
                            }
                            tbody {
                                for flow in snapshot.flows.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{flow.name}" }
                                        td { code { "{flow.route}" } }
                                        td { "{flow.status}" }
                                        td { code { "{flow.id}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "执行合约" }
                    p { "参考执行计划包含 {snapshot.example_step_count} 个步骤，来自后端合约构造函数。" }
                    p { a { href: example_url.clone(), "{example_url}" } }
                }
            }
        }
    }
}

fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}{path}")
    }
}

fn configured_text(value: bool) -> &'static str {
    if value {
        "已配置"
    } else {
        "未配置"
    }
}

fn connected_text(value: bool) -> &'static str {
    if value {
        "已连接"
    } else {
        "未连接"
    }
}
