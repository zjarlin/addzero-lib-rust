// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateContainerBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateContainerBodyExpiresAfter,
    CreateContainerBodyNetworkPolicy,
    CreateContainerBodySkill,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerBody {
    /// Name of the container to create.
    pub name: String,
    /// IDs of files to copy to the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    /// Container expiration time in seconds relative to the 'anchor' time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<CreateContainerBodyExpiresAfter>,
    /// An optional list of skills referenced by id or inline data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<CreateContainerBodySkill>>,
    /// Optional memory limit for the container. Defaults to "1g".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
    /// Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<CreateContainerBodyNetworkPolicy>,
}
