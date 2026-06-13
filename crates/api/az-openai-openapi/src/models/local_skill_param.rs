// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `LocalSkillParam` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkillParam {
    /// The name of the skill.
    pub name: String,
    /// The description of the skill.
    pub description: String,
    /// The path to the directory containing the skill.
    pub path: String,
}
