// Generated from OpenAPI spec. Do not edit by hand.
//! `OrganizationCertificateDeactivationResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationCertificate,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCertificateDeactivationResponse {
    /// The organization certificate deactivation result type.
    pub object: String,
    pub data: Vec<OrganizationCertificate>,
}
