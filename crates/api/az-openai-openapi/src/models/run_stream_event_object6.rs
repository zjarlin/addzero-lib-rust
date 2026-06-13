// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStreamEventObject6` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) ends with status `incomplete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject6 {
    pub event: String,
    pub data: RunObject,
}
