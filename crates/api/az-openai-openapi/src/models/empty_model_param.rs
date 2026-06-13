// Generated from OpenAPI spec. Do not edit by hand.
//! `EmptyModelParam` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyModelParam {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
