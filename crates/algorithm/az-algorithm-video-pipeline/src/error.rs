//! 视频算法 pipeline 错误类型。

use std::path::PathBuf;

/// 视频算法 pipeline API 使用的 Result。
pub type AlgorithmVideoPipelineResult<T> = Result<T, AlgorithmVideoPipelineError>;

/// 视频算法 pipeline 运行过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum AlgorithmVideoPipelineError {
    /// 配置或输入帧不合法。
    #[error("invalid video pipeline input: {reason}")]
    InvalidInput {
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

    /// 汇总 JSON 序列化失败。
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AlgorithmVideoPipelineError {
    /// 为配置或输入帧错误补充可读原因。
    #[must_use]
    pub fn invalid_input(reason: impl Into<String>) -> Self {
        Self::InvalidInput {
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
