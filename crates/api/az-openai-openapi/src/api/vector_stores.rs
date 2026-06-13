// Generated from OpenAPI spec. Do not edit by hand.
//! VectorStores REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateVectorStoreFileBatchRequest,
    CreateVectorStoreFileRequest,
    CreateVectorStoreRequest,
    DeleteVectorStoreFileResponse,
    DeleteVectorStoreResponse,
    ListVectorStoreFilesResponse,
    ListVectorStoresResponse,
    UpdateVectorStoreFileAttributesRequest,
    UpdateVectorStoreRequest,
    VectorStoreFileBatchObject,
    VectorStoreFileContentResponse,
    VectorStoreFileObject,
    VectorStoreObject,
    VectorStoreSearchRequest,
    VectorStoreSearchResultsPage,
};

/// VectorStores REST endpoints.
#[async_trait]
pub trait OpenAiVectorStoresApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns a list of vector stores.
    ///
    /// REST: `GET /vector_stores`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES`](crate::paths::OpenAiApiPath::VECTOR_STORES).
    async fn list_vector_stores(
        &self,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListVectorStoresResponse, Self::Error>;

    /// Create a vector store.
    ///
    /// REST: `POST /vector_stores`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES`](crate::paths::OpenAiApiPath::VECTOR_STORES).
    async fn create_vector_store(
        &self,
        body: CreateVectorStoreRequest,
    ) -> Result<VectorStoreObject, Self::Error>;

    /// Retrieves a vector store.
    ///
    /// REST: `GET /vector_stores/{vector_store_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID).
    async fn get_vector_store(
        &self,
        vector_store_id: String,
    ) -> Result<VectorStoreObject, Self::Error>;

    /// Modifies a vector store.
    ///
    /// REST: `POST /vector_stores/{vector_store_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID).
    async fn modify_vector_store(
        &self,
        vector_store_id: String,
        body: UpdateVectorStoreRequest,
    ) -> Result<VectorStoreObject, Self::Error>;

    /// Delete a vector store.
    ///
    /// REST: `DELETE /vector_stores/{vector_store_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID).
    async fn delete_vector_store(
        &self,
        vector_store_id: String,
    ) -> Result<DeleteVectorStoreResponse, Self::Error>;

    /// Create a vector store file batch.
    ///
    /// REST: `POST /vector_stores/{vector_store_id}/file_batches`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES).
    async fn create_vector_store_file_batch(
        &self,
        vector_store_id: String,
        body: CreateVectorStoreFileBatchRequest,
    ) -> Result<VectorStoreFileBatchObject, Self::Error>;

    /// Retrieves a vector store file batch.
    ///
    /// REST: `GET /vector_stores/{vector_store_id}/file_batches/{batch_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID).
    async fn get_vector_store_file_batch(
        &self,
        vector_store_id: String,
        batch_id: String,
    ) -> Result<VectorStoreFileBatchObject, Self::Error>;

    /// Cancel a vector store file batch. This attempts to cancel the processing of files in this batch as
    /// soon as possible.
    ///
    /// REST: `POST /vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID_BY_CANCEL).
    async fn cancel_vector_store_file_batch(
        &self,
        vector_store_id: String,
        batch_id: String,
    ) -> Result<VectorStoreFileBatchObject, Self::Error>;

    /// Returns a list of vector store files in a batch.
    ///
    /// REST: `GET /vector_stores/{vector_store_id}/file_batches/{batch_id}/files`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID_BY_FILES`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID_BY_FILES).
    async fn list_files_in_vector_store_batch(
        &self,
        vector_store_id: String,
        batch_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
        filter: Option<String>,
    ) -> Result<ListVectorStoreFilesResponse, Self::Error>;

    /// Returns a list of vector store files.
    ///
    /// REST: `GET /vector_stores/{vector_store_id}/files`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES).
    async fn list_vector_store_files(
        &self,
        vector_store_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
        filter: Option<String>,
    ) -> Result<ListVectorStoreFilesResponse, Self::Error>;

    /// Create a vector store file by attaching a [File](/docs/api-reference/files) to a [vector
    /// store](/docs/api-reference/vector-stores/object).
    ///
    /// REST: `POST /vector_stores/{vector_store_id}/files`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES).
    async fn create_vector_store_file(
        &self,
        vector_store_id: String,
        body: CreateVectorStoreFileRequest,
    ) -> Result<VectorStoreFileObject, Self::Error>;

    /// Retrieves a vector store file.
    ///
    /// REST: `GET /vector_stores/{vector_store_id}/files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID).
    async fn get_vector_store_file(
        &self,
        vector_store_id: String,
        file_id: String,
    ) -> Result<VectorStoreFileObject, Self::Error>;

    /// Update attributes on a vector store file.
    ///
    /// REST: `POST /vector_stores/{vector_store_id}/files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID).
    async fn update_vector_store_file_attributes(
        &self,
        vector_store_id: String,
        file_id: String,
        body: UpdateVectorStoreFileAttributesRequest,
    ) -> Result<VectorStoreFileObject, Self::Error>;

    /// Delete a vector store file. This will remove the file from the vector store but the file itself will
    /// not be deleted. To delete the file, use the [delete file](/docs/api-reference/files/delete)
    /// endpoint.
    ///
    /// REST: `DELETE /vector_stores/{vector_store_id}/files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID).
    async fn delete_vector_store_file(
        &self,
        vector_store_id: String,
        file_id: String,
    ) -> Result<DeleteVectorStoreFileResponse, Self::Error>;

    /// Retrieve the parsed contents of a vector store file.
    ///
    /// REST: `GET /vector_stores/{vector_store_id}/files/{file_id}/content`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID_BY_CONTENT`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID_BY_CONTENT).
    async fn retrieve_vector_store_file_content(
        &self,
        vector_store_id: String,
        file_id: String,
    ) -> Result<VectorStoreFileContentResponse, Self::Error>;

    /// Search a vector store for relevant chunks based on a query and file attributes filter.
    ///
    /// REST: `POST /vector_stores/{vector_store_id}/search`.
    /// Path constant: [`OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_SEARCH`](crate::paths::OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_SEARCH).
    async fn search_vector_store(
        &self,
        vector_store_id: String,
        body: VectorStoreSearchRequest,
    ) -> Result<VectorStoreSearchResultsPage, Self::Error>;
}
