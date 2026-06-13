// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepStreamEventObject4` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepObject,
};

/// Occurs when a [run step](/docs/api-reference/run-steps/step-object) is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepStreamEventObject4 {
    pub event: String,
    pub data: RunStepObject,
}
