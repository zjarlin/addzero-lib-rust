// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogExternalKeyRegistered` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogExternalKeyRegistered {
    /// The ID of the external key configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The configuration for the external key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<OpenAiJsonObject>,
}
