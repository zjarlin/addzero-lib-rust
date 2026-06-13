// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MCPToolAllowedTools` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPToolFilter,
};

/// List of allowed tool names or a filter object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPToolAllowedTools {
    MCPAllowedTools(Vec<String>),
    MCPToolFilter(MCPToolFilter),
}
