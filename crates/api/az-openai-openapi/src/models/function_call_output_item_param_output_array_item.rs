// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionCallOutputItemParamOutputArrayItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputFileContentParam,
    InputImageContentParamAutoParam,
    InputTextContentParam,
};

/// A piece of message content, such as text, an image, or a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallOutputItemParamOutputArrayItem {
    InputTextContentParam(InputTextContentParam),
    InputImageContentParamAutoParam(InputImageContentParamAutoParam),
    InputFileContentParam(InputFileContentParam),
}
