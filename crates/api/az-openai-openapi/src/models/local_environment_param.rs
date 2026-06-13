// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `LocalEnvironmentParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    LocalSkillParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEnvironmentParam {
    /// Use a local computer environment.
    #[serde(rename = "type")]
    pub type_value: String,
    /// An optional list of skills.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<LocalSkillParam>>,
}
