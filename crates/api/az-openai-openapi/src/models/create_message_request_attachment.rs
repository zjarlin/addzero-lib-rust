// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateMessageRequestAttachment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateMessageRequestAttachmentTool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequestAttachment {
    /// The ID of the file to attach to the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The tools to add this file to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<CreateMessageRequestAttachmentTool>>,
}
