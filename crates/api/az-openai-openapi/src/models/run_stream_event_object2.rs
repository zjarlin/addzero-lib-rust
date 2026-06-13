// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStreamEventObject2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) moves to a `queued` status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject2 {
    pub event: String,
    pub data: RunObject,
}
