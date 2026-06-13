// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsMessageCreationObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsMessageCreationObjectMessageCreation,
};

/// Details of the message creation by the run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsMessageCreationObject {
    /// Always `message_creation`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub message_creation: RunStepDetailsMessageCreationObjectMessageCreation,
}
