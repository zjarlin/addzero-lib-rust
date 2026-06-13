// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogCertificatesActivated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogCertificatesActivatedCertificate,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCertificatesActivated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<AuditLogCertificatesActivatedCertificate>>,
}
