// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2,
};

/// InputMessagesTemplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplate {
    /// The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A list of chat messages forming the prompt or context. May include variable references to the `item`
    /// namespace, ie {{item.name}}.
    pub template: Vec<CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2>,
}
