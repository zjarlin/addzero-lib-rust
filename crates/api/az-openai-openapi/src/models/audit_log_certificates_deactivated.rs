// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogCertificatesDeactivated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogCertificatesDeactivatedCertificate,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCertificatesDeactivated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<AuditLogCertificatesDeactivatedCertificate>>,
}
