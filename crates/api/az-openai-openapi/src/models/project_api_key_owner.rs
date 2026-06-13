// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectApiKeyOwner` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectApiKeyOwnerServiceAccount,
    ProjectApiKeyOwnerUser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKeyOwner {
    /// `user` or `service_account`
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<ProjectApiKeyOwnerUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account: Option<ProjectApiKeyOwnerServiceAccount>,
}
