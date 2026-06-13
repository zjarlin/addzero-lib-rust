// Generated from OpenAPI spec. Do not edit by hand.
//! `CustomTextFormatParam` DTO.

use serde::{Deserialize, Serialize};

/// Unconstrained free-form text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTextFormatParam {
    /// Unconstrained text format. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
