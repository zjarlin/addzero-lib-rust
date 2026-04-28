use addzero_remote_model::{
    ClipboardPayload, DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform,
    SessionCapability, SessionSummary, VideoCodec, VideoFrameEnvelope,
};
use addzero_remote_session::RemoteRelayService;
use addzero_remote_ui::{DeviceCard, REMOTE_STYLE, RemoteShell, SessionPanel};
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
        notes: Some("Web 端不支持文件传输。".into()),
    };
    let host = DeviceDescriptor {
        device_id: Uuid::new_v4(),
        device_name: "Linux Host".into(),
        platform: RemotePlatform::LinuxWayland,
        role: DeviceRole::Host,
        capabilities: SessionCapability::full_host(),
        online_status: OnlineStatus::Online,
        last_seen_at: Utc::now(),
        notes: Some("Wayland 受限，首版仅保证观看与基础控制。".into()),
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
                content: "web clipboard bridge ready".into(),
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
            stage: rsx!(
                section { class: "rd-stage__surface",
                    div { class: "toolbar",
                        button { "Connect" }
                        input { value: "Text clipboard only" }
                    }
                    div { class: "stage-screen", "Browser-safe remote stage placeholder." }
                    p { class: "muted", "This viewer intentionally omits file transfer in v1." }
                }
            ),
            detail: rsx!(SessionPanel { summary: Some(summary), allow_files: false })
        }
    }
}
