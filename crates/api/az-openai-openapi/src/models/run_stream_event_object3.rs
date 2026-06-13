// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStreamEventObject3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) moves to an `in_progress` status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject3 {
    pub event: String,
    pub data: RunObject,
}
