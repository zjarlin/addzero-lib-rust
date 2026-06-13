//! OCR 文字识别模型规格。

use az_algorithm_onnx::logic_onnx_image::model::{
    OnnxImageModelSpec, TensorElementKind, TensorInputSpec,
};

/// OCR 文字检测稳定算法 code。
pub const DETECTION_ALGORITHM_CODE: &str = "ocr_text_detection";

/// OCR 文字识别稳定算法 code。
pub const RECOGNITION_ALGORITHM_CODE: &str = "ocr_text_recognition";

/// 默认检测输出目录。
pub const DEFAULT_DETECTION_RESULT_DIR: &str = "target/az-algorithm-results/ocr_text_detection";

/// 默认识别输出目录。
pub const DEFAULT_RECOGNITION_RESULT_DIR: &str =
    "target/az-algorithm-results/ocr_text_recognition";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/models";

const PADDLE_DET_INPUT: &[usize] = &[1, 3, 640, 640];
const PADDLE_REC_INPUT: &[usize] = &[1, 3, 48, 320];

/// PaddleOCR v3 文字检测模型。
pub const OCR_PADDLE_V3_DETECTION: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "ocr_paddle_v3_detection",
    label: "PaddleOCR v3 text detection",
    source_repo: "monkt/paddleocr-onnx",
    source_file: "detection/v3/det.onnx",
    local_file: "ocr_paddle_v3_det.onnx",
    license: "apache-2.0",
    revision: "7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65",
    input: TensorInputSpec {
        shape: PADDLE_DET_INPUT,
        element: TensorElementKind::Float32,
    },
    notes: "Detects text regions before recognition.",
};

/// PaddleOCR 中文文字识别模型。
pub const OCR_PADDLE_CHINESE_RECOGNITION: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "ocr_paddle_chinese_recognition",
    label: "PaddleOCR Chinese text recognition",
    source_repo: "monkt/paddleocr-onnx",
    source_file: "languages/chinese/rec.onnx",
    local_file: "ocr_paddle_chinese_rec.onnx",
    license: "apache-2.0",
    revision: "7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65",
    input: TensorInputSpec {
        shape: PADDLE_REC_INPUT,
        element: TensorElementKind::Float32,
    },
    notes: "Requires ocr_paddle_chinese_dict.txt for CTC label decoding.",
};
