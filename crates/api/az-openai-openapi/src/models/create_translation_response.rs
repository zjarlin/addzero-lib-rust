// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateTranslationResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateTranslationResponseJson,
    CreateTranslationResponseVerboseJson,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTranslationResponse {
    CreateTranslationResponseJson(CreateTranslationResponseJson),
    CreateTranslationResponseVerboseJson(CreateTranslationResponseVerboseJson),
}
