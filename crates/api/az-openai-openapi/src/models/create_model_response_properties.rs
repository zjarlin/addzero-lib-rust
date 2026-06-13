// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModelResponseProperties` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
    ServiceTier,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelResponseProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key`
    /// instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost
    /// cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse.
    /// [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// A stable identifier used to help detect users of your application that may be violating OpenAI's
    /// usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length
    /// of 64 characters. We recommend hashing their username or email address, in order to avoid sending us
    /// any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces
    /// the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
}
