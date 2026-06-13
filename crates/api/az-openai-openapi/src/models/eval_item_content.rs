// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalItemContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalItemContentArray,
    EvalItemContentItem,
};

/// Inputs to the model - can contain template strings. Supports text, output text, input images, and
/// input audio, either as a single item or an array of items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvalItemContent {
    EvalItemContentItem(EvalItemContentItem),
    EvalItemContentArray(EvalItemContentArray),
}
