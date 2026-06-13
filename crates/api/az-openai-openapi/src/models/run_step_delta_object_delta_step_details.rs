// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaObjectDeltaStepDetails` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsMessageCreationObject,
    RunStepDeltaStepDetailsToolCallsObject,
};

/// The details of the run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepDeltaObjectDeltaStepDetails {
    RunStepDeltaStepDetailsMessageCreationObject(RunStepDeltaStepDetailsMessageCreationObject),
    RunStepDeltaStepDetailsToolCallsObject(RunStepDeltaStepDetailsToolCallsObject),
}
