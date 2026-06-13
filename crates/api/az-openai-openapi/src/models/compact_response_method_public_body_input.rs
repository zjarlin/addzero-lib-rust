// Generated from OpenAPI spec. Do not edit by hand.
//! `CompactResponseMethodPublicBodyInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputItem,
};

/// Text, image, or file inputs to the model, used to generate a response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompactResponseMethodPublicBodyInput {
    String(String),
    Array(Vec<InputItem>),
}
