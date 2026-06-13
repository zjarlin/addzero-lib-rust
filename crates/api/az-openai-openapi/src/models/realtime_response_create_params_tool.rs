// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseCreateParamsTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPTool,
    RealtimeFunctionTool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseCreateParamsTool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    MCPTool(MCPTool),
}
