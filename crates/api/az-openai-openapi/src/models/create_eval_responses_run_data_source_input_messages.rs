// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessages` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2,
};

/// InputMessagesTemplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages {
    /// The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A list of chat messages forming the prompt or context. May include variable references to the `item`
    /// namespace, ie {{item.name}}.
    pub template: Vec<CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2>,
}
