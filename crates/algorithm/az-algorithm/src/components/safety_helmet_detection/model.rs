//! 安全帽检测模型规格。

use crate::onnx::image::model::{
    OnnxImageModelSpec, TensorElementKind, TensorInputSpec,
};

/// 安全帽检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "safety_helmet_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/safety_helmet_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/safety_helmet_detection/models";

const PPE_YOLO11S_INPUT: &[usize] = &[1, 3, 640, 640];

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
    notes: "PPE detector used as the default local safety helmet backend.",
};
