// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuningJobEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Fine-tuning job event object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJobEvent {
    /// The object type, which is always "fine_tuning.job.event".
    pub object: String,
    /// The object identifier.
    pub id: String,
    /// The Unix timestamp (in seconds) for when the fine-tuning job was created.
    pub created_at: i64,
    /// The log level of the event.
    pub level: String,
    /// The message of the event.
    pub message: String,
    /// The type of event.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The data associated with the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<OpenAiJsonObject>,
}
