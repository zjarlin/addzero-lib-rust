// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsMessageCreationObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsMessageCreationObjectMessageCreation,
};

/// Details of the message creation by the run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsMessageCreationObject {
    /// Always `message_creation`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_creation: Option<RunStepDeltaStepDetailsMessageCreationObjectMessageCreation>,
}
