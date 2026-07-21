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

pub(super) const RENDERER_ID: &str = "drive-center.page";

#[component]
pub(super) fn DriveCenterClientPage(api_base_url: String) -> Element {
    let snapshot = use_resource(move || {
        let api_base_url = api_base_url.clone();
        async move { load_snapshot(&api_base_url).await }
    });
    render_resource(snapshot, drive_center_view)
}

fn drive_center_view(snapshot: &DriveCenterPageSnapshot) -> Element {
    let task_count = snapshot.tasks.len();
    rsx! {
        section { class: "space-y-4",
            PageHeader { eyebrow: "Operations / Storage", title: "Drive Center", description: "网盘任务、路径动作与 PostgreSQL 队列表。" }
            div { class: "grid gap-4 md:grid-cols-2",
                StatusCard { title: "运行态", status: snapshot.status.clone(), primary_api: "/api/drive-center/status" }
                Card {
                    CardHeader { CardTitle { "任务队列" } CardDescription { "{task_count} 条来自 drive-center API 的任务记录。" } }
                    CardContent {
                        if !snapshot.status.store_connected {
                            EmptyState { title: "数据源未连接", description: "未连接数据库，当前不读取任务队列。" }
                        } else if snapshot.tasks.is_empty() {
                            EmptyState { title: "暂无任务", description: "数据库当前没有网盘任务。" }
                        } else {
                            Table {
                                TableHeader { TableRow { TableHead { "路径" } TableHead { "动作" } TableHead { "状态" } TableHead { "ID" } } }
                                TableBody { for task in snapshot.tasks.iter().take(MAX_LIST_ROWS) {
                                    TableRow { TableCell { "{task.path}" } TableCell { code { "{task.action}" } } TableCell { Badge { variant: BadgeVariant::Secondary, "{task.status}" } } TableCell { code { "{task.id}" } } }
                                } }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn load_snapshot(api_base_url: &str) -> Result<DriveCenterPageSnapshot, String> {
    let status = fetch_json(api_base_url, "/api/drive-center/status").await?;
    let tasks = fetch_data(api_base_url, "/api/drive-center/tasks").await?;
    Ok(DriveCenterPageSnapshot { status, tasks })
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct DriveCenterPageSnapshot {
    status: PluginStatus,
    tasks: Vec<DriveTaskSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct DriveTaskSummary {
    id: String,
    path: String,
    action: String,
    status: String,
}
