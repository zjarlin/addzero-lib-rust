// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ContainerAutoParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerAutoParamNetworkPolicy,
    ContainerAutoParamSkill,
    ContainerMemoryLimit,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerAutoParam {
    /// Automatically creates a container for this request
    #[serde(rename = "type")]
    pub type_value: String,
    /// An optional list of uploaded files to make available to your code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<ContainerMemoryLimit>,
    /// Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<ContainerAutoParamNetworkPolicy>,
    /// An optional list of skills referenced by id or inline data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<ContainerAutoParamSkill>>,
}
