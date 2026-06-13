//! Batch REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Batch REST endpoints.
#[async_trait]
pub trait OpenAiBatchApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// List your organization's batches.
    ///
    /// REST: `GET /batches`.
    /// Path constant: [`OpenAiApiPath::BATCHES`](crate::paths::OpenAiApiPath::BATCHES).
    async fn list_batches(
        &self,
        after: Option<String>,
        limit: Option<i64>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates and executes a batch from an uploaded file of requests
    ///
    /// REST: `POST /batches`.
    /// Path constant: [`OpenAiApiPath::BATCHES`](crate::paths::OpenAiApiPath::BATCHES).
    async fn create_batch(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Retrieves a batch.
    ///
    /// REST: `GET /batches/{batch_id}`.
    /// Path constant: [`OpenAiApiPath::BATCHES_BY_BATCH_ID`](crate::paths::OpenAiApiPath::BATCHES_BY_BATCH_ID).
    async fn retrieve_batch(&self, batch_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Cancels an in-progress batch. The batch will be in status `cancelling` for up to 10 minutes, before changing to `cancelled`, where it will have partial results (if any) available in the output file.
    ///
    /// REST: `POST /batches/{batch_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::BATCHES_BY_BATCH_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::BATCHES_BY_BATCH_ID_BY_CANCEL).
    async fn cancel_batch(&self, batch_id: String) -> Result<OpenAiResponseBody, Self::Error>;
}
