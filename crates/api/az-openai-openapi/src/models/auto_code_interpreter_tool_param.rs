// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AutoCodeInterpreterToolParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AutoCodeInterpreterToolParamNetworkPolicy,
    ContainerMemoryLimit,
};

/// Configuration for a code interpreter container. Optionally specify the IDs of the files to run the
/// code on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCodeInterpreterToolParam {
    /// Always `auto`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// An optional list of uploaded files to make available to your code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<ContainerMemoryLimit>,
    /// Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<AutoCodeInterpreterToolParamNetworkPolicy>,
}
