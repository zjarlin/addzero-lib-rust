//! 安全帽检测模型规格。

use std::path::PathBuf;

use az_onnx::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputKind, TensorElementKind, TensorInputSpec,
};

/// 安全帽检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "safety_helmet_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/safety_helmet_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/safety_helmet_detection/models";

const PPE_YOLO11S_INPUT: &[usize] = &[1, 3, 640, 640];

/// 默认检测置信度阈值。
pub const DEFAULT_SCORE_THRESHOLD: f32 = 0.25;

/// 默认 YOLO NMS 阈值。
pub const DEFAULT_NMS_THRESHOLD: f32 = 0.45;

/// 用于安全帽检测的 YOLO11s PPE 模型。
pub const SAFETY_HELMET_DETECTION_PPE_YOLO11S: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "safety_helmet_detection_ppe_yolo11s",
    label: "YOLO11s PPE safety helmet detection",
    source_repo: "nduka1999/nd_ppe_yolo11s",
    source_file: "best.onnx",
    local_file: "safety_helmet_detection_ppe_yolo11s.onnx",
    license: "mit",
    revision: "90f3e8915ef403dbbc77bb6ba713916321e2970f",
    input: TensorInputSpec {
        shape: PPE_YOLO11S_INPUT,
        element: TensorElementKind::Float32,
    },
    output_kind: OnnxImageOutputKind::RawTensor,
    notes: "PPE detector used as the default local safety helmet backend.",
};

/// 安全帽检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct SafetyHelmetDetectionOptions {
    /// PPE YOLO11s ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// 检测置信度阈值。
    pub score_threshold: f32,
    /// NMS 阈值。
    pub nms_threshold: f32,
}

/// PPE 模型支持的检测类别。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetyHelmetDetectionClass {
    /// 已佩戴安全帽。
    Hardhat,
    /// 未佩戴安全帽。
    NoHardhat,
    /// 已穿反光背心。
    Vest,
    /// 未穿反光背心。
    NoVest,
    /// 人员。
    Person,
}

impl SafetyHelmetDetectionClass {
    /// 返回 YOLO 类别索引。
    #[must_use]
    pub const fn class_index(self) -> usize {
        match self {
            Self::Hardhat => 0,
            Self::NoHardhat => 1,
            Self::Vest => 2,
            Self::NoVest => 3,
            Self::Person => 4,
        }
    }

    /// 从 YOLO 类别索引解析类别。
    #[must_use]
    pub const fn from_class_index(class_index: usize) -> Option<Self> {
        match class_index {
            0 => Some(Self::Hardhat),
            1 => Some(Self::NoHardhat),
            2 => Some(Self::Vest),
            3 => Some(Self::NoVest),
            4 => Some(Self::Person),
            _ => None,
        }
    }

    /// 返回稳定标签。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hardhat => "hardhat",
            Self::NoHardhat => "no_hardhat",
            Self::Vest => "vest",
            Self::NoVest => "no_vest",
            Self::Person => "person",
        }
    }

    /// 返回是否应视为安全帽告警类。
    #[must_use]
    pub const fn is_helmet_alarm(self) -> bool {
        matches!(self, Self::NoHardhat)
    }
}

/// 单个 PPE 检测框。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SafetyHelmetDetectionBox {
    /// 左上角 x 坐标，单位是原图像素。
    pub x_min: f32,
    /// 左上角 y 坐标，单位是原图像素。
    pub y_min: f32,
    /// 右下角 x 坐标，单位是原图像素。
    pub x_max: f32,
    /// 右下角 y 坐标，单位是原图像素。
    pub y_max: f32,
    /// 检测类别。
    pub detection_class: SafetyHelmetDetectionClass,
    /// YOLO 类别索引。
    pub class_index: usize,
    /// 检测置信度。
    pub confidence: f32,
}

/// 单个 ONNX 输出摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SafetyHelmetDetectionOutputSummary {
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

/// 安全帽检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SafetyHelmetDetectionOutputFiles {
    /// 原始输入图副本。
    pub source_input: PathBuf,
    /// 模型实际看到的 resize 输入预览图。
    pub model_input_preview: PathBuf,
    /// ONNX 原始输出摘要 JSON。
    pub raw_outputs_json: PathBuf,
    /// 后处理得到的 PPE 框 JSON。
    pub detected_safety_helmets_json: PathBuf,
    /// 画出 PPE 框的输出图。
    pub detected_safety_helmets_image: PathBuf,
}

/// 安全帽检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SafetyHelmetDetectionRun {
    /// 输入图片路径。
    pub input_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 检测到的 PPE 框。
    pub detections: Vec<SafetyHelmetDetectionBox>,
    /// 输出文件路径。
    pub files: SafetyHelmetDetectionOutputFiles,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<SafetyHelmetDetectionOutputSummary>,
}
