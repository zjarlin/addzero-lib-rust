// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateResponseGATool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPTool,
    RealtimeFunctionTool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseGATool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    MCPTool(MCPTool),
}
