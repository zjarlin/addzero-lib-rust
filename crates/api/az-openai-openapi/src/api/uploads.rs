// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Uploads REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    AddUploadPartRequest,
    CompleteUploadRequest,
    CreateUploadRequest,
    Upload,
    UploadPart,
};

/// Uploads REST endpoints.
#[async_trait]
pub trait OpenAiUploadsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Creates an intermediate [Upload](/docs/api-reference/uploads/object) object that you can add
    /// [Parts](/docs/api-reference/uploads/part-object) to. Currently, an Upload can accept at most 8 GB in
    /// total and expires after an hour after you create it. Once you complete the Upload, we will create a
    /// [File](/docs/api-reference/files/object) object that contains all the parts you uploaded. This File
    /// is usable in the rest of our platform as a regular File object. For certain `purpose` values, the
    /// correct `mime_type` must be specified. Please refer to documentation for the [supported MIME types
    /// for your use case](/docs/assistants/tools/file-search#supported-files). For guidance on the proper
    /// filename extensions for each purpose, please follow the documentation on [creating a
    /// File](/docs/api-reference/files/create). Returns the Upload object with status `pending`.
    ///
    /// REST: `POST /uploads`.
    /// Path constant: [`OpenAiApiPath::UPLOADS`](crate::paths::OpenAiApiPath::UPLOADS).
    async fn create_upload(&self, body: CreateUploadRequest) -> Result<Upload, Self::Error>;

    /// Cancels the Upload. No Parts may be added after an Upload is cancelled. Returns the Upload object
    /// with status `cancelled`.
    ///
    /// REST: `POST /uploads/{upload_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_CANCEL).
    async fn cancel_upload(&self, upload_id: String) -> Result<Upload, Self::Error>;

    /// Completes the [Upload](/docs/api-reference/uploads/object). Within the returned Upload object, there
    /// is a nested [File](/docs/api-reference/files/object) object that is ready to use in the rest of the
    /// platform. You can specify the order of the Parts by passing in an ordered list of the Part IDs. The
    /// number of bytes uploaded upon completion must match the number of bytes initially specified when
    /// creating the Upload object. No Parts may be added after an Upload is completed. Returns the Upload
    /// object with status `completed`, including an additional `file` property containing the created
    /// usable File object.
    ///
    /// REST: `POST /uploads/{upload_id}/complete`.
    /// Path constant: [`OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_COMPLETE`](crate::paths::OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_COMPLETE).
    async fn complete_upload(
        &self,
        upload_id: String,
        body: CompleteUploadRequest,
    ) -> Result<Upload, Self::Error>;

    /// Adds a [Part](/docs/api-reference/uploads/part-object) to an [Upload](/docs/api-
    /// reference/uploads/object) object. A Part represents a chunk of bytes from the file you are trying to
    /// upload. Each Part can be at most 64 MB, and you can add Parts until you hit the Upload maximum of 8
    /// GB. It is possible to add multiple Parts in parallel. You can decide the intended order of the Parts
    /// when you [complete the Upload](/docs/api-reference/uploads/complete).
    ///
    /// REST: `POST /uploads/{upload_id}/parts`.
    /// Path constant: [`OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_PARTS`](crate::paths::OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_PARTS).
    async fn add_upload_part(
        &self,
        upload_id: String,
        body: AddUploadPartRequest,
    ) -> Result<UploadPart, Self::Error>;
}
