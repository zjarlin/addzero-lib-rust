//! Images REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Images REST endpoints.
#[async_trait]
pub trait OpenAiImagesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Creates an edited or extended image given one or more source images and a prompt. This endpoint supports GPT Image models (`gpt-image-1.5`, `gpt-image-1`, `gpt-image-1-mini`, and `chatgpt-image-latest`) and `dall-e-2`.
    ///
    /// REST: `POST /images/edits`.
    /// Path constant: [`OpenAiApiPath::IMAGES_BY_EDITS`](crate::paths::OpenAiApiPath::IMAGES_BY_EDITS).
    async fn create_image_edit(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates an image given a prompt. [Learn more](/docs/guides/images).
    ///
    /// REST: `POST /images/generations`.
    /// Path constant: [`OpenAiApiPath::IMAGES_BY_GENERATIONS`](crate::paths::OpenAiApiPath::IMAGES_BY_GENERATIONS).
    async fn create_image(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates a variation of a given image. This endpoint only supports `dall-e-2`.
    ///
    /// REST: `POST /images/variations`.
    /// Path constant: [`OpenAiApiPath::IMAGES_BY_VARIATIONS`](crate::paths::OpenAiApiPath::IMAGES_BY_VARIATIONS).
    async fn create_image_variation(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
