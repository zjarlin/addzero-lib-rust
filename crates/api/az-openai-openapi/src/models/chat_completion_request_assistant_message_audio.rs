// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestAssistantMessageAudio` DTO.

use serde::{Deserialize, Serialize};

/// Data about a previous audio response from the model. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestAssistantMessageAudio {
    /// Unique identifier for a previous audio response from the model.
    pub id: String,
}
