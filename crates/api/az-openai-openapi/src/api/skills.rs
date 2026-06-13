// Generated from OpenAPI spec. Do not edit by hand.
//! Skills REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateSkillBody,
    CreateSkillVersionBody,
    DeletedSkillResource,
    DeletedSkillVersionResource,
    OrderEnum,
    SetDefaultSkillVersionBody,
    SkillListResource,
    SkillResource,
    SkillVersionListResource,
    SkillVersionResource,
};

/// Skills REST endpoints.
#[async_trait]
pub trait OpenAiSkillsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List all skills for the current project.
    ///
    /// REST: `GET /skills`.
    /// Path constant: [`OpenAiApiPath::SKILLS`](crate::paths::OpenAiApiPath::SKILLS).
    async fn list_skills(
        &self,
        limit: Option<i32>,
        order: Option<OrderEnum>,
        after: Option<String>,
    ) -> Result<SkillListResource, Self::Error>;

    /// Create a new skill.
    ///
    /// REST: `POST /skills`.
    /// Path constant: [`OpenAiApiPath::SKILLS`](crate::paths::OpenAiApiPath::SKILLS).
    async fn create_skill(
        &self,
        body: Option<CreateSkillBody>,
    ) -> Result<SkillResource, Self::Error>;

    /// Get a skill by its ID.
    ///
    /// REST: `GET /skills/{skill_id}`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID).
    async fn get_skill(&self, skill_id: String) -> Result<SkillResource, Self::Error>;

    /// Update the default version pointer for a skill.
    ///
    /// REST: `POST /skills/{skill_id}`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID).
    async fn update_skill_default_version(
        &self,
        skill_id: String,
        body: Option<SetDefaultSkillVersionBody>,
    ) -> Result<SkillResource, Self::Error>;

    /// Delete a skill by its ID.
    ///
    /// REST: `DELETE /skills/{skill_id}`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID).
    async fn delete_skill(&self, skill_id: String) -> Result<DeletedSkillResource, Self::Error>;

    /// Download a skill zip bundle by its ID.
    ///
    /// REST: `GET /skills/{skill_id}/content`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_CONTENT`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_CONTENT).
    async fn get_skill_content(&self, skill_id: String) -> Result<String, Self::Error>;

    /// List skill versions for a skill.
    ///
    /// REST: `GET /skills/{skill_id}/versions`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS).
    async fn list_skill_versions(
        &self,
        skill_id: String,
        limit: Option<i32>,
        order: Option<OrderEnum>,
        after: Option<String>,
    ) -> Result<SkillVersionListResource, Self::Error>;

    /// Create a new immutable skill version.
    ///
    /// REST: `POST /skills/{skill_id}/versions`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS).
    async fn create_skill_version(
        &self,
        skill_id: String,
        body: Option<CreateSkillVersionBody>,
    ) -> Result<SkillVersionResource, Self::Error>;

    /// Get a specific skill version.
    ///
    /// REST: `GET /skills/{skill_id}/versions/{version}`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION).
    async fn get_skill_version(
        &self,
        skill_id: String,
        version: String,
    ) -> Result<SkillVersionResource, Self::Error>;

    /// Delete a skill version.
    ///
    /// REST: `DELETE /skills/{skill_id}/versions/{version}`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION).
    async fn delete_skill_version(
        &self,
        skill_id: String,
        version: String,
    ) -> Result<DeletedSkillVersionResource, Self::Error>;

    /// Download a skill version zip bundle.
    ///
    /// REST: `GET /skills/{skill_id}/versions/{version}/content`.
    /// Path constant: [`OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION_BY_CONTENT`](crate::paths::OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION_BY_CONTENT).
    async fn get_skill_version_content(
        &self,
        skill_id: String,
        version: String,
    ) -> Result<String, Self::Error>;
}
