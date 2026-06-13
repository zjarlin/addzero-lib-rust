//! 二维码识别错误类型。

use std::path::PathBuf;

/// 二维码识别 API 使用的 Result。
pub type QrCodeRecognitionResult<T> = Result<T, QrCodeRecognitionError>;

/// 二维码识别过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum QrCodeRecognitionError {
    /// 文件系统操作失败。
    #[error("filesystem error at `{path}`: {source}")]
    Io {
        /// 失败操作涉及的路径。
        path: PathBuf,
        /// 原始 IO 错误。
        #[source]
        source: std::io::Error,
    },

    /// 图片解码或编码失败。
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
}

impl QrCodeRecognitionError {
    /// 为 IO 错误补充路径上下文。
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
