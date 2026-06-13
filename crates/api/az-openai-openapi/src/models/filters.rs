// Generated from OpenAPI spec. Do not edit by hand.
//! `Filters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComparisonFilter,
    CompoundFilter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filters {
    ComparisonFilter(ComparisonFilter),
    CompoundFilter(CompoundFilter),
}
