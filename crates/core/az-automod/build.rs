use std::env;
use std::ffi::OsString;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTC");
    println!("cargo::rustc-check-cfg=cfg(az_automod_nightly_tracked_path)");

    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let Ok(output) = Command::new(rustc).arg("--version").output() else {
        return;
    };

    if !output.status.success() {
        return;
    }

    let version = String::from_utf8_lossy(&output.stdout);
    if version.contains("nightly") {
        println!("cargo::rustc-cfg=az_automod_nightly_tracked_path");
    }
}
