use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

use crate::ui::state::load_snapshot;

const MAX_LIST_ROWS: usize = 12;

#[allow(non_snake_case)]
pub fn DriveCenterPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let task_count = snapshot.tasks.len();
    let status_url = api_url(&context.api_base_url, "/api/drive-center/status");
    let tasks_url = api_url(&context.api_base_url, "/api/drive-center/tasks");

    rsx! {
        section { class: "native-plugin-page native-plugin-page--drive-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Operations / Storage" }
                h1 { "Drive Center" }
                p { "网盘任务、路径动作与 PostgreSQL 队列表。" }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "运行态" }
                    dl {
                        dt { "路由" }
                        dd { "{context.active_route}" }
                        dt { "状态接口" }
                        dd { a { href: status_url.clone(), "{status_url}" } }
                        dt { "任务接口" }
                        dd { a { href: tasks_url.clone(), "{tasks_url}" } }
                        dt { "DATABASE_URL" }
                        dd { "{configured_text(snapshot.status.database_configured)}" }
                        dt { "任务表连接" }
                        dd { "{connected_text(snapshot.status.store_connected)}" }
                        dt { "表前缀" }
                        dd { code { "{snapshot.status.table_prefix}" } }
                    }
                    if let Some(error) = &snapshot.error {
                        p { class: "native-plugin-page__error", "{error}" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "任务队列" }
                    p { "{task_count} 条来自 drive-center Toasty store 的任务记录。" }
                    if !snapshot.status.store_connected {
                        p { class: "native-plugin-page__empty", "未连接数据库，当前不读取任务队列。" }
                    } else if snapshot.tasks.is_empty() {
                        p { class: "native-plugin-page__empty", "数据库当前没有网盘任务。" }
                    } else {
                        table {
                            thead {
                                tr {
                                    th { "路径" }
                                    th { "动作" }
                                    th { "状态" }
                                    th { "ID" }
                                }
                            }
                            tbody {
                                for task in snapshot.tasks.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{task.path}" }
                                        td { code { "{task.action}" } }
                                        td { "{task.status}" }
                                        td { code { "{task.id}" } }
                                    }
                                }
                            }
                        }
                    }
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
