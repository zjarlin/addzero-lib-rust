use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

use crate::ui::state::{DEFAULT_NAMESPACE, load_snapshot};

const MAX_LIST_ROWS: usize = 12;

#[allow(non_snake_case)]
pub fn ConfigCenterPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let status_url = api_url(&context.api_base_url, "/api/config-center/status");
    let dotfiles_url = api_url(&context.api_base_url, "/api/config-center/dotfiles");
    let pairing_url = api_url(&context.api_base_url, "/api/config-center/pairing");
    let entries_path = format!("/api/config-center/entries?namespace={DEFAULT_NAMESPACE}");
    let entries_url = api_url(&context.api_base_url, &entries_path);
    let action_url = api_url(&context.api_base_url, "/api/config-center/ui-action");
    let pending_count = snapshot
        .dotfiles
        .as_ref()
        .map(|dotfiles| dotfiles.pending_files.len())
        .unwrap_or_default();
    let conflict_count = snapshot
        .dotfiles
        .as_ref()
        .map(|dotfiles| dotfiles.conflicts.len())
        .unwrap_or_default();

    rsx! {
        section { class: "native-plugin-page native-plugin-page--config-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Environment / Config" }
                h1 { "Config Center" }
                p { "XDG 路径、Dotfiles 扫描、配对身份与 PostgreSQL 配置项。" }
            }
            if !snapshot.errors.is_empty() {
                article { class: "native-plugin-card native-plugin-card--danger",
                    h2 { "运行告警" }
                    ul {
                        for error in snapshot.errors.iter() {
                            li { "{error}" }
                        }
                    }
                }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "运行态" }
                    dl {
                        dt { "路由" }
                        dd { "{context.active_route}" }
                        dt { "状态接口" }
                        dd { a { href: status_url.clone(), "{status_url}" } }
                        dt { "DATABASE_URL" }
                        dd { "{configured_text(snapshot.status.as_ref().map(|status| status.database_configured).unwrap_or(false))}" }
                        dt { "配置表连接" }
                        dd { "{connected_text(snapshot.status.as_ref().map(|status| status.store_connected).unwrap_or(false))}" }
                        dt { "表前缀" }
                        dd { code { "{table_prefix(&snapshot)}" } }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "XDG 路径" }
                    if let Some(status) = &snapshot.status {
                        dl {
                            dt { "data" }
                            dd { code { "{status.paths.data_dir}" } }
                            dt { "config" }
                            dd { code { "{status.paths.config_dir}" } }
                            dt { "state" }
                            dd { code { "{status.paths.state_dir}" } }
                            dt { "cache" }
                            dd { code { "{status.paths.cache_dir}" } }
                        }
                    } else {
                        p { class: "native-plugin-page__empty", "当前无法解析 XDG 路径。" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "Dotfiles 扫描" }
                    p { a { href: dotfiles_url.clone(), "{dotfiles_url}" } }
                    if let Some(dotfiles) = &snapshot.dotfiles {
                        dl {
                            dt { "监控文件" }
                            dd { "{dotfiles.watched_files}" }
                            dt { "变更文件" }
                            dd { "{dotfiles.changed_files}" }
                            dt { "待处理" }
                            dd { "{pending_count}" }
                            dt { "冲突" }
                            dd { "{conflict_count}" }
                            dt { "baseline" }
                            dd { code { "{dotfiles.baseline_path}" } }
                        }
                        if dotfiles.pending_files.is_empty() {
                            p { class: "native-plugin-page__empty", "当前没有待同步 dotfiles。" }
                        } else {
                            ul {
                                for file in dotfiles.pending_files.iter().take(MAX_LIST_ROWS) {
                                    li {
                                        strong { "{file.relative_path}" }
                                        span { " · {file.status}" }
                                        br {}
                                        code { "{file.target_path}" }
                                    }
                                }
                            }
                        }
                    } else {
                        p { class: "native-plugin-page__empty", "当前无法读取 dotfiles 扫描状态。" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "配对身份" }
                    p { a { href: pairing_url.clone(), "{pairing_url}" } }
                    if let Some(pairing) = &snapshot.pairing {
                        dl {
                            dt { "设备" }
                            dd { "{pairing.device_name}" }
                            dt { "指纹" }
                            dd { code { "{pairing.fingerprint}" } }
                            dt { "home" }
                            dd { code { "{pairing.home_path}" } }
                            dt { "metadata" }
                            dd { code { "{pairing.metadata_path}" } }
                        }
                    } else {
                        p { class: "native-plugin-page__empty", "当前无法读取本机配对身份。" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "配置项" }
                    p { "{snapshot.entries.len()} 条来自 {DEFAULT_NAMESPACE} namespace 的 PostgreSQL 记录。" }
                    p { a { href: entries_url.clone(), "{entries_url}" } }
                    if !snapshot.status.as_ref().map(|status| status.store_connected).unwrap_or(false) {
                        p { class: "native-plugin-page__empty", "未连接数据库，当前不读取配置项。" }
                    } else if snapshot.entries.is_empty() {
                        p { class: "native-plugin-page__empty", "数据库当前没有配置项记录。" }
                    } else {
                        table {
                            thead {
                                tr {
                                    th { "namespace" }
                                    th { "key" }
                                    th { "value" }
                                }
                            }
                            tbody {
                                for entry in snapshot.entries.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{entry.namespace}" }
                                        td { code { "{entry.key}" } }
                                        td { "{entry.value}" }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "写入入口" }
                    p { "配置写入走统一 REST API；页面不维护一份私有状态。" }
                    form {
                        method: "post",
                        action: action_url.clone(),
                        label { "namespace" }
                        input { name: "namespace", value: "{DEFAULT_NAMESPACE}" }
                        label { "key" }
                        input { name: "key", placeholder: "projects.default_sync_root" }
                        label { "value" }
                        input { name: "value", placeholder: "az-sync/workspace" }
                        button { r#type: "submit", "提交配置项" }
                    }
                }
            }
        }
    }
}

fn api_url(api_base_url: &str, path: &str) -> String {
    let base = api_base_url.trim_end_matches('/');
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

fn table_prefix(snapshot: &crate::ui::state::ConfigCenterPageSnapshot) -> String {
    snapshot
        .status
        .as_ref()
        .map(|status| status.table_prefix.clone())
        .unwrap_or_else(|| "biz_config_center_".to_string())
}
