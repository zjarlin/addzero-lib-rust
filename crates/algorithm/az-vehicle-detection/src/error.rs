//! 车辆检测错误类型。

use std::path::PathBuf;

/// 车辆检测 API 统一使用的 Result。
pub type VehicleDetectionResult<T> = Result<T, VehicleDetectionError>;

/// 车辆检测执行过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum VehicleDetectionError {
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

    /// JSON 序列化失败。
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// 输入张量或输出张量形状不符合 COCO SSD 预期。
    #[error("invalid tensor shape for `{model_code}`: {reason}")]
    InvalidTensorShape {
        /// 稳定模型 code。
        model_code: &'static str,
        /// 可读校验失败原因。
        reason: String,
    },

    /// 模型文件不存在。
    #[error("model file `{path}` is missing")]
    ModelFileMissing {
        /// 预期模型文件路径。
        path: PathBuf,
    },

    /// ONNX 输出缺少后处理需要的张量。
    #[error("missing ONNX output `{output_name}`")]
    MissingOnnxOutput {
        /// 缺失的输出张量名称。
        output_name: String,
    },

    /// 当前实现无法处理该 ONNX 输出类型。
    #[error("unsupported ONNX output tensor type `{tensor_type}` from output `{output_name}`")]
    UnsupportedOnnxOutput {
        /// 输出张量名称。
        output_name: String,
        /// 输出张量元素类型。
        tensor_type: String,
    },
}

impl VehicleDetectionError {
    /// 为 IO 错误补充路径上下文。
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
