#![forbid(unsafe_code)]

pub mod plugin_api;
pub mod plugin_host;
pub mod config;
pub mod db;
pub mod di;
pub mod state;

/// Link all native plugins into the binary via inventory.
/// Call this once at startup in the host binary.
pub fn link_plugins() {
    // Each plugin crate registers via inventory::submit!.
    // This function is intentionally empty — the side effect is
    // ensuring plugin crates are linked at compile time.
}
