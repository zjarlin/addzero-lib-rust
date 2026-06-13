// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogActorApiKey` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogActorServiceAccount,
    AuditLogActorUser,
};

/// The API Key used to perform the audit logged action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogActorApiKey {
    /// The tracking id of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of API key. Can be either `user` or `service_account`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<AuditLogActorUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account: Option<AuditLogActorServiceAccount>,
}
