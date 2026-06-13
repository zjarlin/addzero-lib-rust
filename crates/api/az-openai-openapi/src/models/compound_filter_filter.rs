// Generated from OpenAPI spec. Do not edit by hand.
//! `CompoundFilterFilter` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    ComparisonFilter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompoundFilterFilter {
    ComparisonFilter(ComparisonFilter),
    Variant2(OpenAiJsonValue),
}
