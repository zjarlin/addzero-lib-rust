//! 人脸检测公开模型。

use std::path::PathBuf;

/// SCRFD 模型稳定 code。
pub const MODEL_CODE: &str = "face_detection_scrfd_500m";

/// 默认模型文件名。
pub const DEFAULT_MODEL_FILE_NAME: &str = "face_detection_scrfd_500m.onnx";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/face_detection";

/// 默认模型输入宽度。
pub const MODEL_INPUT_WIDTH: u32 = 640;

/// 默认模型输入高度。
pub const MODEL_INPUT_HEIGHT: u32 = 640;

/// SCRFD ONNX 输入形状。
pub const MODEL_INPUT_SHAPE: &[usize] = &[1, 3, 640, 640];

/// 人脸检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct FaceDetectionOptions {
    /// SCRFD ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// 置信度阈值。
    pub score_threshold: f32,
    /// NMS 阈值。
    pub nms_threshold: f32,
}

/// 单个人脸检测框。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FaceDetectionBox {
    /// 左上角 x 坐标，单位是原图像素。
    pub x_min: f32,
    /// 左上角 y 坐标，单位是原图像素。
    pub y_min: f32,
    /// 右下角 x 坐标，单位是原图像素。
    pub x_max: f32,
    /// 右下角 y 坐标，单位是原图像素。
    pub y_max: f32,
    /// 人脸置信度。
    pub confidence: f32,
}

/// 单个 ONNX 输出摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FaceDetectionOutputSummary {
    /// 输出张量名称。
    pub name: String,
    /// 输出张量元素类型。
    pub tensor_type: String,
    /// 运行时输出形状。
    pub shape: Vec<i64>,
    /// 张量标量元素数量。
    pub element_count: usize,
    /// 前几个 f32 样本值。
    pub sample_f32: Vec<f32>,
}

/// 人脸检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FaceDetectionOutputFiles {
    /// 原始输入图副本。
    pub source_input: PathBuf,
    /// 模型实际看到的 resize 输入预览图。
    pub model_input_preview: PathBuf,
    /// ONNX 原始输出摘要 JSON。
    pub raw_outputs_json: PathBuf,
    /// SCRFD 后处理得到的人脸框 JSON。
    pub detected_faces_json: PathBuf,
    /// 画出人脸框的输出图。
    pub detected_faces_image: PathBuf,
}

/// 人脸检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FaceDetectionRun {
    /// 输入图片路径。二进制或 base64 输入会先写入输出目录，再记录该路径。
    pub input_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 检测到的人脸框。
    pub faces: Vec<FaceDetectionBox>,
    /// 输出文件路径。
    pub files: FaceDetectionOutputFiles,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<FaceDetectionOutputSummary>,
}
