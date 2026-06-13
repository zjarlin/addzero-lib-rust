// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionTracing2TracingConfiguration` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Granular configuration for tracing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionTracing2TracingConfiguration {
    /// The name of the workflow to attach to this trace. This is used to name the trace in the traces
    /// dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    /// The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiJsonObject>,
}
