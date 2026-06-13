// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateSkillVersionBodyFiles` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateSkillVersionBodyFiles {
    Array(Vec<OpenAiBinaryBody>),
    String(OpenAiBinaryBody),
}
