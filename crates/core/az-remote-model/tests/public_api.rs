use az_remote_model::{RemotePlatform, SessionCapability};

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
fn web_viewer_capability_disables_file_transfer() {
    let capability = SessionCapability::web_viewer();
    assert!(capability.screen);
    assert!(!capability.file_transfer);
}
