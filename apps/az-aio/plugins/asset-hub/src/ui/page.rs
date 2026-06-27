use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

use crate::ui::state::load_snapshot;

const MAX_LIST_ROWS: usize = 12;

#[allow(non_snake_case)]
pub fn AssetHubPage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot();
    let asset_count = snapshot.assets.len();
    let skill_count = snapshot.scanned_skills.len();
    let status_url = api_url(&context.api_base_url, "/api/asset-hub/status");

    rsx! {
        section { class: "native-plugin-page native-plugin-page--asset-hub",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Knowledge / Assets" }
                h1 { "Asset Hub" }
                p { "资产库、技能目录扫描与 PostgreSQL 持久化资产。" }
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
                        dd { "{configured_text(snapshot.status.database_configured)}" }
                        dt { "资产表连接" }
                        dd { "{connected_text(snapshot.status.store_connected)}" }
                        dt { "表前缀" }
                        dd { code { "{snapshot.status.table_prefix}" } }
                    }
                    if let Some(error) = &snapshot.error {
                        p { class: "native-plugin-page__error", "{error}" }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "持久化资产" }
                    p { "{asset_count} 条来自 asset-hub Toasty store 的资产记录。" }
                    if !snapshot.status.store_connected {
                        p { class: "native-plugin-page__empty", "未连接数据库，当前不读取持久化资产。" }
                    } else if snapshot.assets.is_empty() {
                        p { class: "native-plugin-page__empty", "数据库当前没有资产记录。" }
                    } else {
                        table {
                            thead {
                                tr {
                                    th { "标题" }
                                    th { "类型" }
                                    th { "状态" }
                                    th { "来源" }
                                }
                            }
                            tbody {
                                for asset in snapshot.assets.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{asset.title}" }
                                        td { code { "{asset.kind}" } }
                                        td { "{asset.status}" }
                                        td { "{asset.source}" }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "native-plugin-card",
                    h2 { "技能目录扫描" }
                    p { "{skill_count} 个技能来自本机 ~/.agents/skills/SKILL.md 扫描结果。" }
                    if snapshot.scanned_skills.is_empty() {
                        p { class: "native-plugin-page__empty", "本机技能目录没有可展示的 SKILL.md 扫描结果。" }
                    } else {
                        ul {
                            for skill in snapshot.scanned_skills.iter().take(MAX_LIST_ROWS) {
                                li {
                                    strong { "{skill.name}" }
                                    span { " · {skill.status}" }
                                    br {}
                                    code { "{skill.source}" }
                                    if !skill.tags.is_empty() {
                                        div {
                                            for tag in skill.tags.iter() {
                                                span { class: "status-badge", "{tag}" }
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
