// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessages` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessagesTemplateItem,
};

/// TemplateInputMessages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessages {
    /// The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A list of chat messages forming the prompt or context. May include variable references to the `item`
    /// namespace, ie {{item.name}}.
    pub template: Vec<CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessagesTemplateItem>,
}
