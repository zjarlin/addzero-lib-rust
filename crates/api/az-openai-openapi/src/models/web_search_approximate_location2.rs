// Generated from OpenAPI spec. Do not edit by hand.
//! `WebSearchApproximateLocation2` DTO.

use serde::{Deserialize, Serialize};

/// The approximate location of the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchApproximateLocation2 {
    /// The type of location approximation. Always `approximate`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}
