// Generated from OpenAPI spec. Do not edit by hand.
//! `DeleteCertificateResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCertificateResponse {
    /// The object type, must be `certificate.deleted`.
    pub object: String,
    /// The ID of the certificate that was deleted.
    pub id: String,
}
