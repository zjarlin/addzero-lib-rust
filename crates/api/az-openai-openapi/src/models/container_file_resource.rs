// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ContainerFileResource` DTO.

use serde::{Deserialize, Serialize};

/// The container file object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerFileResource {
    /// Unique identifier for the file.
    pub id: String,
    /// The type of this object (`container.file`).
    pub object: String,
    /// The container this file belongs to.
    pub container_id: String,
    /// Unix timestamp (in seconds) when the file was created.
    pub created_at: i64,
    /// Size of the file in bytes.
    pub bytes: i32,
    /// Path of the file in the container.
    pub path: String,
    /// Source of the file (e.g., `user`, `assistant`).
    pub source: String,
}
