// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Usage REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    UsageResponse,
};

/// Usage REST endpoints.
#[async_trait]
pub trait OpenAiUsageApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get costs details for the organization.
    ///
    /// REST: `GET /organization/costs`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_COSTS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_COSTS).
    async fn usage_costs(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get audio speeches usage details for the organization.
    ///
    /// REST: `GET /organization/usage/audio_speeches`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_AUDIO_SPEECHES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_AUDIO_SPEECHES).
    async fn usage_audio_speeches(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        models: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get audio transcriptions usage details for the organization.
    ///
    /// REST: `GET /organization/usage/audio_transcriptions`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_AUDIO_TRANSCRIPTIONS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_AUDIO_TRANSCRIPTIONS).
    async fn usage_audio_transcriptions(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        models: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get code interpreter sessions usage details for the organization.
    ///
    /// REST: `GET /organization/usage/code_interpreter_sessions`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_CODE_INTERPRETER_SESSIONS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_CODE_INTERPRETER_SESSIONS).
    async fn usage_code_interpreter_sessions(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get completions usage details for the organization.
    ///
    /// REST: `GET /organization/usage/completions`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_COMPLETIONS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_COMPLETIONS).
    async fn usage_completions(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        models: Option<Vec<String>>,
        batch: Option<bool>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get embeddings usage details for the organization.
    ///
    /// REST: `GET /organization/usage/embeddings`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_EMBEDDINGS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_EMBEDDINGS).
    async fn usage_embeddings(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        models: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get images usage details for the organization.
    ///
    /// REST: `GET /organization/usage/images`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_IMAGES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_IMAGES).
    async fn usage_images(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        sources: Option<Vec<String>>,
        sizes: Option<Vec<String>>,
        project_ids: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        models: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get moderations usage details for the organization.
    ///
    /// REST: `GET /organization/usage/moderations`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_MODERATIONS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_MODERATIONS).
    async fn usage_moderations(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        api_key_ids: Option<Vec<String>>,
        models: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;

    /// Get vector stores usage details for the organization.
    ///
    /// REST: `GET /organization/usage/vector_stores`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_VECTOR_STORES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_VECTOR_STORES).
    async fn usage_vector_stores(
        &self,
        start_time: i32,
        end_time: Option<i32>,
        bucket_width: Option<String>,
        project_ids: Option<Vec<String>>,
        group_by: Option<Vec<String>>,
        limit: Option<i32>,
        page: Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
}
