//! OCR 文字识别执行辅助函数。

use std::path::{Path, PathBuf};

use anyhow::Context;
use crate::onnx::image::assist::run_real_image_model;
use crate::onnx::image::model::OnnxImageRun;

use crate::components::ocr_text_recognition::model::{
    DEFAULT_DETECTION_RESULT_DIR, DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RECOGNITION_RESULT_DIR,
    DETECTION_ALGORITHM_CODE, OCR_PADDLE_CHINESE_RECOGNITION, OCR_PADDLE_V3_DETECTION,
    RECOGNITION_ALGORITHM_CODE,
};

/// OCR 检测与识别两阶段真实推理结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OcrTextRecognitionRun {
    /// 文字检测模型输出。
    pub detection: OnnxImageRun,
    /// 中文识别模型输出。
    pub recognition: OnnxImageRun,
}

/// 使用默认模型和默认输出目录执行 OCR 两阶段真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_ocr_text_recognition_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<OcrTextRecognitionRun> {
    let workspace_root = workspace_root()?;
    run_ocr_text_recognition_from_path_with_output(
        image_path,
        workspace_root.join(DEFAULT_DETECTION_RESULT_DIR),
        workspace_root.join(DEFAULT_RECOGNITION_RESULT_DIR),
    )
}

/// 使用默认模型和指定输出目录执行 OCR 两阶段真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_ocr_text_recognition_from_path_with_output(
    image_path: impl AsRef<Path>,
    detection_output_dir: impl AsRef<Path>,
    recognition_output_dir: impl AsRef<Path>,
) -> anyhow::Result<OcrTextRecognitionRun> {
    let image_path = image_path.as_ref();
    let resource_dir = crate_root().join(DEFAULT_MODEL_RESOURCE_DIR);
    let detection = run_real_image_model(
        DETECTION_ALGORITHM_CODE,
        &OCR_PADDLE_V3_DETECTION,
        &resource_dir,
        image_path,
        detection_output_dir,
    )?;
    let recognition = run_real_image_model(
        RECOGNITION_ALGORITHM_CODE,
        &OCR_PADDLE_CHINESE_RECOGNITION,
        &resource_dir,
        image_path,
        recognition_output_dir,
    )?;
    Ok(OcrTextRecognitionRun {
        detection,
        recognition,
    })
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .with_context(|| format!("failed to resolve workspace root from `{}`", env!("CARGO_MANIFEST_DIR")))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
