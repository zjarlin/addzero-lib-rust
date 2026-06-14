//! Shared application state.

use crate::plugin_host::HostSnapshot;

/// Holds the plugin snapshot, consumed by route handlers.
#[derive(Clone)]
pub struct AppState {
    pub snapshot: HostSnapshot,
}

impl AppState {
    pub fn new(snapshot: HostSnapshot) -> Self {
        Self { snapshot }
    }
}
