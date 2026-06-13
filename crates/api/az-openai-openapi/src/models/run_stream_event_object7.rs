// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStreamEventObject7` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject7 {
    pub event: String,
    pub data: RunObject,
}
