use az_remote_model::{
    ClipboardPayload, DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform, SessionCapability,
};
use az_remote_protocol::{ControlFrame, DeviceHello};
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
