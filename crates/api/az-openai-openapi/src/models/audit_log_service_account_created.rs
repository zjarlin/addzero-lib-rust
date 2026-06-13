// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogServiceAccountCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogServiceAccountCreatedData,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogServiceAccountCreated {
    /// The service account ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to create the service account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogServiceAccountCreatedData>,
}
