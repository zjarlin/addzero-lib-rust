// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsMessageCreationObjectMessageCreation` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsMessageCreationObjectMessageCreation {
    /// The ID of the message that was created by this run step.
    pub message_id: String,
}
