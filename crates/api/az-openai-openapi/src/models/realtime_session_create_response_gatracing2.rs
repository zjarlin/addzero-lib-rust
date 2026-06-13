// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponseGATracing2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseGATracing2TracingConfiguration,
};

/// Realtime API can write session traces to the [Traces
/// Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once
/// tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace
/// for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseGATracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateResponseGATracing2TracingConfiguration),
}
