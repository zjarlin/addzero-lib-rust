// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageRefParam2` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRefParam2 {
    /// A fully qualified URL or base64-encoded data URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
