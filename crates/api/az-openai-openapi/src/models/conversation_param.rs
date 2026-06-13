// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ConversationParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ConversationParam2,
};

/// The conversation that this response belongs to. Items from this conversation are prepended to
/// `input_items` for this response request. Input items and output items from this response are
/// automatically added to this conversation after this response completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConversationParam {
    ConversationID(String),
    ConversationParam2(ConversationParam2),
}
