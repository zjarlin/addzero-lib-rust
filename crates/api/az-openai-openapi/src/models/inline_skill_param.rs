// Generated from OpenAPI spec. Do not edit by hand.
//! `InlineSkillParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InlineSkillSourceParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineSkillParam {
    /// Defines an inline skill for this request.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the skill.
    pub name: String,
    /// The description of the skill.
    pub description: String,
    /// Inline skill payload
    pub source: InlineSkillSourceParam,
}
