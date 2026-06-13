// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationRequestInputItem3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInputItem3Object,
    CreateModerationRequestInputItem3Object2,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateModerationRequestInputItem3 {
    Object(CreateModerationRequestInputItem3Object),
    Object2(CreateModerationRequestInputItem3Object2),
}
