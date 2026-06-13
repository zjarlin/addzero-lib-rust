// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateSkillBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateSkillBodyFiles,
};

/// Uploads a skill either as a directory (multipart `files[]`) or as a single zip file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillBody {
    pub files: CreateSkillBodyFiles,
}
