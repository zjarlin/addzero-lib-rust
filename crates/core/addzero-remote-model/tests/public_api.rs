use addzero_remote_model::{RemotePlatform, SessionCapability};

#[test]
fn remote_platform_display_is_human_readable() {
    assert_eq!(RemotePlatform::LinuxWayland.to_string(), "Linux (Wayland)");
}

#[test]
fn web_viewer_capability_disables_file_transfer() {
    let capability = SessionCapability::web_viewer();
    assert!(capability.screen);
    assert!(!capability.file_transfer);
}
