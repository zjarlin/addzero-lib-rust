//! Embeddings REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Embeddings REST endpoints.
#[async_trait]
pub trait OpenAiEmbeddingsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Creates an embedding vector representing the input text.
    ///
    /// REST: `POST /embeddings`.
    /// Path constant: [`OpenAiApiPath::EMBEDDINGS`](crate::paths::OpenAiApiPath::EMBEDDINGS).
    async fn create_embedding(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
