// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsMessageCreationObjectMessageCreation` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsMessageCreationObjectMessageCreation {
    /// The ID of the message that was created by this run step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}
