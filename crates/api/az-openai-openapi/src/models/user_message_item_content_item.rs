// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserMessageItemContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UserMessageInputText,
    UserMessageQuotedText,
};

/// Content blocks that comprise a user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageItemContentItem {
    UserMessageInputText(UserMessageInputText),
    UserMessageQuotedText(UserMessageQuotedText),
}
