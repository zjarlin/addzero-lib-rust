// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateRequestTracing2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateRequestTracing2TracingConfiguration,
};

/// Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a
/// session, the configuration cannot be modified. `auto` will create a trace for the session with
/// default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateRequestTracing2TracingConfiguration),
}
