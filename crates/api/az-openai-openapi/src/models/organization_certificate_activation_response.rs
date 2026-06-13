// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `OrganizationCertificateActivationResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationCertificate,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCertificateActivationResponse {
    /// The organization certificate activation result type.
    pub object: String,
    pub data: Vec<OrganizationCertificate>,
}
