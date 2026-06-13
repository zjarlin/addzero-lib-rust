// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogIpAllowlistConfigDeactivated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogIpAllowlistConfigDeactivatedConfig,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistConfigDeactivated {
    /// The configurations that were deactivated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configs: Option<Vec<AuditLogIpAllowlistConfigDeactivatedConfig>>,
}
