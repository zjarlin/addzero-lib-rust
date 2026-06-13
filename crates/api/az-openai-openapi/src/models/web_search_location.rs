// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchLocation` DTO.

use serde::{Deserialize, Serialize};

/// Approximate location parameters for the search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchLocation {
    /// The two-letter [ISO country code](https://en.wikipedia.org/wiki/ISO_3166-1) of the user, e.g. `US`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Free text input for the region of the user, e.g. `California`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Free text input for the city of the user, e.g. `San Francisco`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// The [IANA timezone](https://timeapi.io/documentation/iana-timezones) of the user, e.g.
    /// `America/Los_Angeles`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}
