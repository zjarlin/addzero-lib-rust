// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ThreadResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ThreadResourceStatus,
};

/// Represents a ChatKit thread and its current status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResource {
    /// Identifier of the thread.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the thread was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Current status for the thread. Defaults to `active` for newly created threads.
    pub status: ThreadResourceStatus,
    /// Free-form string that identifies your end user who owns the thread.
    pub user: String,
}
