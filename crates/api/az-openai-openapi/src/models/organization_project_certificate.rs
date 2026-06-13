// Generated from OpenAPI spec. Do not edit by hand.
//! `OrganizationProjectCertificate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationProjectCertificateCertificateDetails,
};

/// Represents an individual certificate configured at the project level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationProjectCertificate {
    /// The object type, which is always `organization.project.certificate`.
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The Unix timestamp (in seconds) of when the certificate was uploaded.
    pub created_at: i64,
    pub certificate_details: OrganizationProjectCertificateCertificateDetails,
    /// Whether the certificate is currently active at the project level.
    pub active: bool,
}
