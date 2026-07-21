use dioxus::prelude::*;
use registry::ui::{
    badge::{Badge, BadgeVariant},
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
};
use serde::Deserialize;

use crate::{
    http::{fetch_data, fetch_json},
    screens::{EmptyState, MAX_LIST_ROWS, PageHeader, PluginStatus, StatusCard, render_resource},
};

pub(super) const RENDERER_ID: &str = "asset-hub.page";

#[component]
pub(super) fn AssetHubClientPage(api_base_url: String) -> Element {
    let snapshot = use_resource(move || {
        let api_base_url = api_base_url.clone();
        async move { load_snapshot(&api_base_url).await }
    });
    render_resource(snapshot, asset_hub_view)
}

fn asset_hub_view(snapshot: &AssetHubPageSnapshot) -> Element {
    let asset_count = snapshot.assets.len();
    let skill_count = snapshot.scanned_skills.len();
    rsx! { section { class: "space-y-4",
        PageHeader { eyebrow: "Knowledge / Assets", title: "Asset Hub", description: "资产库、技能目录扫描与 PostgreSQL 持久化资产。" }
        div { class: "grid gap-4 md:grid-cols-2",
            StatusCard { title: "运行态", status: snapshot.status.clone(), primary_api: "/api/asset-hub/status" }
            Card {
                CardHeader { CardTitle { "持久化资产" } CardDescription { "{asset_count} 条来自 asset-hub API 的资产记录。" } }
                CardContent {
                    if !snapshot.status.store_connected {
                        EmptyState { title: "数据源未连接", description: "未连接数据库，当前不读取持久化资产。" }
                    } else if snapshot.assets.is_empty() {
                        EmptyState { title: "暂无资产", description: "数据库当前没有资产记录。" }
                    } else {
                        Table {
                            TableHeader { TableRow { TableHead { "标题" } TableHead { "类型" } TableHead { "状态" } TableHead { "来源" } } }
                            TableBody { for asset in snapshot.assets.iter().take(MAX_LIST_ROWS) {
                                TableRow { TableCell { "{asset.title}" } TableCell { code { "{asset.kind}" } } TableCell { Badge { variant: BadgeVariant::Secondary, "{asset.status}" } } TableCell { "{asset.source}" } }
                            } }
                        }
                    }
                }
            }
            Card {
                CardHeader { CardTitle { "技能目录扫描" } CardDescription { "{skill_count} 个技能来自 asset-hub API。" } }
                CardContent {
                    if snapshot.scanned_skills.is_empty() {
                        EmptyState { title: "暂无技能", description: "当前没有可展示的 SKILL.md 扫描结果。" }
                    } else {
                        ul { class: "space-y-2", for skill in snapshot.scanned_skills.iter().take(MAX_LIST_ROWS) {
                            li { class: "rounded-md border p-3", strong { "{skill.name}" } Badge { class: "ml-2", variant: BadgeVariant::Secondary, "{skill.status}" } br {} code { "{skill.source}" } }
                        } }
                    }
                }
            }
        }
    } }
}

async fn load_snapshot(api_base_url: &str) -> Result<AssetHubPageSnapshot, String> {
    let status = fetch_json(api_base_url, "/api/asset-hub/status").await?;
    let scanned_skills = fetch_data(api_base_url, "/api/asset-hub/skills").await?;
    let assets = fetch_data(api_base_url, "/api/asset-hub/assets").await?;
    Ok(AssetHubPageSnapshot {
        status,
        assets,
        scanned_skills,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct AssetHubPageSnapshot {
    status: PluginStatus,
    assets: Vec<AssetSummary>,
    scanned_skills: Vec<ScannedSkillAsset>,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct AssetSummary {
    kind: String,
    title: String,
    status: String,
    source: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScannedSkillAsset {
    name: String,
    source: String,
    status: String,
}
