// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `NamespaceToolParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    NamespaceToolParamTool,
};

/// Groups function/custom tools under a shared namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceToolParam {
    /// The type of the tool. Always `namespace`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The namespace name used in tool calls (for example, `crm`).
    pub name: String,
    /// A description of the namespace shown to the model.
    pub description: String,
    /// The function/custom tools available inside this namespace.
    pub tools: Vec<NamespaceToolParamTool>,
}
