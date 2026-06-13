// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageObjectAttachmentTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantToolsCode,
    AssistantToolsFileSearchTypeOnly,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageObjectAttachmentTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearchTypeOnly(AssistantToolsFileSearchTypeOnly),
}
