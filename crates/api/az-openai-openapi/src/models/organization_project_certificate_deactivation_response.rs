// Generated from OpenAPI spec. Do not edit by hand.
//! `OrganizationProjectCertificateDeactivationResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationProjectCertificate,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationProjectCertificateDeactivationResponse {
    /// The project certificate deactivation result type.
    pub object: String,
    pub data: Vec<OrganizationProjectCertificate>,
}
