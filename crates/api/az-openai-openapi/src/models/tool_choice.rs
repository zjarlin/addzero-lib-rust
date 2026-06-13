// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolChoice` DTO.

use serde::{Deserialize, Serialize};

/// Tool selection that the assistant should honor when executing the item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoice {
    /// Identifier of the requested tool.
    pub id: String,
}
