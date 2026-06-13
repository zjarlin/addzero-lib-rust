// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateMessageRequestAttachmentTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantToolsCode,
    AssistantToolsFileSearchTypeOnly,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestAttachmentTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearchTypeOnly(AssistantToolsFileSearchTypeOnly),
}
