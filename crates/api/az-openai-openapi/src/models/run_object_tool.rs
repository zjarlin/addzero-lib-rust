// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunObjectTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantToolsCode,
    AssistantToolsFileSearch,
    AssistantToolsFunction,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunObjectTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
