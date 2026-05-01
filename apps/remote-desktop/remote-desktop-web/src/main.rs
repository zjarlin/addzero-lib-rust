use addzero_remote_model::{
    ClipboardPayload, DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform,
    SessionCapability, SessionSummary, VideoCodec, VideoFrameEnvelope,
};
use addzero_remote_session::RemoteRelayService;
use addzero_remote_ui::{
    DeviceCard, REMOTE_STYLE, RemoteActionItem, RemoteActionTone, RemoteShell, RemoteStage,
    RemoteStageViewModel, RemoteStatusChip, SessionPanel,
};
use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

fn main() {
    dioxus::launch(App);
}

fn seed() -> (Vec<DeviceDescriptor>, SessionSummary) {
    let viewer = DeviceDescriptor {
        device_id: Uuid::new_v4(),
        device_name: "Browser Viewer".into(),
        platform: RemotePlatform::Browser,
        role: DeviceRole::Viewer,
        capabilities: SessionCapability::web_viewer(),
        online_status: OnlineStatus::Online,
        last_seen_at: Utc::now(),
        notes: Some("Web 端聚焦观看与轻交互，不暴露桌面端文件传输。".into()),
    };
    let host = DeviceDescriptor {
        device_id: Uuid::new_v4(),
        device_name: "Linux Host".into(),
        platform: RemotePlatform::LinuxWayland,
        role: DeviceRole::Host,
        capabilities: SessionCapability::full_host(),
        online_status: OnlineStatus::Online,
        last_seen_at: Utc::now(),
        notes: Some("Wayland 首版优先保证观看、焦点同步与基础输入。".into()),
    };
    let mut relay = RemoteRelayService::new();
    let viewer = relay.register_device(viewer);
    let host = relay.register_device(host);
    let request = relay
        .request_session(
            viewer.device_id,
            host.device_id,
            SessionCapability::web_viewer(),
        )
        .expect("request");
    relay
        .grant_session(request.session_id, true, None)
        .expect("grant");
    relay
        .push_clipboard(
            request.session_id,
            ClipboardPayload {
                content: "浏览器剪贴板桥已就绪。".into(),
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
                width: 1280,
                height: 720,
                sequence: 8,
                captured_at: Utc::now(),
            },
        )
        .expect("frame");
    (
        relay.list_devices(),
        relay.session_summary(request.session_id).expect("summary"),
    )
}

#[component]
fn App() -> Element {
    let (devices, summary) = seed();
    let selected_host = devices
        .iter()
        .find(|device| matches!(device.role, DeviceRole::Host))
        .cloned();

    let stage_model = RemoteStageViewModel {
        title: selected_host
            .as_ref()
            .map(|device| format!("{} · Web Viewer · 已连接", device.device_name))
            .unwrap_or_else(|| "Web Viewer".to_string()),
        subtitle: "Web 端保留观看与基础控制能力，把文件传输和重度诊断留给桌面端。".to_string(),
        actions: vec![
            RemoteActionItem {
                label: "连接".to_string(),
                tone: RemoteActionTone::Primary,
            },
            RemoteActionItem {
                label: "全屏观看".to_string(),
                tone: RemoteActionTone::Neutral,
            },
            RemoteActionItem {
                label: "断开".to_string(),
                tone: RemoteActionTone::Danger,
            },
        ],
        status_chips: vec![
            RemoteStatusChip {
                label: "观看模式".to_string(),
                emphasis: true,
            },
            RemoteStatusChip {
                label: "文本剪贴板：可用".to_string(),
                emphasis: false,
            },
            RemoteStatusChip {
                label: "文件传输：桌面端处理".to_string(),
                emphasis: false,
            },
        ],
        placeholder_title: "浏览器安全画布".to_string(),
        placeholder_body: "这里强调观看和轻交互体验，不再出现 mock/debug 文案；如果需要重度远控或文件传输，建议切回桌面端控制台。".to_string(),
        permission_notice: None,
    };

    rsx! {
        document::Style { "{REMOTE_STYLE}" }
        RemoteShell {
            title: "Web Viewer".to_string(),
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
            detail: rsx!(SessionPanel { summary: Some(summary), allow_files: false })
        }
    }
}
