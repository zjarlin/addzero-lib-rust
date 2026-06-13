// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationRequestInputArray3Item3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInputArray3Item3Object,
    CreateModerationRequestInputArray3Item3Object2,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateModerationRequestInputArray3Item3 {
    Object(CreateModerationRequestInputArray3Item3Object),
    Object2(CreateModerationRequestInputArray3Item3Object2),
}
