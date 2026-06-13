// Generated from OpenAPI spec. Do not edit by hand.
//! `PredictionContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    PredictionContentContent,
};

/// Static predicted output content, such as the content of a text file that is being regenerated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionContent {
    /// The type of the predicted content you want to provide. This type is currently always `content`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The content that should be matched when generating a model response. If generated tokens would match
    /// this content, the entire model response can be returned much more quickly.
    pub content: PredictionContentContent,
}
