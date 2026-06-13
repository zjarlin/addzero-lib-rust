// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepStreamEventObject6` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepObject,
};

/// Occurs when a [run step](/docs/api-reference/run-steps/step-object) is cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepStreamEventObject6 {
    pub event: String,
    pub data: RunStepObject,
}
