// Generated from OpenAPI spec. Do not edit by hand.
//! `UploadCertificateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCertificateRequest {
    /// An optional name for the certificate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The certificate content in PEM format
    pub certificate: String,
}
