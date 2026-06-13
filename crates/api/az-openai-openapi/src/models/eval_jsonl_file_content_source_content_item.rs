// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalJsonlFileContentSourceContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalJsonlFileContentSourceContentItem {
    pub item: OpenAiJsonObject,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: Option<OpenAiJsonObject>,
}
