// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateAssistantRequestTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantToolsCode,
    AssistantToolsFileSearch,
    AssistantToolsFunction,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateAssistantRequestTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
