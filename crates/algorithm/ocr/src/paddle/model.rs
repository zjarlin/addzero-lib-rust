//! OCR 文字识别模型规格。

use std::path::PathBuf;

use az_onnx::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputKind, TensorElementKind, TensorInputSpec,
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
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/ocr_text_recognition/models";

/// PaddleOCR 中文识别字典文件。
pub const OCR_PADDLE_CHINESE_DICT_FILE: &str = "ocr_paddle_chinese_dict.txt";

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
    output_kind: OnnxImageOutputKind::RawTensor,
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
    output_kind: OnnxImageOutputKind::RawTensor,
    notes: "Requires ocr_paddle_chinese_dict.txt for CTC label decoding.",
};

/// 单个 OCR CTC 时间步的解码 token。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OcrTextToken {
    /// CTC 时间步下标。
    pub time_step: usize,
    /// 模型输出类别下标。
    pub class_index: usize,
    /// 字典 token。
    pub token: String,
    /// 该 token 的原始模型分数。
    pub score: f32,
}

/// OCR 文本行在原图上的像素范围。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OcrTextBoundingBox {
    /// 左上角 x 坐标。
    pub x_min: u32,
    /// 左上角 y 坐标。
    pub y_min: u32,
    /// 右下角 x 坐标。
    pub x_max: u32,
    /// 右下角 y 坐标。
    pub y_max: u32,
    /// 检测热力图平均分。
    pub score: f32,
}

/// 单行 OCR 识别结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OcrTextLine {
    /// 行序号，按从上到下排序。
    pub index: usize,
    /// 行文本。
    pub text: String,
    /// 原图像素范围。
    pub bounding_box: OcrTextBoundingBox,
    /// CTC token 明细。
    pub tokens: Vec<OcrTextToken>,
}

/// OCR 文本后处理输出文件。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct OcrTextRecognitionOutputFiles {
    /// 识别出的纯文本文件。
    pub recognized_text: PathBuf,
    /// 识别文本、token 与来源文件的 JSON 文件。
    pub recognized_text_json: PathBuf,
}
