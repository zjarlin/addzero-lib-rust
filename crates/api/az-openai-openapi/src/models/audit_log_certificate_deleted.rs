// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogCertificateDeleted` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCertificateDeleted {
    /// The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The certificate content in PEM format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
}
