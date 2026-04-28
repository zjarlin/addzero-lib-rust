use addzero_remote_host::{HostPlatformAdapter, MockHostPlatformAdapter};

#[test]
fn mock_host_adapter_returns_descriptor() {
    let adapter = MockHostPlatformAdapter;
    let descriptor = adapter.descriptor("host").expect("descriptor");
    assert_eq!(descriptor.device_name, "host");
    assert!(descriptor.capabilities.screen);
}
