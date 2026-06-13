// Generated from OpenAPI spec. Do not edit by hand.
//! `RunObjectToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunObjectToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
