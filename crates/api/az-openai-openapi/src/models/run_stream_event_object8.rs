// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStreamEventObject8` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) moves to a `cancelling` status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject8 {
    pub event: String,
    pub data: RunObject,
}
