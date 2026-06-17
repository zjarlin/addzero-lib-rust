//! 人员检测模型规格。

use std::path::PathBuf;

use az_onnx::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputKind, TensorElementKind, TensorInputSpec,
};

/// 人员检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "person_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/person_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/person_detection/models";

/// COCO person 类别 ID。
pub const COCO_PERSON_CLASS_ID: f32 = 1.0;

/// 默认人员置信度阈值。
pub const DEFAULT_SCORE_THRESHOLD: f32 = 0.5;

/// 默认 YOLO 人员置信度阈值。
pub const DEFAULT_YOLO_SCORE_THRESHOLD: f32 = 0.25;

const SSD_MOBILENET_INPUT: &[usize] = &[1, 1200, 1200, 3];
const YOLO11_INPUT: &[usize] = &[1, 3, 640, 640];

/// 复用于人员检测的 COCO SSD MobileNet v1 模型。
pub const PERSON_DETECTION_COCO_SSD_MOBILENET_V1: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "person_detection_coco_ssd_mobilenet_v1",
    label: "COCO SSD MobileNet v1 person detection",
    source_repo: "onnxmodelzoo/ssd_mobilenet_v1_10",
    source_file: "ssd_mobilenet_v1_10.onnx",
    local_file: "coco_ssd_mobilenet_v1_10.onnx",
    license: "apache-2.0",
    revision: "338a91b8e06061536f22129b4bf5227a3d496e8c",
    input: TensorInputSpec {
        shape: SSD_MOBILENET_INPUT,
        element: TensorElementKind::Uint8,
    },
    output_kind: OnnxImageOutputKind::RawTensor,
    notes: "COCO class filtering should select person detections.",
};

/// 复用于人员检测的 YOLO11n COCO ONNX 模型。
pub const PERSON_DETECTION_YOLO11N_COCO: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "person_detection_yolo11n_coco",
    label: "YOLO11n COCO person detection",
    source_repo: "unity/inference-engine-yolo",
    source_file: "models/yolo11n.onnx",
    local_file: "yolo11n_coco.onnx",
    license: "unknown",
    revision: "main",
    input: TensorInputSpec {
        shape: YOLO11_INPUT,
        element: TensorElementKind::Float32,
    },
    output_kind: OnnxImageOutputKind::RawTensor,
    notes: "The ONNX graph uses fp16 tensors with YOLO output shape [1, 84, 8400].",
};

/// 人员检测模型选择。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PersonDetectionModelKind {
    /// COCO SSD MobileNet v1。
    CocoSsdMobileNetV1,
    /// YOLO11n COCO。
    Yolo11nCoco,
}

impl PersonDetectionModelKind {
    /// 返回该模型对应的静态规格。
    #[must_use]
    pub const fn spec(self) -> &'static OnnxImageModelSpec {
        match self {
            Self::CocoSsdMobileNetV1 => &PERSON_DETECTION_COCO_SSD_MOBILENET_V1,
            Self::Yolo11nCoco => &PERSON_DETECTION_YOLO11N_COCO,
        }
    }
}

/// 人员检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct PersonDetectionOptions {
    /// COCO SSD MobileNet ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 使用的人员检测模型类型。
    pub model_kind: PersonDetectionModelKind,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// person 类别置信度阈值。
    pub score_threshold: f32,
}

/// 单个人员检测框。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PersonDetectionBox {
    /// 左上角 x 坐标，单位是原图像素。
    pub x_min: f32,
    /// 左上角 y 坐标，单位是原图像素。
    pub y_min: f32,
    /// 右下角 x 坐标，单位是原图像素。
    pub x_max: f32,
    /// 右下角 y 坐标，单位是原图像素。
    pub y_max: f32,
    /// COCO 类别 ID，person 为 1.0。
    pub class_id: f32,
    /// 人员置信度。
    pub confidence: f32,
}

/// 单个 ONNX 输出摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PersonDetectionOutputSummary {
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

/// 人员检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PersonDetectionOutputFiles {
    /// 原始输入图副本。
    pub source_input: PathBuf,
    /// 模型实际看到的 resize 输入预览图。
    pub model_input_preview: PathBuf,
    /// ONNX 原始输出摘要 JSON。
    pub raw_outputs_json: PathBuf,
    /// COCO SSD 后处理得到的人员框 JSON。
    pub detected_persons_json: PathBuf,
    /// 画出人员框的输出图。
    pub detected_persons_image: PathBuf,
}

/// 人员检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PersonDetectionRun {
    /// 输入图片路径。二进制或 base64 输入会先写入输出目录，再记录该路径。
    pub input_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 检测到的人员框。
    pub persons: Vec<PersonDetectionBox>,
    /// 输出文件路径。
    pub files: PersonDetectionOutputFiles,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<PersonDetectionOutputSummary>,
}

/// 人员视频检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct PersonVideoDetectionOptions {
    /// COCO SSD MobileNet ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 使用的人员检测模型类型。
    pub model_kind: PersonDetectionModelKind,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// ffmpeg 可执行文件路径或命令名。
    pub ffmpeg_path: PathBuf,
    /// 抽帧帧率，单位 fps。
    pub sample_fps: u32,
    /// 最多处理多少张抽帧图。为 `None` 时处理全部抽帧。
    pub max_frames: Option<usize>,
    /// person 类别置信度阈值。
    pub score_threshold: f32,
}

/// 单帧人员检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PersonVideoFrameDetection {
    /// 抽帧序号，从 0 开始。
    pub frame_index: usize,
    /// 按抽帧帧率估算的时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 抽帧图片路径。
    pub frame_path: PathBuf,
    /// 标注后抽帧图片路径。
    pub annotated_frame_path: PathBuf,
    /// 当前帧检测到的人员框。
    pub persons: Vec<PersonDetectionBox>,
}

/// 人员视频检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PersonVideoDetectionOutputFiles {
    /// 原始输入视频副本。
    pub source_input_video: PathBuf,
    /// ffmpeg 抽帧目录。
    pub extracted_frame_dir: PathBuf,
    /// 标注帧目录。
    pub annotated_frame_dir: PathBuf,
    /// 每帧人员框 JSON。
    pub frame_detections_json: PathBuf,
    /// 标注后视频。
    pub annotated_video: PathBuf,
}

/// 人员视频检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PersonVideoDetectionRun {
    /// 输入视频绝对路径。
    pub input_video_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 输出文件路径。
    pub files: PersonVideoDetectionOutputFiles,
    /// 每帧人员检测结果。
    pub frames: Vec<PersonVideoFrameDetection>,
}
