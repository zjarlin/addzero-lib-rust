// Generated from OpenAPI spec. Do not edit by hand.
//! `SetDefaultSkillVersionBody` DTO.

use serde::{Deserialize, Serialize};

/// Updates the default version pointer for a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDefaultSkillVersionBody {
    /// The skill version number to set as default.
    pub default_version: String,
}
