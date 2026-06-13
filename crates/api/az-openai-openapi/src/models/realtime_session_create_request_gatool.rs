// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateRequestGATool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPTool,
    RealtimeFunctionTool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestGATool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    MCPTool(MCPTool),
}
