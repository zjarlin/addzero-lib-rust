#![forbid(unsafe_code)]

use addzero_remote_model::{DeviceDescriptor, SessionSummary};
use dioxus::prelude::*;

pub const REMOTE_STYLE: &str = include_str!("style.css");

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
    rsx! {
        article { class: class,
            div { class: "device-card__title", "{device.device_name}" }
            div { class: "device-card__meta", "{device.platform} · {device.online_status:?}" }
            if let Some(notes) = &device.notes {
                p { class: "device-card__note", "{notes}" }
            }
        }
    }
}

#[component]
pub fn SessionPanel(summary: Option<SessionSummary>, allow_files: bool) -> Element {
    rsx! {
        section { class: "glass-card",
            h2 { "Session" }
            if let Some(summary) = summary {
                p { class: "muted", "State: {summary.state:?}" }
                if let Some(clipboard) = &summary.clipboard {
                    p { class: "muted", "Clipboard: {clipboard.content}" }
                }
                if let Some(frame) = &summary.latest_frame {
                    p { class: "muted", "Frame: #{frame.sequence} · {frame.width}×{frame.height}" }
                }
                if allow_files {
                    if let Some(transfer) = &summary.pending_transfer {
                        p { class: "muted", "File: {transfer.file_name} ({transfer.total_bytes} bytes)" }
                    } else {
                        p { class: "muted", "File panel ready." }
                    }
                }
            } else {
                p { class: "muted", "No active session." }
            }
        }
    }
}
