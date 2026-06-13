// Generated from OpenAPI spec. Do not edit by hand.
//! `ThreadResourceStatus` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ActiveStatus,
    ClosedStatus,
    LockedStatus,
};

/// Current status for the thread. Defaults to `active` for newly created threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThreadResourceStatus {
    ActiveStatus(ActiveStatus),
    LockedStatus(LockedStatus),
    ClosedStatus(ClosedStatus),
}
