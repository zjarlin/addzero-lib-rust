// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2ChatMessage,
    EvalItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2 {
    ChatMessage(CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2ChatMessage),
    EvalItem(EvalItem),
}
