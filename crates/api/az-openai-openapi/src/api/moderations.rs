// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Moderations REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateModerationRequest,
    CreateModerationResponse,
};

/// Moderations REST endpoints.
#[async_trait]
pub trait OpenAiModerationsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Classifies if text and/or image inputs are potentially harmful. Learn more in the [moderation
    /// guide](/docs/guides/moderation).
    ///
    /// REST: `POST /moderations`.
    /// Path constant: [`OpenAiApiPath::MODERATIONS`](crate::paths::OpenAiApiPath::MODERATIONS).
    async fn create_moderation(
        &self,
        body: CreateModerationRequest,
    ) -> Result<CreateModerationResponse, Self::Error>;
}
