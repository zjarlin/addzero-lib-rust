// Generated from OpenAPI spec. Do not edit by hand.
//! `SkillVersionResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersionResource {
    /// The object type, which is `skill.version`.
    pub object: String,
    /// Unique identifier for the skill version.
    pub id: String,
    /// Identifier of the skill for this version.
    pub skill_id: String,
    /// Version number for this skill.
    pub version: String,
    /// Unix timestamp (seconds) for when the version was created.
    pub created_at: i64,
    /// Name of the skill version.
    pub name: String,
    /// Description of the skill version.
    pub description: String,
}
