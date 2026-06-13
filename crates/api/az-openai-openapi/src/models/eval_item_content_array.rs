// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalItemContentArray` DTO.

use crate::models::{
    EvalItemContentItem,
};

/// A list of inputs, each of which may be either an input text, output text, input image, or input
/// audio object.
pub type EvalItemContentArray = Vec<EvalItemContentItem>;
