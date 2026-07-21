use az_remote_model::contract::{
    DeviceDescriptor, DeviceRole, KeyState, OnlineStatus, PointerButton, RemoteInputEvent,
    RemotePlatform, SessionCapability, SessionState, VideoCodec,
};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn remote_platform_display_is_human_readable() {
    assert_eq!(RemotePlatform::LinuxWayland.to_string(), "Linux (Wayland)");
}

#[test]
fn remote_platform_code_is_machine_readable() {
    assert_eq!(RemotePlatform::LinuxWayland.code(), "linux_wayland");
    assert_eq!(
        serde_json::to_string(&RemotePlatform::MacOs).expect("serialize"),
        "\"mac_os\""
    );
    assert_eq!(
        RemotePlatform::from_code("linux_x11"),
        Some(RemotePlatform::LinuxX11)
    );
}

#[test]
fn device_role_code_is_machine_readable() {
    assert_eq!(DeviceRole::Host.code(), "host");
}

#[test]
fn online_status_from_code_reads_protocol_code() {
    assert_eq!(
        OnlineStatus::from_code("offline"),
        Some(OnlineStatus::Offline)
    );
}

#[test]
fn pointer_button_all_lists_supported_buttons() {
    assert_eq!(
        PointerButton::ALL,
        &[
            PointerButton::Left,
            PointerButton::Middle,
            PointerButton::Right
        ]
    );
}

#[test]
fn key_state_serde_uses_snake_case_protocol_code() {
    assert_eq!(
        serde_json::to_string(&KeyState::Down).expect("serialize key state"),
        "\"down\""
    );
}

#[test]
fn video_codec_code_uses_snake_case_protocol_code() {
    assert_eq!(VideoCodec::JpegFrames.code(), "jpeg_frames");
}

#[test]
fn session_state_from_code_reads_protocol_code() {
    assert_eq!(
        SessionState::from_code("active"),
        Some(SessionState::Active)
    );
}

#[test]
fn device_descriptor_serializes_code_enums_as_snake_case() {
    let descriptor = DeviceDescriptor {
        device_id: Uuid::nil(),
        device_name: "demo".to_owned(),
        platform: RemotePlatform::MacOs,
        role: DeviceRole::Host,
        capabilities: SessionCapability::full_host(),
        online_status: OnlineStatus::Online,
        last_seen_at: Utc::now(),
        notes: None,
    };

    let value = serde_json::to_value(descriptor).expect("serialize device descriptor");

    assert_eq!(value["role"], "host");
    assert_eq!(value["online_status"], "online");
}

#[test]
fn remote_input_event_serializes_nested_code_enums_as_snake_case() {
    let event = RemoteInputEvent::PointerButton {
        button: PointerButton::Left,
        state: KeyState::Down,
    };

    let value = serde_json::to_value(event).expect("serialize remote input event");

    assert_eq!(value["PointerButton"]["button"], "left");
    assert_eq!(value["PointerButton"]["state"], "down");
}

#[test]
fn web_viewer_capability_disables_file_transfer() {
    let capability = SessionCapability::web_viewer();
    assert!(capability.screen);
    assert!(!capability.file_transfer);
}
