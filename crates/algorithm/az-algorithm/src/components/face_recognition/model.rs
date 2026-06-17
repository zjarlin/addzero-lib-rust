//! 人脸识别模型规格。

use std::path::PathBuf;

use crate::components::face_detection::model::FaceDetectionBox;
use az_onnx::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputFiles, OnnxImageOutputKind, OnnxOutputSummary,
    TensorElementKind, TensorInputSpec,
};

/// 人脸识别稳定算法 code。
pub const ALGORITHM_CODE: &str = "face_recognition";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/face_recognition";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/face_recognition/models";

/// 默认同人脸阈值。SFace 官方 cosine 阈值通常在 0.363 左右，真实业务应按部署数据集重新标定。
pub const DEFAULT_SAME_IDENTITY_THRESHOLD: f32 = 0.363;

const SFACE_INPUT: &[usize] = &[1, 3, 112, 112];

/// SFace 人脸识别模型。
pub const FACE_RECOGNITION_SFACE_2021DEC: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "face_recognition_sface_2021dec",
    label: "OpenCV SFace face recognition",
    source_repo: "opencv/opencv_zoo",
    source_file: "models/face_recognition_sface/face_recognition_sface_2021dec.onnx",
    local_file: "face_recognition_sface_2021dec.onnx",
    license: "apache-2.0",
    revision: "main",
    input: TensorInputSpec {
        shape: SFACE_INPUT,
        element: TensorElementKind::Float32,
    },
    output_kind: OnnxImageOutputKind::Embedding,
    notes: "Embeds YuNet-aligned face crops for face matching.",
};

/// 单张人脸 embedding 提取结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FaceEmbeddingRun {
    /// 原始输入图片路径。
    pub input_path: PathBuf,
    /// 被选中用于识别的人脸框。
    pub detected_face: FaceDetectionBox,
    /// 当前输入检测到的人脸数量。
    pub detected_face_count: usize,
    /// 模型实际看到的裁剪输入与 raw 输出摘要文件。
    pub files: OnnxImageOutputFiles,
    /// embedding 维度。
    pub embedding_dimension: usize,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<OnnxOutputSummary>,
}

/// 两张人脸相似度输出文件。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FaceRecognitionOutputFiles {
    /// 待识别人脸输出文件。
    pub probe: OnnxImageOutputFiles,
    /// 参考人脸输出文件。
    pub reference: OnnxImageOutputFiles,
    /// 相似度 JSON 文件。
    pub similarity_json: PathBuf,
    /// 左右拼接图片和 JSON 标注结果。
    pub comparison_image: PathBuf,
}

/// 两张人脸 embedding 的相似度结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FaceRecognitionRun {
    /// 算法稳定 code。
    pub algorithm_code: String,
    /// 待识别人脸。
    pub probe: FaceEmbeddingRun,
    /// 参考人脸。
    pub reference: FaceEmbeddingRun,
    /// SFace 模型路径。
    pub model_path: PathBuf,
    /// SFace embedding 余弦相似度，范围通常为 -1 到 1。
    pub cosine_similarity: f32,
    /// 本次运行采用的同人脸阈值，作用于 `cosine_similarity`。
    pub same_identity_threshold: f32,
    /// `cosine_similarity >= same_identity_threshold` 的结果。
    pub same_identity: bool,
    /// 输出文件路径。
    pub files: FaceRecognitionOutputFiles,
}
