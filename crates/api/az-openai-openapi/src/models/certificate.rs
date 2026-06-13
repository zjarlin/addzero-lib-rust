// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Certificate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CertificateCertificateDetails,
};

/// Represents an individual `certificate` uploaded to the organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// The object type. - If creating, updating, or getting a specific certificate, the object type is
    /// `certificate`. - If listing, activating, or deactivating certificates for the organization, the
    /// object type is `organization.certificate`. - If listing, activating, or deactivating certificates
    /// for a project, the object type is `organization.project.certificate`.
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The Unix timestamp (in seconds) of when the certificate was uploaded.
    pub created_at: i64,
    pub certificate_details: CertificateCertificateDetails,
    /// Whether the certificate is currently active at the specified scope. Not returned when getting
    /// details for a specific certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}
