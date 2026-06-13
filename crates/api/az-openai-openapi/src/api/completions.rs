// Generated from OpenAPI spec. Do not edit by hand.
//! Completions REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateCompletionRequest,
    CreateCompletionResponse,
};

/// Completions REST endpoints.
#[async_trait]
pub trait OpenAiCompletionsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Creates a completion for the provided prompt and parameters. Returns a completion object, or a
    /// sequence of completion objects if the request is streamed.
    ///
    /// REST: `POST /completions`.
    /// Path constant: [`OpenAiApiPath::COMPLETIONS`](crate::paths::OpenAiApiPath::COMPLETIONS).
    async fn create_completion(
        &self,
        body: CreateCompletionRequest,
    ) -> Result<CreateCompletionResponse, Self::Error>;
}
