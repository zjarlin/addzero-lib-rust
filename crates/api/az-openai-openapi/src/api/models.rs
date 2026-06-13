// Generated from OpenAPI spec. Do not edit by hand.
//! Models REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    DeleteModelResponse,
    ListModelsResponse,
    Model,
};

/// Models REST endpoints.
#[async_trait]
pub trait OpenAiModelsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Lists the currently available models, and provides basic information about each one such as the
    /// owner and availability.
    ///
    /// REST: `GET /models`.
    /// Path constant: [`OpenAiApiPath::MODELS`](crate::paths::OpenAiApiPath::MODELS).
    async fn list_models(&self) -> Result<ListModelsResponse, Self::Error>;

    /// Retrieves a model instance, providing basic information about the model such as the owner and
    /// permissioning.
    ///
    /// REST: `GET /models/{model}`.
    /// Path constant: [`OpenAiApiPath::MODELS_BY_MODEL`](crate::paths::OpenAiApiPath::MODELS_BY_MODEL).
    async fn retrieve_model(&self, model: String) -> Result<Model, Self::Error>;

    /// Delete a fine-tuned model. You must have the Owner role in your organization to delete a model.
    ///
    /// REST: `DELETE /models/{model}`.
    /// Path constant: [`OpenAiApiPath::MODELS_BY_MODEL`](crate::paths::OpenAiApiPath::MODELS_BY_MODEL).
    async fn delete_model(&self, model: String) -> Result<DeleteModelResponse, Self::Error>;
}
