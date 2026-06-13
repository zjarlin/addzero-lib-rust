// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerResourceExpiresAfter` DTO.

use serde::{Deserialize, Serialize};

/// The container will expire after this time period. The anchor is the reference point for the
/// expiration. The minutes is the number of minutes after the anchor before the container expires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResourceExpiresAfter {
    /// The reference point for the expiration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// The number of minutes after the anchor before the container expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: Option<i32>,
}
