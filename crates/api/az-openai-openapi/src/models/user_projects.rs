// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserProjects` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UserProjectsDataItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProjects {
    pub object: String,
    pub data: Vec<UserProjectsDataItem>,
}
