// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceInputMessagesTemplateItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EasyInputMessage,
    EvalItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceInputMessagesTemplateItem {
    EasyInputMessage(EasyInputMessage),
    EvalItem(EvalItem),
}
