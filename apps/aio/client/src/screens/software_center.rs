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

pub(super) const RENDERER_ID: &str = "software-center.page";

#[component]
pub(super) fn SoftwareCenterClientPage(api_base_url: String) -> Element {
    let snapshot = use_resource(move || {
        let api_base_url = api_base_url.clone();
        async move { load_snapshot(&api_base_url).await }
    });
    render_resource(snapshot, software_center_view)
}

fn software_center_view(snapshot: &SoftwareCenterPageSnapshot) -> Element {
    let installer_count = snapshot.installers.len();
    let package_count = snapshot.packages.len();
    rsx! { section { class: "space-y-4",
        PageHeader { eyebrow: "Knowledge / Software", title: "Software Center", description: "安装包扫描、归档结果与 PostgreSQL 软件包目录。" }
        div { class: "grid gap-4 md:grid-cols-2",
            StatusCard { title: "运行态", status: snapshot.status.clone(), primary_api: "/api/software-center/status" }
            Card {
                CardHeader { CardTitle { "本机安装包" } CardDescription { "{installer_count} 个文件来自插件扫描 API。" } }
                CardContent {
                    if snapshot.installers.is_empty() {
                        EmptyState { title: "暂无安装包", description: "当前没有识别到安装包。" }
                    } else {
                        Table {
                            TableHeader { TableRow { TableHead { "文件" } TableHead { "平台" } TableHead { "架构" } TableHead { "状态" } } }
                            TableBody { for installer in snapshot.installers.iter().take(MAX_LIST_ROWS) {
                                TableRow { TableCell { "{installer.file_name}" } TableCell { "{installer.platform}" } TableCell { "{installer.arch}" } TableCell { Badge { variant: BadgeVariant::Secondary, "{installer.status}" } } }
                            } }
                        }
                    }
                }
            }
            Card {
                CardHeader { CardTitle { "软件包目录" } CardDescription { "{package_count} 条来自 software-center API 的软件包记录。" } }
                CardContent {
                    if !snapshot.status.store_connected {
                        EmptyState { title: "数据源未连接", description: "未连接数据库，当前不读取软件包目录。" }
                    } else if snapshot.packages.is_empty() {
                        EmptyState { title: "暂无软件包", description: "数据库当前没有软件包记录。" }
                    } else {
                        Table {
                            TableHeader { TableRow { TableHead { "名称" } TableHead { "平台" } TableHead { "架构" } TableHead { "状态" } } }
                            TableBody { for package in snapshot.packages.iter().take(MAX_LIST_ROWS) {
                                TableRow { TableCell { "{package.name}" } TableCell { "{package.platform}" } TableCell { "{package.arch}" } TableCell { Badge { variant: BadgeVariant::Secondary, "{package.status}" } } }
                            } }
                        }
                    }
                }
            }
        }
    } }
}

async fn load_snapshot(api_base_url: &str) -> Result<SoftwareCenterPageSnapshot, String> {
    let status = fetch_json(api_base_url, "/api/software-center/status").await?;
    let installers = fetch_data(api_base_url, "/api/software-center/installers").await?;
    let packages = fetch_data(api_base_url, "/api/software-center/packages").await?;
    Ok(SoftwareCenterPageSnapshot {
        status,
        installers,
        packages,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct SoftwareCenterPageSnapshot {
    status: PluginStatus,
    installers: Vec<InstallerPackage>,
    packages: Vec<SoftwarePackageSummary>,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallerPackage {
    file_name: String,
    platform: String,
    arch: String,
    status: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct SoftwarePackageSummary {
    name: String,
    platform: String,
    arch: String,
    status: String,
}
