// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VideoCharacterResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCharacterResource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Unix timestamp (in seconds) when the character was created.
    pub created_at: i64,
}
