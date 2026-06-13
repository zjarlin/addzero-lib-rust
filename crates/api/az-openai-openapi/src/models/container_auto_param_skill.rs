// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ContainerAutoParamSkill` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InlineSkillParam,
    SkillReferenceParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContainerAutoParamSkill {
    SkillReferenceParam(SkillReferenceParam),
    InlineSkillParam(InlineSkillParam),
}
