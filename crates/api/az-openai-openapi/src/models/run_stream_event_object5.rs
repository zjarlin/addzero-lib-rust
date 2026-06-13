// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStreamEventObject5` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject5 {
    pub event: String,
    pub data: RunObject,
}
