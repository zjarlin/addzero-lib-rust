// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseInstructions` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputItem,
};

/// A system (or developer) message inserted into the model's context. When using along with
/// `previous_response_id`, the instructions from a previous response will not be carried over to the
/// next response. This makes it simple to swap out system (or developer) messages in new responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseInstructions {
    String(String),
    InputItemList(Vec<InputItem>),
}
