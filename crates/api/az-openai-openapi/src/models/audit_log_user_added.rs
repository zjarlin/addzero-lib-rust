// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogUserAdded` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogUserAddedData,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogUserAdded {
    /// The user ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to add the user to the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogUserAddedData>,
}
