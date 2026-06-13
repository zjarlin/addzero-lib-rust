// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreSearchRequestFilters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComparisonFilter,
    CompoundFilter,
};

/// A filter to apply based on file attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorStoreSearchRequestFilters {
    ComparisonFilter(ComparisonFilter),
    CompoundFilter(CompoundFilter),
}
