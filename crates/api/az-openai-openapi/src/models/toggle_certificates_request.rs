// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToggleCertificatesRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleCertificatesRequest {
    pub certificate_ids: Vec<String>,
}
