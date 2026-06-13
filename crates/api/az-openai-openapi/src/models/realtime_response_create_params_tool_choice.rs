// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeResponseCreateParamsToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ToolChoiceFunction,
    ToolChoiceMCP,
    ToolChoiceOptions,
};

/// How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseCreateParamsToolChoice {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMCP(ToolChoiceMCP),
}
