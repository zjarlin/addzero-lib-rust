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
use addzero_remote_ui::{DeviceCard, REMOTE_STYLE, RemoteShell, SessionPanel};
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
        notes: Some("Desktop viewer with file transfer.".into()),
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
                content: "Build completed on relay node.".into(),
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
                chunk_index: 0,
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
            stage: rsx!(
                section { class: "rd-stage__surface",
                    div { class: "toolbar",
                        button { "Connect" }
                        button { "Disconnect" }
                        input { value: "Clipboard sync enabled" }
                    }
                    div { class: "stage-screen", "Mock remote desktop canvas · input/file transfer wired at service level." }
                    p { class: "muted", "Host permission hint: {MockHostPlatformAdapter.permission_hint()}" }
                }
            ),
            detail: rsx!(SessionPanel { summary: Some(summary), allow_files: true })
        }
    }
}
