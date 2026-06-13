// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepStreamEventObject2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepObject,
};

/// Occurs when a [run step](/docs/api-reference/run-steps/step-object) moves to an `in_progress` state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepStreamEventObject2 {
    pub event: String,
    pub data: RunStepObject,
}
