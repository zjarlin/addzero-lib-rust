use dioxus::prelude::*;
use dioxus_components::{
    Badge, ContentHeader, DataTable, MetricRow, MetricStrip, Stack, StatTile, Surface, SurfaceHeader,
    Tone, WorkbenchButton,
};
use uuid::Uuid;

use addzero_agent_runtime_contract::{
    AgentArtifact, AgentArtifactChannel, AgentNodeStatus, ConflictResolution,
    PairingSessionSummary, PairingStatus, ResolveConflictRequest, SkillConflict,
};

use crate::app::Route;
use crate::state::AppServices;

#[component]
pub fn SystemAgentNodes() -> Element {
    let runtime_api = use_context::<AppServices>().agent_runtime.clone();
    let resolving = use_signal::<Option<Uuid>>(|| None);
    let feedback = use_signal::<Option<String>>(|| None);
    let overview_resource = {
        let runtime_api = runtime_api.clone();
        use_resource(move || {
            let runtime_api = runtime_api.clone();
            async move { runtime_api.overview().await }
        })
    };

    let overview = match overview_resource.read().as_ref() {
        Some(Ok(overview)) => overview.clone(),
        Some(Err(err)) => {
            return rsx! {
                ContentHeader {
                    title: "系统管理".to_string(),
                    subtitle: "Agent 节点：配对 runtime、安装包和同步状态。".to_string()
                }
                Surface { div { class: "callout", "无法加载 Agent 节点：{err}" } }
            };
        }
        None => {
            return rsx! {
                ContentHeader {
                    title: "系统管理".to_string(),
                    subtitle: "Agent 节点：配对 runtime、安装包和同步状态。".to_string()
                }
                Surface { div { class: "empty-state", "正在加载 Agent 节点…" } }
            };
        }
    };

    let pending_pairings = overview
        .pairing_sessions
        .iter()
        .filter(|pairing| {
            matches!(
                pairing.status,
                PairingStatus::Pending | PairingStatus::Approved
            )
        })
        .count();
    let unresolved_conflicts = overview
        .conflicts
        .iter()
        .filter(|conflict| conflict.resolved_at.is_none())
        .count();
    let active_node = overview.active_node.clone();

    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "Agent 节点：独立 runtime 的下载、配对、在线状态和 skills 双向同步。".to_string()
        }
        MetricStrip { columns: 4,
            StatTile {
                label: "分发形态".to_string(),
                value: overview.artifacts.len().to_string(),
                detail: "macOS 后台二进制为主，docker compose 为辅。".to_string()
            }
            StatTile {
                label: "待批准配对".to_string(),
                value: pending_pairings.to_string(),
                detail: "当前等待在浏览器批准页确认的 child runtime。".to_string()
            }
            StatTile {
                label: "当前节点".to_string(),
                value: active_node.as_ref().map(|node| node.display_name.clone()).unwrap_or_else(|| "未配对".to_string()),
                detail: "MVP 仅允许一个 active child runtime。".to_string()
            }
            StatTile {
                label: "未解决冲突".to_string(),
                value: unresolved_conflicts.to_string(),
                detail: "skills 双端都改时会停下等待人工处理。".to_string()
            }
        }
        if let Some(msg) = feedback.read().as_ref() {
            div { class: "callout callout--info", "{msg}" }
        }
        Surface {
            SurfaceHeader {
                title: "下载与安装".to_string(),
                subtitle: "这部分只登记外部独立工程的产物元数据，不在本仓库编译 agent 本体。".to_string()
            }
            div { class: "stack",
                for artifact in overview.artifacts.iter() {
                    ArtifactCard { artifact: artifact.clone() }
                }
            }
        }
        Surface {
            SurfaceHeader {
                title: "配对状态".to_string(),
                subtitle: "child runtime 启动后先创建 pairing session，再由当前后台会话批准。".to_string()
            }
            if overview.pairing_sessions.is_empty() {
                div { class: "empty-state", "还没有任何 pairing session。" }
            } else {
                DataTable {
                    columns: vec!["节点".to_string(), "交付形态".to_string(), "状态".to_string(), "批准页".to_string()],
                    for pairing in overview.pairing_sessions.iter() {
                        PairingRow { pairing: pairing.clone() }
                    }
                }
            }
        }
        Surface {
            SurfaceHeader {
                title: "当前节点".to_string(),
                subtitle: "新节点换 token 成功后会自动 revoke 旧节点。".to_string()
            }
            if let Some(node) = active_node {
                Stack {
                    MetricRow {
                        label: "节点".to_string(),
                        value: node.display_name.clone(),
                        tone: status_tone(node.status),
                    }
                    MetricRow {
                        label: "平台".to_string(),
                        value: format!("{} · {}", channel_label(node.channel), node.platform),
                    }
                    MetricRow {
                        label: "版本".to_string(),
                        value: node.agent_version.clone(),
                    }
                    MetricRow {
                        label: "最近心跳".to_string(),
                        value: node.last_seen_at.map(format_timestamp).unwrap_or_else(|| "尚未心跳".to_string()),
                        tone: status_tone(node.status),
                    }
                    MetricRow {
                        label: "最近同步".to_string(),
                        value: node.last_sync_at.map(format_timestamp).unwrap_or_else(|| "尚未同步".to_string()),
                    }
                    MetricRow {
                        label: "最近统计".to_string(),
                        value: format!(
                            "上传 {} / 下发 {} / 冲突 {}",
                            node.last_uploaded_count,
                            node.last_downloaded_count,
                            node.last_conflict_count
                        ),
                    }
                }
            } else {
                div { class: "empty-state", "当前没有 active 节点。先在 agent 侧创建 pairing session。" }
            }
        }
        Surface {
            SurfaceHeader {
                title: "最近同步与冲突".to_string(),
                subtitle: format!("当前 canonical skills 根目录：{}。", overview.fs_root)
            }
            if overview.conflicts.is_empty() {
                div { class: "empty-state", "当前没有未处理的冲突。" }
            } else {
                div { class: "stack",
                    for conflict in overview.conflicts.iter() {
                        ConflictCard {
                            conflict: conflict.clone(),
                            resolving: *resolving.read() == Some(conflict.id),
                            on_use_web: {
                                let id = conflict.id;
                                let runtime_api = runtime_api.clone();
                                move |_| {
                                    let runtime_api = runtime_api.clone();
                                    let mut resolving = resolving;
                                    let mut feedback = feedback;
                                    let mut overview_resource = overview_resource;
                                    spawn(async move {
                                        resolving.set(Some(id));
                                        match runtime_api
                                            .resolve_conflict(
                                                id,
                                                ResolveConflictRequest {
                                                    resolution: ConflictResolution::UseWeb,
                                                },
                                            )
                                            .await
                                        {
                                            Ok(conflict) => {
                                                feedback.set(Some(format!(
                                                    "已处理冲突：{}",
                                                    conflict.skill_name
                                                )));
                                                overview_resource.restart();
                                            }
                                            Err(err) => feedback
                                                .set(Some(format!("处理冲突失败：{err}"))),
                                        }
                                        resolving.set(None);
                                    });
                                }
                            },
                            on_use_agent: {
                                let id = conflict.id;
                                let runtime_api = runtime_api.clone();
                                move |_| {
                                    let runtime_api = runtime_api.clone();
                                    let mut resolving = resolving;
                                    let mut feedback = feedback;
                                    let mut overview_resource = overview_resource;
                                    spawn(async move {
                                        resolving.set(Some(id));
                                        match runtime_api
                                            .resolve_conflict(
                                                id,
                                                ResolveConflictRequest {
                                                    resolution: ConflictResolution::UseAgent,
                                                },
                                            )
                                            .await
                                        {
                                            Ok(conflict) => {
                                                feedback.set(Some(format!(
                                                    "已处理冲突：{}",
                                                    conflict.skill_name
                                                )));
                                                overview_resource.restart();
                                            }
                                            Err(err) => feedback
                                                .set(Some(format!("处理冲突失败：{err}"))),
                                        }
                                        resolving.set(None);
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SystemAgentPairingApproval(id: String) -> Element {
    let runtime_api = use_context::<AppServices>().agent_runtime.clone();
    let parsed_id = Uuid::parse_str(id.as_str()).ok();
    let mut approving = use_signal(|| false);
    let mut feedback = use_signal::<Option<String>>(|| None);
    let mut pairing_resource = {
        let runtime_api = runtime_api.clone();
        use_resource(move || {
            let runtime_api = runtime_api.clone();
            async move {
                let pairing_id = parsed_id.ok_or_else(|| "无效的 pairing id".to_string())?;
                runtime_api
                    .get_pairing(pairing_id)
                    .await
                    .map_err(|err| err.to_string())
            }
        })
    };

    let pairing = match pairing_resource.read().as_ref() {
        Some(Ok(pairing)) => pairing.clone(),
        Some(Err(err)) => {
            return rsx! {
                ContentHeader {
                    title: "系统管理".to_string(),
                    subtitle: "批准 Agent 节点配对".to_string()
                }
                Surface { div { class: "callout", "{err}" } }
            };
        }
        None => {
            return rsx! {
                ContentHeader {
                    title: "系统管理".to_string(),
                    subtitle: "批准 Agent 节点配对".to_string()
                }
                Surface { div { class: "empty-state", "正在加载配对信息…" } }
            };
        }
    };

    let approve = move |_| {
        let Some(pairing_id) = parsed_id else {
            feedback.set(Some("无效的 pairing id".to_string()));
            return;
        };
        let runtime_api = runtime_api.clone();
        spawn(async move {
            approving.set(true);
            match runtime_api.approve_pairing(pairing_id).await {
                Ok(updated) => {
                    feedback.set(Some(format!("已批准节点：{}", updated.device_name)));
                    pairing_resource.restart();
                }
                Err(err) => feedback.set(Some(format!("批准失败：{err}"))),
            }
            approving.set(false);
        });
    };

    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "批准 Agent 节点配对".to_string()
        }
        Surface {
            SurfaceHeader {
                title: pairing.device_name.clone(),
                subtitle: format!(
                    "{} · {} · {}",
                    channel_label(pairing.channel),
                    pairing.platform,
                    pairing.agent_version
                )
            }
            Stack {
                MetricRow {
                    label: "状态".to_string(),
                    value: pairing_status_label(pairing.status).to_string(),
                    tone: pairing_status_tone(pairing.status),
                }
                MetricRow {
                    label: "过期时间".to_string(),
                    value: format_timestamp(pairing.expires_at),
                }
                MetricRow {
                    label: "批准页".to_string(),
                    value: pairing.approve_url.clone(),
                }
            }
            if let Some(msg) = feedback.read().as_ref() {
                div { class: "callout callout--info", "{msg}" }
            }
            div { class: "editor-footer",
                Link { to: Route::SystemAgentNodes,
                    WorkbenchButton { class: "toolbar-button".to_string(), "返回 Agent 节点" }
                }
                span { class: "editor-footer__spacer" }
                if matches!(pairing.status, PairingStatus::Pending) {
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        tone: Tone::Accent,
                        onclick: approve,
                        if *approving.read() { "批准中…" } else { "批准配对" }
                    }
                } else {
                    WorkbenchButton {
                        class: "toolbar-button".to_string(),
                        "{pairing_status_label(pairing.status)}"
                    }
                }
            }
        }
    }
}

#[component]
fn ArtifactCard(artifact: AgentArtifact) -> Element {
    let badge_variant = match artifact.channel {
        AgentArtifactChannel::MacosBinary => "pg",
        AgentArtifactChannel::DockerCompose => "both",
    };

    rsx! {
        div { class: "callout",
            div { class: "stack",
                div { class: "knowledge-source",
                    Badge { label: channel_label(artifact.channel).to_string(), variant: badge_variant.to_string() }
                    span { class: "badge", "{artifact.version}" }
                    span { class: "badge badge--fs", "{artifact.platform}" }
                }
                strong { "{artifact.title}" }
                div { "下载：{artifact.download_url}" }
                div { "Checksum：{artifact.checksum}" }
                div { "安装：{artifact.install_command}" }
                div { "启动：{artifact.launch_command}" }
                div { "卸载：{artifact.uninstall_command}" }
                div { class: "cell-overflow", "{artifact.note}" }
            }
        }
    }
}

#[component]
fn PairingRow(pairing: PairingSessionSummary) -> Element {
    rsx! {
        tr {
            td {
                div { class: "stack",
                    strong { "{pairing.device_name}" }
                    span { class: "cell-overflow", "{pairing.platform} · {pairing.agent_version}" }
                }
            }
            td { "{channel_label(pairing.channel)}" }
            td {
                Badge {
                    label: pairing_status_label(pairing.status).to_string(),
                    variant: pairing_badge(pairing.status).to_string()
                }
            }
            td {
                Link { to: Route::SystemAgentPairingApproval { id: pairing.id.to_string() },
                    WorkbenchButton { class: "toolbar-button".to_string(), "打开批准页" }
                }
            }
        }
    }
}

#[component]
fn ConflictCard(
    conflict: SkillConflict,
    resolving: bool,
    on_use_web: EventHandler<MouseEvent>,
    on_use_agent: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "callout",
            div { class: "stack",
                div { class: "knowledge-source",
                    Badge { label: "冲突".to_string(), variant: "warning".to_string() }
                    span { class: "badge", "{conflict.skill_name}" }
                }
                div { "Server hash：{short_hash(conflict.server_hash.as_str())}" }
                div { "Agent hash：{short_hash(conflict.agent_hash.as_str())}" }
                div { "创建时间：{format_timestamp(conflict.created_at)}" }
                div { class: "editor-footer",
                    WorkbenchButton {
                        class: "toolbar-button".to_string(),
                        onclick: move |event| on_use_web.call(event),
                        if resolving { "处理中…" } else { "接受 Web 版本" }
                    }
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        tone: Tone::Accent,
                        onclick: move |event| on_use_agent.call(event),
                        if resolving { "处理中…" } else { "接受 Agent 版本" }
                    }
                }
            }
        }
    }
}

fn channel_label(channel: AgentArtifactChannel) -> &'static str {
    match channel {
        AgentArtifactChannel::MacosBinary => "macOS 后台二进制",
        AgentArtifactChannel::DockerCompose => "docker compose",
    }
}

fn pairing_status_label(status: PairingStatus) -> &'static str {
    match status {
        PairingStatus::Pending => "待批准",
        PairingStatus::Approved => "已批准",
        PairingStatus::Exchanged => "已换 token",
        PairingStatus::Expired => "已过期",
        PairingStatus::Revoked => "已撤销",
    }
}

fn pairing_badge(status: PairingStatus) -> &'static str {
    match status {
        PairingStatus::Pending => "fs",
        PairingStatus::Approved => "pg",
        PairingStatus::Exchanged => "both",
        PairingStatus::Expired | PairingStatus::Revoked => "warning",
    }
}

fn pairing_status_tone(status: PairingStatus) -> Tone {
    match status {
        PairingStatus::Pending => Tone::Warning,
        PairingStatus::Approved | PairingStatus::Exchanged => Tone::Positive,
        PairingStatus::Expired | PairingStatus::Revoked => Tone::Default,
    }
}

fn status_tone(status: AgentNodeStatus) -> Tone {
    match status {
        AgentNodeStatus::Online => Tone::Positive,
        AgentNodeStatus::Pending | AgentNodeStatus::Offline => Tone::Warning,
        AgentNodeStatus::Revoked => Tone::Default,
    }
}

fn format_timestamp(value: chrono::DateTime<chrono::Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn short_hash(value: &str) -> String {
    if value.len() <= 14 {
        value.to_string()
    } else {
        format!("{}…", &value[..14])
    }
}
