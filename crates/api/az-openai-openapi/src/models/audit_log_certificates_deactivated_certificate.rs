// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogCertificatesDeactivatedCertificate` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCertificatesDeactivatedCertificate {
    /// The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
