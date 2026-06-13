// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateSkillVersionBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateSkillVersionBodyFiles,
};

/// Uploads a new immutable version of a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillVersionBody {
    pub files: CreateSkillVersionBodyFiles,
    /// Whether to set this version as the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}
