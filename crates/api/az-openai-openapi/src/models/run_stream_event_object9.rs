// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStreamEventObject9` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) is cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject9 {
    pub event: String,
    pub data: RunObject,
}
