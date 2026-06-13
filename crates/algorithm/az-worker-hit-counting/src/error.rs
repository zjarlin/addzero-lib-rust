//! 工人敲击计数错误类型。

use std::path::PathBuf;

/// 工人敲击计数 API 使用的 Result。
pub type WorkerHitCountingResult<T> = Result<T, WorkerHitCountingError>;

/// 工人敲击计数可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum WorkerHitCountingError {
    /// 视觉动作观测或敲击计数配置无效。
    #[error("invalid visual action input: {reason}")]
    InvalidVisualActionInput {
        /// 可读校验失败原因。
        reason: String,
    },

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

    /// ONNX Runtime 返回错误。
    #[error("ONNX Runtime error: {0}")]
    OnnxRuntime(#[from] ort::Error),

    /// 输入张量或输出张量形状不符合姿态模型预期。
    #[error("invalid pose tensor shape: {reason}")]
    InvalidPoseTensorShape {
        /// 可读校验失败原因。
        reason: String,
    },
}

impl WorkerHitCountingError {
    /// 为视觉动作输入或配置错误补充可读原因。
    #[must_use]
    pub fn invalid_visual_action_input(reason: impl Into<String>) -> Self {
        Self::InvalidVisualActionInput {
            reason: reason.into(),
        }
    }

    /// 为 IO 错误补充路径上下文。
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
