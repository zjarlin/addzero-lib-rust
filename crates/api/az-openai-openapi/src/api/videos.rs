//! Videos REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Videos REST endpoints.
#[async_trait]
pub trait OpenAiVideosApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// List recently generated videos for the current project.
    ///
    /// REST: `GET /videos`.
    /// Path constant: [`OpenAiApiPath::VIDEOS`](crate::paths::OpenAiApiPath::VIDEOS).
    async fn list_videos(
        &self,
        limit: Option<i64>,
        order: Option<String>,
        after: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Create a new video generation job from a prompt and optional reference assets.
    ///
    /// REST: `POST /videos`.
    /// Path constant: [`OpenAiApiPath::VIDEOS`](crate::paths::OpenAiApiPath::VIDEOS).
    async fn create_video(
        &self,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Create a character from an uploaded video.
    ///
    /// REST: `POST /videos/characters`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_CHARACTERS`](crate::paths::OpenAiApiPath::VIDEOS_BY_CHARACTERS).
    async fn create_video_character(
        &self,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Fetch a character.
    ///
    /// REST: `GET /videos/characters/{character_id}`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_CHARACTERS_BY_CHARACTER_ID`](crate::paths::OpenAiApiPath::VIDEOS_BY_CHARACTERS_BY_CHARACTER_ID).
    async fn get_video_character(
        &self,
        character_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Create a new video generation job by editing a source video or existing generated video.
    ///
    /// REST: `POST /videos/edits`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_EDITS`](crate::paths::OpenAiApiPath::VIDEOS_BY_EDITS).
    async fn create_video_edit(
        &self,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Create an extension of a completed video.
    ///
    /// REST: `POST /videos/extensions`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_EXTENSIONS`](crate::paths::OpenAiApiPath::VIDEOS_BY_EXTENSIONS).
    async fn create_video_extend(
        &self,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Permanently delete a completed or failed video and its stored assets.
    ///
    /// REST: `DELETE /videos/{video_id}`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_VIDEO_ID`](crate::paths::OpenAiApiPath::VIDEOS_BY_VIDEO_ID).
    async fn delete_video(&self, video_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Fetch the latest metadata for a generated video.
    ///
    /// REST: `GET /videos/{video_id}`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_VIDEO_ID`](crate::paths::OpenAiApiPath::VIDEOS_BY_VIDEO_ID).
    async fn get_video(&self, video_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Download the generated video bytes or a derived preview asset. Streams the rendered video content for the specified video job.
    ///
    /// REST: `GET /videos/{video_id}/content`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_VIDEO_ID_BY_CONTENT`](crate::paths::OpenAiApiPath::VIDEOS_BY_VIDEO_ID_BY_CONTENT).
    async fn retrieve_video_content(
        &self,
        video_id: String,
        variant: Option<String>,
    ) -> Result<OpenAiBinaryBody, Self::Error>;

    /// Create a remix of a completed video using a refreshed prompt.
    ///
    /// REST: `POST /videos/{video_id}/remix`.
    /// Path constant: [`OpenAiApiPath::VIDEOS_BY_VIDEO_ID_BY_REMIX`](crate::paths::OpenAiApiPath::VIDEOS_BY_VIDEO_ID_BY_REMIX).
    async fn create_video_remix(
        &self,
        video_id: String,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
