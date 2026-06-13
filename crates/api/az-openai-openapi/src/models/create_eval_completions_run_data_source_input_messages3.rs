// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceInputMessages3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalCompletionsRunDataSourceInputMessages3ItemReferenceInputMessages,
    CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessages,
};

/// Used when sampling from a model. Dictates the structure of the messages passed into the model. Can
/// either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with
/// variable references to the `item` namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceInputMessages3 {
    TemplateInputMessages(CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessages),
    ItemReferenceInputMessages(CreateEvalCompletionsRunDataSourceInputMessages3ItemReferenceInputMessages),
}
