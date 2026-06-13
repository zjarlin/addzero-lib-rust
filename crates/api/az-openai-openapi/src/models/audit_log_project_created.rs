// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogProjectCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogProjectCreatedData,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogProjectCreated {
    /// The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to create the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogProjectCreatedData>,
}
