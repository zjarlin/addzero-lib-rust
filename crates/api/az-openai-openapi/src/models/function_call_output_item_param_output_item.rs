// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionCallOutputItemParamOutputItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputFileContentParam,
    InputImageContentParamAutoParam,
    InputTextContentParam,
};

/// A piece of message content, such as text, an image, or a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallOutputItemParamOutputItem {
    InputTextContentParam(InputTextContentParam),
    InputImageContentParamAutoParam(InputImageContentParamAutoParam),
    InputFileContentParam(InputFileContentParam),
}
