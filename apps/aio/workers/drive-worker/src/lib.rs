#![forbid(unsafe_code)]

//! Drive CRDT sync worker providing WebSocket-based line-CRDT text sync
//! for the workbench.

automod::dir!(pub "src");

pub use sync_msg::CrdtSyncMsg;
pub use sync_state::{DriveSyncState, handle_drive_sync};
