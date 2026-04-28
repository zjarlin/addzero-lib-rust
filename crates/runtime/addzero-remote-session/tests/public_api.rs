use addzero_remote_model::{
    ClipboardPayload, DeviceDescriptor, DeviceRole, FileTransferEnvelope, OnlineStatus,
    RemotePlatform, SessionCapability, VideoCodec, VideoFrameEnvelope,
};
use addzero_remote_session::RemoteRelayService;
use chrono::Utc;
use uuid::Uuid;

fn device(name: &str, role: DeviceRole, platform: RemotePlatform) -> DeviceDescriptor {
    DeviceDescriptor {
        device_id: Uuid::new_v4(),
        device_name: name.into(),
        platform,
        role,
        capabilities: SessionCapability::full_host(),
        online_status: OnlineStatus::Offline,
        last_seen_at: Utc::now(),
        notes: None,
    }
}

#[test]
fn relay_service_tracks_session_lifecycle() {
    let mut relay = RemoteRelayService::new();
    let viewer = relay.register_device(device("viewer", DeviceRole::Viewer, RemotePlatform::MacOs));
    let host = relay.register_device(device("host", DeviceRole::Host, RemotePlatform::Windows));

    let request = relay
        .request_session(
            viewer.device_id,
            host.device_id,
            SessionCapability::full_host(),
        )
        .expect("request session");
    relay
        .grant_session(request.session_id, true, None)
        .expect("grant session");

    relay
        .push_clipboard(
            request.session_id,
            ClipboardPayload {
                content: "sync".into(),
                updated_at: Utc::now(),
            },
        )
        .expect("clipboard");
    relay
        .stage_file_transfer(
            request.session_id,
            FileTransferEnvelope {
                transfer_id: Uuid::new_v4(),
                session_id: request.session_id,
                file_name: "demo.txt".into(),
                total_bytes: 128,
                chunk_index: 0,
                chunk_count: 1,
            },
        )
        .expect("file transfer");
    relay
        .push_frame(
            request.session_id,
            VideoFrameEnvelope {
                session_id: request.session_id,
                codec: VideoCodec::JpegFrames,
                width: 1440,
                height: 900,
                sequence: 1,
                captured_at: Utc::now(),
            },
        )
        .expect("video frame");

    let summary = relay.session_summary(request.session_id).expect("summary");
    assert!(summary.clipboard.is_some());
    assert!(summary.pending_transfer.is_some());
    assert!(summary.latest_frame.is_some());
}
