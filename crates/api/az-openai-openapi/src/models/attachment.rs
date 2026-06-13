// Generated from OpenAPI spec. Do not edit by hand.
//! `Attachment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AttachmentType,
};

/// Attachment metadata included on thread items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment discriminator.
    #[serde(rename = "type")]
    pub type_value: AttachmentType,
    /// Identifier for the attachment.
    pub id: String,
    /// Original display name for the attachment.
    pub name: String,
    /// MIME type of the attachment.
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
}
