// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `SkillResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResource {
    /// Unique identifier for the skill.
    pub id: String,
    /// The object type, which is `skill`.
    pub object: String,
    /// Name of the skill.
    pub name: String,
    /// Description of the skill.
    pub description: String,
    /// Unix timestamp (seconds) for when the skill was created.
    pub created_at: i64,
    /// Default version for the skill.
    pub default_version: String,
    /// Latest version for the skill.
    pub latest_version: String,
}
