// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateThreadAndRunRequestToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThreadAndRunRequestToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
