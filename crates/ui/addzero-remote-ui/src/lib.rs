#![forbid(unsafe_code)]

use addzero_remote_model::{DeviceDescriptor, SessionState, SessionSummary};
use dioxus::prelude::*;

pub const REMOTE_STYLE: &str = include_str!("style.css");

#[derive(Clone, PartialEq)]
pub struct RemoteActionItem {
    pub label: String,
    pub tone: RemoteActionTone,
}

#[derive(Clone, PartialEq)]
pub enum RemoteActionTone {
    Primary,
    Neutral,
    Danger,
}

#[derive(Clone, PartialEq)]
pub struct RemoteStatusChip {
    pub label: String,
    pub emphasis: bool,
}

#[derive(Clone, PartialEq)]
pub struct RemotePermissionNotice {
    pub title: String,
    pub body: String,
    pub bullets: Vec<String>,
    pub cta_primary: String,
    pub cta_secondary: String,
}

#[derive(Clone, PartialEq)]
pub struct RemoteStageViewModel {
    pub title: String,
    pub subtitle: String,
    pub actions: Vec<RemoteActionItem>,
    pub status_chips: Vec<RemoteStatusChip>,
    pub placeholder_title: String,
    pub placeholder_body: String,
    pub permission_notice: Option<RemotePermissionNotice>,
}

#[component]
pub fn RemoteShell(title: String, sidebar: Element, stage: Element, detail: Element) -> Element {
    rsx! {
        div { class: "rd-shell",
            header { class: "rd-topbar",
                div {
                    p { class: "rd-eyebrow", "Addzero Remote Desktop" }
                    h1 { class: "rd-title", "{title}" }
                }
            }
            div { class: "rd-layout",
                aside { class: "rd-sidebar", {sidebar} }
                main { class: "rd-stage", {stage} }
                aside { class: "rd-detail", {detail} }
            }
        }
    }
}

#[component]
pub fn DeviceCard(device: DeviceDescriptor, active: bool) -> Element {
    let class = if active {
        "device-card device-card--active"
    } else {
        "device-card"
    };
    let status_class = if format!("{:?}", device.online_status) == "Online" {
        "device-card__status-dot device-card__status-dot--online"
    } else {
        "device-card__status-dot"
    };

    rsx! {
        article { class: class,
            div { class: "device-card__header",
                div {
                    div { class: "device-card__title", "{device.device_name}" }
                    div { class: "device-card__meta", "{device.platform}" }
                }
                div { class: "device-card__status",
                    span { class: status_class }
                    span { class: "device-card__status-text", "{device.online_status:?}" }
                }
            }
            if let Some(notes) = &device.notes {
                p { class: "device-card__note", "{notes}" }
            }
        }
    }
}

#[component]
pub fn RemoteStage(model: RemoteStageViewModel) -> Element {
    rsx! {
        section { class: "rd-stage__surface",
            div { class: "rd-stage__header",
                div {
                    h2 { class: "rd-stage__title", "{model.title}" }
                    p { class: "rd-stage__subtitle", "{model.subtitle}" }
                }
                div { class: "rd-stage__actions",
                    for action in model.actions.iter() {
                        RemoteActionButton { item: action.clone() }
                    }
                }
            }

            if !model.status_chips.is_empty() {
                div { class: "rd-stage__chips",
                    for chip in model.status_chips.iter() {
                        RemoteStatusBadge { chip: chip.clone() }
                    }
                }
            }

            div { class: "stage-screen stage-screen--rich",
                div { class: "stage-screen__center",
                    div { class: "stage-screen__title", "{model.placeholder_title}" }
                    p { class: "stage-screen__body", "{model.placeholder_body}" }
                }
            }

            if let Some(notice) = &model.permission_notice {
                section { class: "permission-card",
                    div { class: "permission-card__head",
                        h3 { class: "permission-card__title", "{notice.title}" }
                        p { class: "permission-card__body", "{notice.body}" }
                    }
                    ul { class: "permission-card__list",
                        for bullet in notice.bullets.iter() {
                            li { "{bullet}" }
                        }
                    }
                    div { class: "permission-card__actions",
                        button { class: "button button--primary", "{notice.cta_primary}" }
                        button { class: "button button--neutral", "{notice.cta_secondary}" }
                    }
                }
            }
        }
    }
}

#[component]
fn RemoteActionButton(item: RemoteActionItem) -> Element {
    let class = match item.tone {
        RemoteActionTone::Primary => "button button--primary",
        RemoteActionTone::Neutral => "button button--neutral",
        RemoteActionTone::Danger => "button button--danger",
    };
    rsx! { button { class: class, "{item.label}" } }
}

#[component]
fn RemoteStatusBadge(chip: RemoteStatusChip) -> Element {
    let class = if chip.emphasis {
        "status-chip status-chip--emphasis"
    } else {
        "status-chip"
    };
    rsx! { span { class: class, "{chip.label}" } }
}

#[component]
pub fn SessionPanel(summary: Option<SessionSummary>, allow_files: bool) -> Element {
    rsx! {
        section { class: "glass-card session-panel",
            h2 { class: "session-panel__title", "会话概览" }
            if let Some(summary) = summary {
                div { class: "session-group",
                    h3 { "连接状态" }
                    p { class: "session-value session-value--strong", "{format_session_state(&summary.state)}" }
                    if let Some(frame) = &summary.latest_frame {
                        p { class: "muted", "当前分辨率 {frame.width}×{frame.height}" }
                    }
                }

                div { class: "session-group",
                    h3 { "控制能力" }
                    p { class: "muted", "输入控制、剪贴板同步与远端刷新链路已接通。" }
                    if let Some(clipboard) = &summary.clipboard {
                        p { class: "muted", "最近剪贴板：{clipboard.content}" }
                    }
                }

                if allow_files {
                    div { class: "session-group",
                        h3 { "文件传输" }
                        if let Some(transfer) = &summary.pending_transfer {
                            p { class: "session-value", "{transfer.file_name}" }
                            p { class: "muted", "共 {transfer.total_bytes} bytes · 第 {transfer.chunk_index + 1}/{transfer.chunk_count} 块" }
                            div { class: "transfer-progress",
                                div {
                                    class: "transfer-progress__bar",
                                    style: format!(
                                        "width: {}%;",
                                        (((transfer.chunk_index + 1) as f32 / transfer.chunk_count as f32) * 100.0).round()
                                    ),
                                }
                            }
                        } else {
                            p { class: "muted", "暂无正在进行的文件传输。" }
                        }
                    }
                }

                div { class: "session-group session-group--diagnostic",
                    h3 { "诊断" }
                    if let Some(frame) = &summary.latest_frame {
                        p { class: "muted", "帧序号 #{frame.sequence}" }
                    }
                    p { class: "muted", "需要更多技术细节时，可在诊断模式继续展开。" }
                }
            } else {
                p { class: "muted", "当前没有活跃会话。" }
            }
        }
    }
}

fn format_session_state(state: &SessionState) -> &'static str {
    match state {
        SessionState::Requested => "等待批准",
        SessionState::Active => "已连接",
        SessionState::Closed => "已断开",
        SessionState::Rejected => "已拒绝",
    }
}
