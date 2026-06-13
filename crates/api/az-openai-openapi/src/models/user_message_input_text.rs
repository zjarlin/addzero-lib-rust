// Generated from OpenAPI spec. Do not edit by hand.
//! `UserMessageInputText` DTO.

use serde::{Deserialize, Serialize};

/// Text block that a user contributed to the thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageInputText {
    /// Type discriminator that is always `input_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Plain-text content supplied by the user.
    pub text: String,
}
