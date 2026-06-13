// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepStreamEventObject3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaObject,
};

/// Occurs when parts of a [run step](/docs/api-reference/run-steps/step-object) are being streamed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepStreamEventObject3 {
    pub event: String,
    pub data: RunStepDeltaObject,
}
