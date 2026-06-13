// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeCallCreateRequestSessionTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPTool,
    RealtimeFunctionTool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeCallCreateRequestSessionTool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    MCPTool(MCPTool),
}
