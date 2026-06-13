//! 共享 ONNX 图片推理错误类型。

use std::path::PathBuf;

/// 共享 ONNX 推理 API 使用的 Result。
pub type OnnxImageResult<T> = Result<T, OnnxImageError>;

/// 本地 ONNX 图片推理过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum OnnxImageError {
    /// 模型文件不存在。
    #[error("model file `{path}` is missing")]
    ModelFileMissing {
        /// 预期模型文件路径。
        path: PathBuf,
    },

    /// 模型没有声明可执行输入形状。
    #[error("model `{model_code}` does not declare a runnable image tensor shape")]
    MissingRunnableShape {
        /// 稳定模型 code。
        model_code: &'static str,
    },

    /// 请求操作使用的张量形状无效。
    #[error("invalid tensor shape for `{model_code}`: {reason}")]
    InvalidTensorShape {
        /// 稳定模型 code。
        model_code: &'static str,
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

    /// 当前 helper 无法摘要该 ONNX 输出类型。
    #[error("unsupported ONNX output tensor type `{tensor_type}` from output `{output_name}`")]
    UnsupportedOnnxOutput {
        /// ONNX 图报告的输出名称。
        output_name: String,
        /// 张量元素类型。
        tensor_type: String,
    },

    /// JSON 序列化失败。
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl OnnxImageError {
    /// 为 IO 错误补充路径上下文。
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
