// Generated from OpenAPI spec. Do not edit by hand.
//! `UserMessageQuotedText` DTO.

use serde::{Deserialize, Serialize};

/// Quoted snippet that the user referenced in their message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageQuotedText {
    /// Type discriminator that is always `quoted_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Quoted text content.
    pub text: String,
}
