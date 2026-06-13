// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ListCertificatesResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OrganizationCertificate,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCertificatesResponse {
    pub data: Vec<OrganizationCertificate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
    pub object: String,
}
