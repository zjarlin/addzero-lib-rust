// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneChatCompletionRequestAssistantMessageAudio` DTO.

use serde::{Deserialize, Serialize};

/// Data about a previous audio response from the model. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneChatCompletionRequestAssistantMessageAudio {
    /// Unique identifier for a previous audio response from the model.
    pub id: String,
}
