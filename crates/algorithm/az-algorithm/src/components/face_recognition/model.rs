//! 人脸识别模型规格。

use crate::onnx::image::model::{
    OnnxImageModelSpec, TensorElementKind, TensorInputSpec,
};

/// 人脸识别稳定算法 code。
pub const ALGORITHM_CODE: &str = "face_recognition";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/face_recognition";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/face_recognition/models";

const ARCFACE_INPUT: &[usize] = &[1, 3, 112, 112];

/// ArcFace ResNet100 int8 人脸识别模型。
pub const FACE_RECOGNITION_ARCFACE_RESNET100_INT8: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "face_recognition_arcface_resnet100_int8",
    label: "ArcFace ResNet100 int8 face recognition",
    source_repo: "onnxmodelzoo/arcfaceresnet100-11-int8",
    source_file: "arcfaceresnet100-11-int8.onnx",
    local_file: "face_recognition_arcface_resnet100_int8.onnx",
    license: "apache-2.0",
    revision: "c0ec783c5907f34e089495d6d0428e847fcededa",
    input: TensorInputSpec {
        shape: ARCFACE_INPUT,
        element: TensorElementKind::Float32,
    },
    notes: "Embeds aligned face crops for face matching.",
};
