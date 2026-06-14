//! 多算法 pipeline 公开模型。

use std::path::PathBuf;

/// 可由图片 pipeline 编排的算法。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ImageAlgorithmKind {
    /// 人脸检测。
    FaceDetection,
    /// 人脸识别。
    FaceRecognition,
    /// 人员检测。
    PersonDetection,
    /// OCR 文字识别。
    OcrTextRecognition,
    /// 火焰检测。
    FlameDetection,
    /// 安全帽检测。
    SafetyHelmetDetection,
    /// 车辆检测。
    VehicleDetection,
    /// 二维码识别。
    QrCodeRecognition,
}

impl ImageAlgorithmKind {
    /// 返回稳定 snake_case code。
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::FaceDetection => "face_detection",
            Self::FaceRecognition => "face_recognition",
            Self::PersonDetection => "person_detection",
            Self::OcrTextRecognition => "ocr_text_recognition",
            Self::FlameDetection => "flame_detection",
            Self::SafetyHelmetDetection => "safety_helmet_detection",
            Self::VehicleDetection => "vehicle_detection",
            Self::QrCodeRecognition => "qr_code_recognition",
        }
    }
}

/// 图片 pipeline 执行配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagePipelineOptions {
    /// 本次启用的算法列表。
    pub algorithms: Vec<ImageAlgorithmKind>,
    /// 本次任务输出根目录。
    pub output_dir: PathBuf,
}

/// 单个算法执行后的汇总信息。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ImageAlgorithmRunSummary {
    /// 算法种类。
    pub algorithm: ImageAlgorithmKind,
    /// 算法稳定 code。
    pub code: String,
    /// 该算法子输出目录。
    pub output_dir: PathBuf,
    /// 该算法写出的文件路径。
    pub files: Vec<PathBuf>,
}

/// 多算法 pipeline 执行结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ImagePipelineRun {
    /// 输入图片绝对路径。
    pub input_path: PathBuf,
    /// 本次任务输出根目录。
    pub output_dir: PathBuf,
    /// 总汇总 JSON 文件。
    pub summary_file: PathBuf,
    /// 每个算法的汇总。
    pub algorithm_runs: Vec<ImageAlgorithmRunSummary>,
}
