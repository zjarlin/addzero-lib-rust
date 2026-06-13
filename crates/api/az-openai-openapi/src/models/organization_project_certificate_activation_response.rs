// Generated from OpenAPI spec. Do not edit by hand.
//! `OrganizationProjectCertificateActivationResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationProjectCertificate,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationProjectCertificateActivationResponse {
    /// The project certificate activation result type.
    pub object: String,
    pub data: Vec<OrganizationProjectCertificate>,
}
