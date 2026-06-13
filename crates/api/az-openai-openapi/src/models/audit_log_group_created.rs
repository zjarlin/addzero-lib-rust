// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogGroupCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogGroupCreatedData,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogGroupCreated {
    /// The ID of the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Information about the created group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogGroupCreatedData>,
}
