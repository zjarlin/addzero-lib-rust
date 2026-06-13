// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessages3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceInputMessages3InputMessagesItemReference,
    CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplate,
};

/// Used when sampling from a model. Dictates the structure of the messages passed into the model. Can
/// either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with
/// variable references to the `item` namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceInputMessages3 {
    InputMessagesTemplate(CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplate),
    InputMessagesItemReference(CreateEvalResponsesRunDataSourceInputMessages3InputMessagesItemReference),
}
