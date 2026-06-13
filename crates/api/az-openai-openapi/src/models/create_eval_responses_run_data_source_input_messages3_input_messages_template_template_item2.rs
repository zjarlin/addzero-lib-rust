// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2ChatMessage,
    EvalItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2 {
    ChatMessage(CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2ChatMessage),
    EvalItem(EvalItem),
}
