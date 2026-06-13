// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateContainerBodySkill` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InlineSkillParam,
    SkillReferenceParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateContainerBodySkill {
    SkillReferenceParam(SkillReferenceParam),
    InlineSkillParam(InlineSkillParam),
}
