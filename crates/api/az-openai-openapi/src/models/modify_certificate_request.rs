// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ModifyCertificateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyCertificateRequest {
    /// The updated name for the certificate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
