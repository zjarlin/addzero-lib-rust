//! 火焰检测模型规格。

use az_algorithm_onnx::logic_onnx_image::model::{
    OnnxImageModelSpec, TensorElementKind, TensorInputSpec,
};

/// 火焰检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "flame_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/flame_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/models";

const FIRE_VIT_INPUT: &[usize] = &[1, 3, 224, 224];

/// 作为首个本地火焰后端的 ViT 火焰分类模型。
pub const FLAME_DETECTION_VIT_INT8: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "flame_detection_vit_int8",
    label: "ViT int8 fire detection",
    source_repo: "prithivMLmods/Fire-Detection-Engine-ONNX",
    source_file: "onnx/model_int8.onnx",
    local_file: "fire_detection_vit_int8.onnx",
    license: "apache-2.0",
    revision: "02bd7f981aac3e27a75f83e0a3b97dfadaffc228",
    input: TensorInputSpec {
        shape: FIRE_VIT_INPUT,
        element: TensorElementKind::Float32,
    },
    notes: "Image classifier for fire presence; it does not produce bounding boxes.",
};
