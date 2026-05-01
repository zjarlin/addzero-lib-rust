#![cfg_attr(
    all(target_os = "windows", feature = "bundle"),
    windows_subsystem = "windows"
)]

use addzero_remote_host::{HostPlatformAdapter, MockHostPlatformAdapter};
use addzero_remote_model::{
    ClipboardPayload, DeviceDescriptor, DeviceRole, FileTransferEnvelope, OnlineStatus,
    RemotePlatform, SessionCapability, SessionSummary, VideoCodec, VideoFrameEnvelope,
};
use addzero_remote_session::RemoteRelayService;
use addzero_remote_ui::{
    DeviceCard, REMOTE_STYLE, RemoteActionItem, RemoteActionTone, RemotePermissionNotice,
    RemoteShell, RemoteStage, RemoteStageViewModel, RemoteStatusChip, SessionPanel,
};
use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

fn main() {
    dioxus::launch(App);
}

fn seed_devices() -> (Vec<DeviceDescriptor>, SessionSummary) {
    let adapter = MockHostPlatformAdapter;
    let host = adapter.descriptor("Studio Mac").expect("host");
    let viewer = DeviceDescriptor {
        device_id: Uuid::new_v4(),
        device_name: "Operator Console".into(),
        platform: RemotePlatform::MacOs,
        role: DeviceRole::Viewer,
        capabilities: SessionCapability::full_host(),
        online_status: OnlineStatus::Online,
        last_seen_at: Utc::now(),
        notes: Some("当前控制台可发起远程控制、剪贴板同步与文件传输。".into()),
    };
    let mut relay = RemoteRelayService::new();
    let viewer = relay.register_device(viewer);
    let host = relay.register_device(host);
    let request = relay
        .request_session(
            viewer.device_id,
            host.device_id,
            SessionCapability::full_host(),
        )
        .expect("request");
    relay
        .grant_session(request.session_id, true, None)
        .expect("grant");
    relay
        .push_clipboard(
            request.session_id,
            ClipboardPayload {
                content: "构建完成通知已同步到本地剪贴板。".into(),
                updated_at: Utc::now(),
            },
        )
        .expect("clipboard");
    relay
        .push_frame(
            request.session_id,
            VideoFrameEnvelope {
                session_id: request.session_id,
                codec: VideoCodec::JpegFrames,
                width: 1728,
                height: 1117,
                sequence: 42,
                captured_at: Utc::now(),
            },
        )
        .expect("frame");
    relay
        .stage_file_transfer(
            request.session_id,
            FileTransferEnvelope {
                transfer_id: Uuid::new_v4(),
                session_id: request.session_id,
                file_name: "release-notes.txt".into(),
                total_bytes: 4096,
                chunk_index: 5,
                chunk_count: 8,
            },
        )
        .expect("file transfer");
    (
        relay.list_devices(),
        relay.session_summary(request.session_id).expect("summary"),
    )
}

#[component]
fn App() -> Element {
    let (devices, summary) = seed_devices();
    let selected_host = devices
        .iter()
        .find(|device| matches!(device.role, DeviceRole::Host))
        .cloned();

    let stage_model = RemoteStageViewModel {
        title: selected_host
            .as_ref()
            .map(|device| format!("{} · macOS · 已连接", device.device_name))
            .unwrap_or_else(|| "远程桌面工作区".to_string()),
        subtitle: "主工作区只保留连接控制、状态反馈和权限引导；诊断细节放到右侧概览。".to_string(),
        actions: vec![
            RemoteActionItem {
                label: "重新连接".to_string(),
                tone: RemoteActionTone::Primary,
            },
            RemoteActionItem {
                label: "全屏".to_string(),
                tone: RemoteActionTone::Neutral,
            },
            RemoteActionItem {
                label: "文件传输".to_string(),
                tone: RemoteActionTone::Neutral,
            },
            RemoteActionItem {
                label: "断开连接".to_string(),
                tone: RemoteActionTone::Danger,
            },
        ],
        status_chips: vec![
            RemoteStatusChip {
                label: "连接状态：已连接".to_string(),
                emphasis: true,
            },
            RemoteStatusChip {
                label: "剪贴板同步：已开启".to_string(),
                emphasis: false,
            },
            RemoteStatusChip {
                label: "分辨率：1728×1117".to_string(),
                emphasis: false,
            },
            RemoteStatusChip {
                label: "输入控制：可用".to_string(),
                emphasis: false,
            },
        ],
        placeholder_title: "远程桌面画布占位区".to_string(),
        placeholder_body: "这里应该承载真实远程画面，而不是调试说明。当前先把主工作区结构、状态层和权限引导重构到可上线的产品形态。".to_string(),
        permission_notice: Some(RemotePermissionNotice {
            title: "目标主机还不能完整接管".to_string(),
            body: "继续远程控制前，需要在 Host 端补齐屏幕录制和辅助功能权限，否则只能看到有限画面或无法注入输入。".to_string(),
            bullets: vec![
                "打开系统设置 → 隐私与安全性 → 屏幕录制，授权 Host 进程。".to_string(),
                "打开系统设置 → 隐私与安全性 → 辅助功能，允许键鼠控制。".to_string(),
                "完成授权后返回这里执行“重新连接”。".to_string(),
            ],
            cta_primary: "打开授权步骤".to_string(),
            cta_secondary: "授权后重试".to_string(),
        }),
    };

    rsx! {
        document::Style { "{REMOTE_STYLE}" }
        RemoteShell {
            title: "Desktop Viewer".to_string(),
            sidebar: rsx!(
                div {
                    for device in devices {
                        DeviceCard {
                            active: matches!(device.role, DeviceRole::Host),
                            device
                        }
                    }
                }
            ),
            stage: rsx!(RemoteStage { model: stage_model }),
            detail: rsx!(SessionPanel { summary: Some(summary), allow_files: true })
        }
    }
}
