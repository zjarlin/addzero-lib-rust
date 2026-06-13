//! Files REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Files REST endpoints.
#[async_trait]
pub trait OpenAiFilesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Returns a list of files.
    ///
    /// REST: `GET /files`.
    /// Path constant: [`OpenAiApiPath::FILES`](crate::paths::OpenAiApiPath::FILES).
    async fn list_files(
        &self,
        purpose: Option<String>,
        limit: Option<i64>,
        order: Option<String>,
        after: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Upload a file that can be used across various endpoints. Individual files can be up to 512 MB, and each project can store up to 2.5 TB of files in total. There is no organization-wide storage limit. Uploads to this endpoint are rate-limited to 1,000 requests per minute per authenticated user. - The Assistants API supports files up to 2 million tokens and of specific file types. See the [Assistants Tools guide](/docs/assistants/tools) for details. - The Fine-tuning API only supports `.jsonl` files. The input also has certain required formats for fine-tuning [chat](/docs/api-reference/fine-tuning/chat-input) or [completions](/docs/api-reference/fine-tuning/completions-input) models. - The Batch API only supports `.jsonl` files up to 200 MB in size. The input also has a specific required [format](/docs/api-reference/batch/request-input). - For Retrieval or `file_search` ingestion, upload files here first. If you need to attach multiple uploaded files to the same vector store, use [`/vector_stores/{vector_store_id}/file_batches`](/docs/api-reference/vector-stores-file-batches/createBatch) instead of attaching them one by one. Vector store attachment has separate limits from file upload, including 2,000 attached files per minute per organization. Please [contact us](https://help.openai.com/) if you need to increase these storage limits.
    ///
    /// REST: `POST /files`.
    /// Path constant: [`OpenAiApiPath::FILES`](crate::paths::OpenAiApiPath::FILES).
    async fn create_file(&self, body: OpenAiRequestBody)
    -> Result<OpenAiResponseBody, Self::Error>;

    /// Delete a file and remove it from all vector stores.
    ///
    /// REST: `DELETE /files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::FILES_BY_FILE_ID).
    async fn delete_file(&self, file_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Returns information about a specific file.
    ///
    /// REST: `GET /files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::FILES_BY_FILE_ID).
    async fn retrieve_file(&self, file_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Returns the contents of the specified file.
    ///
    /// REST: `GET /files/{file_id}/content`.
    /// Path constant: [`OpenAiApiPath::FILES_BY_FILE_ID_BY_CONTENT`](crate::paths::OpenAiApiPath::FILES_BY_FILE_ID_BY_CONTENT).
    async fn download_file(&self, file_id: String) -> Result<OpenAiBinaryBody, Self::Error>;
}
