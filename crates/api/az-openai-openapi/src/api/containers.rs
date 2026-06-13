// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Containers REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    ContainerFileListResource,
    ContainerFileResource,
    ContainerListResource,
    ContainerResource,
    CreateContainerBody,
    CreateContainerFileBody,
};

/// Containers REST endpoints.
#[async_trait]
pub trait OpenAiContainersApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List Containers
    ///
    /// REST: `GET /containers`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS`](crate::paths::OpenAiApiPath::CONTAINERS).
    async fn list_containers(
        &self,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        name: Option<String>,
    ) -> Result<ContainerListResource, Self::Error>;

    /// Create Container
    ///
    /// REST: `POST /containers`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS`](crate::paths::OpenAiApiPath::CONTAINERS).
    async fn create_container(
        &self,
        body: Option<CreateContainerBody>,
    ) -> Result<ContainerResource, Self::Error>;

    /// Retrieve Container
    ///
    /// REST: `GET /containers/{container_id}`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID).
    async fn retrieve_container(
        &self,
        container_id: String,
    ) -> Result<ContainerResource, Self::Error>;

    /// Delete Container
    ///
    /// REST: `DELETE /containers/{container_id}`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID).
    async fn delete_container(&self, container_id: String) -> Result<(), Self::Error>;

    /// List Container files
    ///
    /// REST: `GET /containers/{container_id}/files`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES).
    async fn list_container_files(
        &self,
        container_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
    ) -> Result<ContainerFileListResource, Self::Error>;

    /// Create a Container File You can send either a multipart/form-data request with the raw file content,
    /// or a JSON request with a file ID.
    ///
    /// REST: `POST /containers/{container_id}/files`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES).
    async fn create_container_file(
        &self,
        container_id: String,
        body: CreateContainerFileBody,
    ) -> Result<ContainerFileResource, Self::Error>;

    /// Retrieve Container File
    ///
    /// REST: `GET /containers/{container_id}/files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID).
    async fn retrieve_container_file(
        &self,
        container_id: String,
        file_id: String,
    ) -> Result<ContainerFileResource, Self::Error>;

    /// Delete Container File
    ///
    /// REST: `DELETE /containers/{container_id}/files/{file_id}`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID).
    async fn delete_container_file(
        &self,
        container_id: String,
        file_id: String,
    ) -> Result<(), Self::Error>;

    /// Retrieve Container File Content
    ///
    /// REST: `GET /containers/{container_id}/files/{file_id}/content`.
    /// Path constant: [`OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID_BY_CONTENT`](crate::paths::OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID_BY_CONTENT).
    async fn retrieve_container_file_content(
        &self,
        container_id: String,
        file_id: String,
    ) -> Result<(), Self::Error>;
}
