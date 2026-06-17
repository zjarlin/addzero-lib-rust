//! 火焰与烟雾检测模型规格。

use std::path::PathBuf;

use az_onnx::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputKind, TensorElementKind, TensorInputSpec,
};

/// 火焰检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "flame_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/flame_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/flame_detection/models";

/// 默认检测置信度阈值。
pub const DEFAULT_SCORE_THRESHOLD: f32 = 0.25;

/// 默认 YOLO NMS 阈值。
pub const DEFAULT_NMS_THRESHOLD: f32 = 0.45;

const FIRE_SMOKE_YOLO_INPUT: &[usize] = &[1, 3, 320, 320];

/// CCTV fire/smoke YOLOv8n 检测模型。
pub const FLAME_DETECTION_FIRE_SMOKE_YOLOV8N: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "flame_detection_fire_smoke_yolov8n",
    label: "YOLOv8n fire/smoke detection",
    source_repo: "fiacecson20/cctv-ai-fire-smoke",
    source_file: "best.onnx",
    local_file: "fire_smoke_yolov8n.onnx",
    license: "mit weights; base model is Ultralytics YOLOv8",
    revision: "343990e42d99a5d27e9f35fc7c80880dc5f43f45",
    input: TensorInputSpec {
        shape: FIRE_SMOKE_YOLO_INPUT,
        element: TensorElementKind::Float32,
    },
    output_kind: OnnxImageOutputKind::RawTensor,
    notes: "YOLOv8 object detector for fire and smoke. Model card notes MIT weights; Ultralytics YOLOv8 base model may require AGPL compliance for hosted service use.",
};

/// 火焰检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct FlameDetectionOptions {
    /// Fire/smoke YOLOv8n ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// 检测置信度阈值。
    pub score_threshold: f32,
    /// NMS 阈值。
    pub nms_threshold: f32,
}

/// 当前 fire/smoke 模型支持的检测类别。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlameDetectionClass {
    /// 明火。
    Fire,
    /// 烟雾。
    Smoke,
}

impl FlameDetectionClass {
    /// 返回 YOLO 类别索引。
    #[must_use]
    pub const fn class_index(self) -> usize {
        match self {
            Self::Fire => 0,
            Self::Smoke => 1,
        }
    }

    /// 从 YOLO 类别索引解析类别。
    #[must_use]
    pub const fn from_class_index(class_index: usize) -> Option<Self> {
        match class_index {
            0 => Some(Self::Fire),
            1 => Some(Self::Smoke),
            _ => None,
        }
    }

    /// 返回标注图使用的英文标签。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fire => "FIRE",
            Self::Smoke => "SMOKE",
        }
    }
}

/// 单个火焰/烟雾检测框。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FlameDetectionBox {
    /// 左上角 x 坐标，单位是原图像素。
    pub x_min: f32,
    /// 左上角 y 坐标，单位是原图像素。
    pub y_min: f32,
    /// 右下角 x 坐标，单位是原图像素。
    pub x_max: f32,
    /// 右下角 y 坐标，单位是原图像素。
    pub y_max: f32,
    /// 检测类别。
    pub detection_class: FlameDetectionClass,
    /// YOLO 类别索引。
    pub class_index: usize,
    /// 检测置信度。
    pub confidence: f32,
}

/// 单个 ONNX 输出摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FlameDetectionOutputSummary {
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

/// 火焰检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FlameDetectionOutputFiles {
    /// 原始输入图副本。
    pub source_input: PathBuf,
    /// 模型实际看到的 resize 输入预览图，包含检测框。
    pub model_input_preview: PathBuf,
    /// ONNX 原始输出摘要 JSON。
    pub raw_outputs_json: PathBuf,
    /// 后处理得到的火焰/烟雾框 JSON。
    pub detected_flames_json: PathBuf,
    /// 画出火焰/烟雾框的输出图。
    pub detected_flames_image: PathBuf,
}

/// 火焰检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FlameDetectionRun {
    /// 输入图片路径。
    pub input_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 检测到的火焰/烟雾框。
    pub detections: Vec<FlameDetectionBox>,
    /// 输出文件路径。
    pub files: FlameDetectionOutputFiles,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<FlameDetectionOutputSummary>,
}

/// 火焰视频检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct FlameVideoDetectionOptions {
    /// Fire/smoke YOLOv8n ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// ffmpeg 可执行文件路径或命令名。
    pub ffmpeg_path: PathBuf,
    /// 抽帧帧率，单位 fps。
    pub sample_fps: u32,
    /// 输出标注视频帧率，单位 fps。
    pub output_fps: u32,
    /// 最多处理多少张抽帧图。为 `None` 时处理全部抽帧。
    pub max_frames: Option<usize>,
    /// 检测置信度阈值。
    pub score_threshold: f32,
    /// NMS 阈值。
    pub nms_threshold: f32,
}

/// 单帧火焰/烟雾检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FlameVideoFrameDetection {
    /// 抽帧序号，从 0 开始。
    pub frame_index: usize,
    /// 按抽帧帧率估算的时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 抽帧图片路径。
    pub frame_path: PathBuf,
    /// 标注后抽帧图片路径。
    pub annotated_frame_path: PathBuf,
    /// 当前帧检测到的火焰/烟雾框。
    pub detections: Vec<FlameDetectionBox>,
}

/// 火焰视频检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FlameVideoDetectionOutputFiles {
    /// 原始输入视频副本。
    pub source_input_video: PathBuf,
    /// ffmpeg 抽帧目录。
    pub extracted_frame_dir: PathBuf,
    /// 标注帧目录。
    pub annotated_frame_dir: PathBuf,
    /// 每帧火焰/烟雾框 JSON。
    pub frame_detections_json: PathBuf,
    /// 标注后视频。
    pub annotated_video: PathBuf,
}

/// 火焰视频检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FlameVideoDetectionRun {
    /// 输入视频绝对路径。
    pub input_video_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 输出文件路径。
    pub files: FlameVideoDetectionOutputFiles,
    /// 每帧检测结果。
    pub frames: Vec<FlameVideoFrameDetection>,
}
