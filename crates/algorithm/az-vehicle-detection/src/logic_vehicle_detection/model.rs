//! 车辆检测模型规格。

use az_algorithm_onnx::logic_onnx_image::model::{
    OnnxImageModelSpec, TensorElementKind, TensorInputSpec,
};

/// 车辆检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "vehicle_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/vehicle_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/models";

const SSD_MOBILENET_INPUT: &[usize] = &[1, 1200, 1200, 3];

/// 复用于车辆检测的 COCO SSD MobileNet v1 模型。
pub const VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "vehicle_detection_coco_ssd_mobilenet_v1",
    label: "COCO SSD MobileNet v1 vehicle detection",
    source_repo: "onnxmodelzoo/ssd_mobilenet_v1_10",
    source_file: "ssd_mobilenet_v1_10.onnx",
    local_file: "coco_ssd_mobilenet_v1_10.onnx",
    license: "apache-2.0",
    revision: "338a91b8e06061536f22129b4bf5227a3d496e8c",
    input: TensorInputSpec {
        shape: SSD_MOBILENET_INPUT,
        element: TensorElementKind::Uint8,
    },
    notes: "COCO class filtering should select car, bus, truck, motorcycle, and bicycle detections.",
};
