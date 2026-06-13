//! 火焰检测执行辅助函数。

use std::path::{Path, PathBuf};

use az_algorithm_onnx::error::{OnnxImageError, OnnxImageResult};
use az_algorithm_onnx::logic_onnx_image::assist::run_real_image_model;
use az_algorithm_onnx::logic_onnx_image::model::OnnxImageRun;

use crate::logic_flame_detection::model::{
    ALGORITHM_CODE, DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RESULT_DIR, FLAME_DETECTION_VIT_INT8,
};

/// 使用默认模型和默认输出目录执行火焰检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_flame_detection_from_path(image_path: impl AsRef<Path>) -> OnnxImageResult<OnnxImageRun> {
    let workspace_root = workspace_root()?;
    run_flame_detection_from_path_with_output(image_path, workspace_root.join(DEFAULT_RESULT_DIR))
}

/// 使用默认模型和指定输出目录执行火焰检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_flame_detection_from_path_with_output(
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> OnnxImageResult<OnnxImageRun> {
    run_real_image_model(
        ALGORITHM_CODE,
        &FLAME_DETECTION_VIT_INT8,
        crate_root().join(DEFAULT_MODEL_RESOURCE_DIR),
        image_path,
        output_dir,
    )
}

fn workspace_root() -> OnnxImageResult<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .map_err(|source| OnnxImageError::io(PathBuf::from(env!("CARGO_MANIFEST_DIR")), source))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
