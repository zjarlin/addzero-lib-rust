use az_remote_model::contract::{
    ClipboardPayload, DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform, SessionCapability,
};
use az_remote_protocol::contract::{ControlFrame, DeviceHello, StreamKind};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn control_frame_round_trips_as_json() {
    let hello = DeviceHello {
        device: DeviceDescriptor {
            device_id: Uuid::new_v4(),
            device_name: "demo-host".into(),
            platform: RemotePlatform::MacOs,
            role: DeviceRole::Host,
            capabilities: SessionCapability::full_host(),
            online_status: OnlineStatus::Online,
            last_seen_at: Utc::now(),
            notes: None,
        },
        relay_token: "token-1".into(),
    };

    let debug = format!("{hello:?}");
    assert!(debug.contains("demo-host"));
    assert!(!debug.contains("token-1"));

    let frame = ControlFrame::Hello(hello);

    let decoded =
        ControlFrame::from_json_bytes(&frame.to_json_bytes().expect("encode")).expect("decode");
    assert_eq!(decoded, frame);
}

#[test]
fn clipboard_frame_is_serializable() {
    let frame = ControlFrame::ClipboardSync(ClipboardPayload {
        content: "hello".into(),
        updated_at: Utc::now(),
    });
    assert!(!frame.to_json_bytes().expect("encode").is_empty());
}

#[test]
fn stream_kind_code_is_snake_case() {
    assert_eq!(StreamKind::Clipboard.code(), "clipboard");
}

#[test]
fn stream_kind_all_lists_supported_streams() {
    assert_eq!(
        StreamKind::ALL,
        &[
            StreamKind::Control,
            StreamKind::Video,
            StreamKind::Input,
            StreamKind::Clipboard,
            StreamKind::File,
        ]
    );
}

#[test]
fn stream_kind_serde_uses_snake_case_protocol_code() {
    assert_eq!(
        serde_json::to_string(&StreamKind::File).expect("serialize stream kind"),
        "\"file\""
    );
}
