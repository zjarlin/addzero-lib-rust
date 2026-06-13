// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CertificateCertificateDetails` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateCertificateDetails {
    /// The Unix timestamp (in seconds) of when the certificate becomes valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_at: Option<i64>,
    /// The Unix timestamp (in seconds) of when the certificate expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// The content of the certificate in PEM format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
