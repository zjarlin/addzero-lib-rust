// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogIpAllowlistConfigActivated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogIpAllowlistConfigActivatedConfig,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistConfigActivated {
    /// The configurations that were activated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configs: Option<Vec<AuditLogIpAllowlistConfigActivatedConfig>>,
}
