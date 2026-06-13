// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ContainerResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerResourceExpiresAfter,
    ContainerResourceNetworkPolicy,
};

/// The container object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResource {
    /// Unique identifier for the container.
    pub id: String,
    /// The type of this object.
    pub object: String,
    /// Name of the container.
    pub name: String,
    /// Unix timestamp (in seconds) when the container was created.
    pub created_at: i64,
    /// Status of the container (e.g., active, deleted).
    pub status: String,
    /// Unix timestamp (in seconds) when the container was last active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
    /// The container will expire after this time period. The anchor is the reference point for the
    /// expiration. The minutes is the number of minutes after the anchor before the container expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<ContainerResourceExpiresAfter>,
    /// The memory limit configured for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
    /// Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<ContainerResourceNetworkPolicy>,
}
