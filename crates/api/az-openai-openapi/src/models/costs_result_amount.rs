// Generated from OpenAPI spec. Do not edit by hand.
//! `CostsResultAmount` DTO.

use serde::{Deserialize, Serialize};

/// The monetary value in its associated currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostsResultAmount {
    /// The numeric value of the cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Lowercase ISO-4217 currency e.g. "usd"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}
