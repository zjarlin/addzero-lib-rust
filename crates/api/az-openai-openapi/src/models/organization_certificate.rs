// Generated from OpenAPI spec. Do not edit by hand.
//! `OrganizationCertificate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationCertificateCertificateDetails,
};

/// Represents an individual certificate configured at the organization level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCertificate {
    /// The object type, which is always `organization.certificate`.
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The Unix timestamp (in seconds) of when the certificate was uploaded.
    pub created_at: i64,
    pub certificate_details: OrganizationCertificateCertificateDetails,
    /// Whether the certificate is currently active at the organization level.
    pub active: bool,
}
