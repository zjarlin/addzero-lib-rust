// Generated from OpenAPI spec. Do not edit by hand.
//! `InlineSkillSourceParam` DTO.

use serde::{Deserialize, Serialize};

/// Inline skill payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineSkillSourceParam {
    /// The type of the inline skill source. Must be `base64`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The media type of the inline skill payload. Must be `application/zip`.
    pub media_type: String,
    /// Base64-encoded skill zip bundle.
    pub data: String,
}
