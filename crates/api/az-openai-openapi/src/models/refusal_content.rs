// Generated from OpenAPI spec. Do not edit by hand.
//! `RefusalContent` DTO.

use serde::{Deserialize, Serialize};

/// A refusal from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalContent {
    /// The type of the refusal. Always `refusal`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The refusal explanation from the model.
    pub refusal: String,
}
