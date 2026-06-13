// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateSkillBodyFiles` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateSkillBodyFiles {
    Array(Vec<OpenAiBinaryBody>),
    String(OpenAiBinaryBody),
}
