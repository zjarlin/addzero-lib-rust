// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepObjectStepDetails` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsMessageCreationObject,
    RunStepDetailsToolCallsObject,
};

/// The details of the run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepObjectStepDetails {
    RunStepDetailsMessageCreationObject(RunStepDetailsMessageCreationObject),
    RunStepDetailsToolCallsObject(RunStepDetailsToolCallsObject),
}
