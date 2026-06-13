// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateThreadRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateMessageRequest,
    CreateThreadRequestToolResources,
    Metadata,
};

/// Options to create a new thread. If no thread is provided when running a request, an empty thread
/// will be created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThreadRequest {
    /// A list of [messages](/docs/api-reference/messages) to start the thread with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<CreateMessageRequest>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<CreateThreadRequestToolResources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
