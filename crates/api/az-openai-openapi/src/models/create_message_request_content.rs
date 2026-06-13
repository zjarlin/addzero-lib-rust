// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateMessageRequestContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateMessageRequestContentArrayOfContentPart,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestContent {
    TextContent(String),
    ArrayOfContentParts(Vec<CreateMessageRequestContentArrayOfContentPart>),
}
