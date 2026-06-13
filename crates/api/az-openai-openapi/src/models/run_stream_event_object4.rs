// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStreamEventObject4` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) moves to a `requires_action` status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject4 {
    pub event: String,
    pub data: RunObject,
}
