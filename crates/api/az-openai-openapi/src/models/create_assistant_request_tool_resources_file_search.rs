// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateAssistantRequestToolResourcesFileSearch` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateAssistantRequestToolResourcesFileSearch {
    Variant1(OpenAiJsonValue),
    Variant2(OpenAiJsonValue),
}
