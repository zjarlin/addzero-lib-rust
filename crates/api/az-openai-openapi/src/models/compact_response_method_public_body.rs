// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CompactResponseMethodPublicBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CompactResponseMethodPublicBodyInput,
    ModelIdsCompaction,
    PromptCacheRetentionEnum,
    ServiceTierEnum,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResponseMethodPublicBody {
    pub model: ModelIdsCompaction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<CompactResponseMethodPublicBodyInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<PromptCacheRetentionEnum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTierEnum>,
}
