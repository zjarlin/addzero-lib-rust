// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStreamEventObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a new [run](/docs/api-reference/runs/object) is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject {
    pub event: String,
    pub data: RunObject,
}
