mod asset_hub;
mod drive_center;
mod software_center;

use dioxus::prelude::*;
use registry::ui::{
    alert::{Alert, AlertDescription, AlertTitle, AlertVariant},
    badge::{Badge, BadgeVariant},
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    empty::{Empty, EmptyDescription, EmptyTitle},
};

const MAX_LIST_ROWS: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
struct PluginStatus {
    database_configured: bool,
    store_connected: bool,
    table_prefix: String,
}

pub(crate) fn render(renderer_id: Option<&str>, route: &str, api_base_url: String) -> Element {
    match renderer_id {
        Some(asset_hub::RENDERER_ID) => rsx! { asset_hub::AssetHubClientPage { api_base_url } },
        Some(drive_center::RENDERER_ID) => {
            rsx! { drive_center::DriveCenterClientPage { api_base_url } }
        }
        Some(software_center::RENDERER_ID) => {
            rsx! { software_center::SoftwareCenterClientPage { api_base_url } }
        }
        _ => rsx! {
            Card {
                CardHeader {
                    CardTitle { "SSR fallback route" }
                    CardDescription { "当前路由尚未迁移到 Dioxus client，会继续由服务端 SSR fallback 提供首屏内容。" }
                }
                CardContent {
                    a { href: format!("?route={route}"), "Open SSR fallback" }
                }
            }
        },
    }
}

fn render_resource<T>(resource: Resource<Result<T, String>>, view: fn(&T) -> Element) -> Element
where
    T: 'static,
{
    match &*resource.read_unchecked() {
        Some(Ok(snapshot)) => view(snapshot),
        Some(Err(error)) => rsx! {
            Alert { variant: AlertVariant::Destructive,
                AlertTitle { "插件页面加载失败" }
                AlertDescription { "{error}" }
            }
        },
        None => rsx! {
            Card {
                CardHeader {
                    CardTitle { "加载中" }
                    CardDescription { "正在通过插件 API 获取页面数据。" }
                }
            }
        },
    }
}

#[component]
fn PageHeader(eyebrow: &'static str, title: &'static str, description: &'static str) -> Element {
    rsx! {
        Card {
            CardHeader {
                Badge { variant: BadgeVariant::Outline, "{eyebrow}" }
                CardTitle { class: "text-lg", "{title}" }
                CardDescription { "{description}" }
            }
        }
    }
}

#[component]
fn StatusCard(title: &'static str, status: PluginStatus, primary_api: &'static str) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{title}" }
                CardDescription { "插件运行时和正式数据源状态。" }
            }
            CardContent {
                dl { class: "grid gap-2 text-sm",
                    dt { "状态接口" }
                    dd { a { href: primary_api, "{primary_api}" } }
                    dt { "DATABASE_URL" }
                    dd { "{configured_text(status.database_configured)}" }
                    dt { "数据表连接" }
                    dd { "{connected_text(status.store_connected)}" }
                    dt { "表前缀" }
                    dd { code { "{status.table_prefix}" } }
                }
            }
        }
    }
}

#[component]
fn EmptyState(title: &'static str, description: &'static str) -> Element {
    rsx! {
        Empty {
            EmptyTitle { "{title}" }
            EmptyDescription { "{description}" }
        }
    }
}

fn configured_text(value: bool) -> &'static str {
    if value { "已配置" } else { "未配置" }
}

fn connected_text(value: bool) -> &'static str {
    if value { "已连接" } else { "未连接" }
}
