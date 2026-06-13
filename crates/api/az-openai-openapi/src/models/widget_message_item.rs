// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WidgetMessageItem` DTO.

use serde::{Deserialize, Serialize};

/// Thread item that renders a widget payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMessageItem {
    /// Identifier of the thread item.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    /// Identifier of the parent thread.
    pub thread_id: String,
    /// Type discriminator that is always `chatkit.widget`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Serialized widget payload rendered in the UI.
    pub widget: String,
}
