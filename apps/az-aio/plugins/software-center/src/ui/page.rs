use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

use crate::ui::state::load_snapshot;

const MAX_LIST_ROWS: usize = 12;

#[allow(non_snake_case)]
pub fn SoftwareCenterPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let installer_count = snapshot.installers.len();
    let package_count = snapshot.packages.len();
    let status_url = api_url(&context.api_base_url, "/api/software-center/status");
    let installers_url = api_url(&context.api_base_url, "/api/software-center/installers");

    rsx! {
        section { class: "native-plugin-page native-plugin-page--software-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Knowledge / Software" }
                h1 { "Software Center" }
                p { "安装包扫描、归档结果与 PostgreSQL 软件包目录。" }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "运行态" }
                    dl {
                        dt { "路由" }
                        dd { "{context.active_route}" }
                        dt { "状态接口" }
                        dd { a { href: status_url.clone(), "{status_url}" } }
                        dt { "扫描接口" }
                        dd { a { href: installers_url.clone(), "{installers_url}" } }
                        dt { "DATABASE_URL" }
                        dd { "{configured_text(snapshot.status.database_configured)}" }
                        dt { "软件包表连接" }
                        dd { "{connected_text(snapshot.status.store_connected)}" }
                        dt { "表前缀" }
                        dd { code { "{snapshot.status.table_prefix}" } }
                    }
                    if let Some(error) = &snapshot.error {
                        p { class: "native-plugin-page__error", "{error}" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "本机安装包" }
                    p { "{installer_count} 个文件来自 Downloads 与 Desktop 的实时扫描。" }
                    if snapshot.installers.is_empty() {
                        p { class: "native-plugin-page__empty", "Downloads 与 Desktop 当前没有识别到安装包。" }
                    } else {
                        table {
                            thead {
                                tr {
                                    th { "文件" }
                                    th { "平台" }
                                    th { "架构" }
                                    th { "状态" }
                                }
                            }
                            tbody {
                                for installer in snapshot.installers.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{installer.file_name}" }
                                        td { "{installer.platform}" }
                                        td { "{installer.arch}" }
                                        td { "{installer.status}" }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "软件包目录" }
                    p { "{package_count} 条来自 software-center Toasty store 的软件包记录。" }
                    if !snapshot.status.store_connected {
                        p { class: "native-plugin-page__empty", "未连接数据库，当前不读取软件包目录。" }
                    } else if snapshot.packages.is_empty() {
                        p { class: "native-plugin-page__empty", "数据库当前没有软件包记录。" }
                    } else {
                        table {
                            thead {
                                tr {
                                    th { "名称" }
                                    th { "平台" }
                                    th { "架构" }
                                    th { "状态" }
                                }
                            }
                            tbody {
                                for package in snapshot.packages.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{package.name}" }
                                        td { "{package.platform}" }
                                        td { "{package.arch}" }
                                        td { "{package.status}" }
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
