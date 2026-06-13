// Generated from OpenAPI spec. Do not edit by hand.
//! `WorkflowParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WorkflowParamStateVariablesValue,
    WorkflowTracingParam,
};

/// Workflow reference and overrides applied to the chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParam {
    /// Identifier for the workflow invoked by the session.
    pub id: String,
    /// Specific workflow version to run. Defaults to the latest deployed version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// State variables forwarded to the workflow. Keys may be up to 64 characters, values must be primitive
    /// types, and the map defaults to an empty object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_variables: Option<std::collections::BTreeMap<String, WorkflowParamStateVariablesValue>>,
    /// Optional tracing overrides for the workflow invocation. When omitted, tracing is enabled by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: Option<WorkflowTracingParam>,
}
