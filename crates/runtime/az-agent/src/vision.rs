use serde_json::{Value, json};

/// Image detail level for vision-capable model inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionDetail {
    /// Let the provider choose.
    Auto,
    /// Lower token/cost detail.
    Low,
    /// Higher fidelity detail.
    High,
}

impl VisionDetail {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

/// Image reference accepted by Responses and chat-compatible vision APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionInput {
    /// URL, data URL, or file ID depending on provider support.
    pub source: VisionSource,
    /// Requested detail level.
    pub detail: VisionDetail,
}

impl VisionInput {
    /// Creates an image input from a URL or data URL.
    pub fn image_url(url: impl Into<String>) -> Self {
        Self {
            source: VisionSource::ImageUrl(url.into()),
            detail: VisionDetail::Auto,
        }
    }

    /// Creates an image input from an uploaded file ID.
    pub fn file_id(file_id: impl Into<String>) -> Self {
        Self {
            source: VisionSource::FileId(file_id.into()),
            detail: VisionDetail::Auto,
        }
    }

    /// Overrides image detail.
    pub fn with_detail(mut self, detail: VisionDetail) -> Self {
        self.detail = detail;
        self
    }

    /// Converts to a Responses `input_image` content item.
    pub fn to_responses_content(&self) -> Value {
        match &self.source {
            VisionSource::ImageUrl(url) => json!({
                "type": "input_image",
                "image_url": url,
                "detail": self.detail.as_str(),
            }),
            VisionSource::FileId(file_id) => json!({
                "type": "input_image",
                "file_id": file_id,
                "detail": self.detail.as_str(),
            }),
        }
    }

    /// Converts to a Chat Completions `image_url` content item.
    pub fn to_chat_content(&self) -> Value {
        match &self.source {
            VisionSource::ImageUrl(url) => json!({
                "type": "image_url",
                "image_url": {
                    "url": url,
                    "detail": self.detail.as_str(),
                }
            }),
            VisionSource::FileId(file_id) => json!({
                "type": "image_url",
                "image_url": {
                    "url": file_id,
                    "detail": self.detail.as_str(),
                }
            }),
        }
    }
}

/// Backing source for a vision input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionSource {
    /// Fully qualified URL or base64 data URL.
    ImageUrl(String),
    /// Uploaded provider file ID.
    FileId(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_responses_input_image_content() {
        let input =
            VisionInput::image_url("https://example.com/a.png").with_detail(VisionDetail::Low);

        assert_eq!(
            input.to_responses_content(),
            json!({
                "type": "input_image",
                "image_url": "https://example.com/a.png",
                "detail": "low"
            })
        );
    }

    #[test]
    fn builds_chat_image_url_content() {
        let input =
            VisionInput::image_url("data:image/png;base64,abc").with_detail(VisionDetail::High);

        assert_eq!(
            input.to_chat_content(),
            json!({
                "type": "image_url",
                "image_url": {
                    "url": "data:image/png;base64,abc",
                    "detail": "high"
                }
            })
        );
    }
}
