// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EffectiveAtParameter` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveAtParameter {
    /// Return only events whose `effective_at` (Unix seconds) is greater than this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gt: Option<i32>,
    /// Return only events whose `effective_at` (Unix seconds) is greater than or equal to this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gte: Option<i32>,
    /// Return only events whose `effective_at` (Unix seconds) is less than this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lt: Option<i32>,
    /// Return only events whose `effective_at` (Unix seconds) is less than or equal to this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lte: Option<i32>,
}
