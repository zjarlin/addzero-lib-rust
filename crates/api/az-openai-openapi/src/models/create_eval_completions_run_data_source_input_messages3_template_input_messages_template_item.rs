// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessagesTemplateItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EasyInputMessage,
    EvalItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessagesTemplateItem {
    EasyInputMessage(EasyInputMessage),
    EvalItem(EvalItem),
}
