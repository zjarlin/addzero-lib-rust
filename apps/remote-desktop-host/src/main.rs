use addzero_remote_host::{HostPlatformAdapter, MockHostPlatformAdapter};

fn main() {
    let adapter = MockHostPlatformAdapter;
    let descriptor = adapter
        .descriptor("Remote Host Agent")
        .expect("host descriptor");
    println!(
        "host agent online: {} ({})",
        descriptor.device_name, descriptor.platform
    );
    println!("permission hint: {}", adapter.permission_hint());
    if let Some(notes) = descriptor.notes {
        println!("note: {notes}");
    }
}
