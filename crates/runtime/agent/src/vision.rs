use serde_json::{Value, json};

/// 视觉模型输入的图片细节级别。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionDetail {
    /// 由 provider 自行选择。
    Auto,
    /// 较低 token 或成本的细节级别。
    Low,
    /// 较高保真度的细节级别。
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

/// Responses 和兼容 chat 的视觉 API 可接受的图片引用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionInput {
    /// URL、data URL 或 file ID，取决于 provider 支持范围。
    pub source: VisionSource,
    /// 请求的细节级别。
    pub detail: VisionDetail,
}

impl VisionInput {
    /// 根据 URL 或 data URL 创建图片输入。
    pub fn image_url(url: impl Into<String>) -> Self {
        Self {
            source: VisionSource::ImageUrl(url.into()),
            detail: VisionDetail::Auto,
        }
    }

    /// 根据已上传文件 ID 创建图片输入。
    pub fn file_id(file_id: impl Into<String>) -> Self {
        Self {
            source: VisionSource::FileId(file_id.into()),
            detail: VisionDetail::Auto,
        }
    }

    /// 覆盖图片细节级别。
    pub fn with_detail(mut self, detail: VisionDetail) -> Self {
        self.detail = detail;
        self
    }

    /// 转换为 Responses 的 `input_image` 内容项。
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

    /// 转换为 Chat Completions 的 `image_url` 内容项。
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

/// 视觉输入的底层来源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionSource {
    /// 完整 URL 或 base64 data URL。
    ImageUrl(String),
    /// 已上传到 provider 的 file ID。
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
