//! 多算法 pipeline 错误类型。

use std::path::PathBuf;

/// 多算法 pipeline API 使用的 Result。
pub type AlgorithmPipelineResult<T> = Result<T, AlgorithmPipelineError>;

/// 多算法 pipeline 运行过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum AlgorithmPipelineError {
    /// 文件系统操作失败。
    #[error("filesystem error at `{path}`: {source}")]
    Io {
        /// 失败操作涉及的路径。
        path: PathBuf,
        /// 原始 IO 错误。
        #[source]
        source: std::io::Error,
    },

    /// 人脸检测算法失败。
    #[error("face detection failed: {0}")]
    FaceDetection(#[from] az_face_detection::error::FaceDetectionError),

    /// ONNX 类算法失败。
    #[error("ONNX image algorithm failed: {0}")]
    OnnxImage(#[from] az_algorithm_onnx::error::OnnxImageError),

    /// 人员检测算法失败。
    #[error("person detection failed: {0}")]
    PersonDetection(#[from] az_person_detection::error::PersonDetectionError),

    /// 火焰检测算法失败。
    #[error("flame detection failed: {0}")]
    FlameDetection(#[from] az_flame_detection::error::FlameDetectionError),

    /// 车辆检测算法失败。
    #[error("vehicle detection failed: {0}")]
    VehicleDetection(#[from] az_vehicle_detection::error::VehicleDetectionError),

    /// 二维码识别算法失败。
    #[error("QR code recognition failed: {source}")]
    QrCodeRecognition {
        /// 原始二维码识别错误。
        #[source]
        source: anyhow::Error,
    },

    /// 汇总 JSON 序列化失败。
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AlgorithmPipelineError {
    /// 为 IO 错误补充路径上下文。
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// 为二维码识别错误补充 pipeline 上下文。
    #[must_use]
    pub fn qr_code_recognition(source: anyhow::Error) -> Self {
        Self::QrCodeRecognition { source }
    }
}
