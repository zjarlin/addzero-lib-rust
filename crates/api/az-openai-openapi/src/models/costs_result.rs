// Generated from OpenAPI spec. Do not edit by hand.
//! `CostsResult` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CostsResultAmount,
};

/// The aggregated costs details of the specific time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostsResult {
    pub object: String,
    /// The monetary value in its associated currency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<CostsResultAmount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_item: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
}
