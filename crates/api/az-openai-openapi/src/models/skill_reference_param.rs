// Generated from OpenAPI spec. Do not edit by hand.
//! `SkillReferenceParam` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReferenceParam {
    /// References a skill created with the /v1/skills endpoint.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the referenced skill.
    pub skill_id: String,
    /// Optional skill version. Use a positive integer or 'latest'. Omit for default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
