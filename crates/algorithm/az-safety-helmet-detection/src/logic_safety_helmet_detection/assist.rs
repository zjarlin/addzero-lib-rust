//! 安全帽检测执行辅助函数。

use std::path::{Path, PathBuf};

use az_algorithm_onnx::error::{OnnxImageError, OnnxImageResult};
use az_algorithm_onnx::logic_onnx_image::assist::{
    LocalOnnxSession, run_real_image_model, write_inference_artifacts_from_image,
};
use az_algorithm_onnx::logic_onnx_image::model::OnnxImageRun;
use image::{DynamicImage, RgbImage};

use crate::logic_safety_helmet_detection::model::{
    ALGORITHM_CODE, DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RESULT_DIR,
    SAFETY_HELMET_DETECTION_PPE_YOLO11S,
};

/// 可复用的安全帽 ONNX 推理实例。
///
/// 当前 runner 只保证真实 ONNX 推理和 raw 输出落盘；安全帽类别框后处理还未实现。
#[derive(Debug)]
pub struct SafetyHelmetDetectionRunner {
    model_path: PathBuf,
    session: LocalOnnxSession,
}

impl SafetyHelmetDetectionRunner {
    /// 加载安全帽检测 ONNX 模型，创建可复用 runner。
    ///
    /// # Errors
    /// 模型文件不存在或 ONNX Runtime 加载失败时返回错误。
    pub fn new(model_path: impl AsRef<Path>) -> OnnxImageResult<Self> {
        let model_path = model_path.as_ref().to_path_buf();
        let session = LocalOnnxSession::from_file(&model_path)?;
        Ok(Self {
            model_path,
            session,
        })
    }

    /// 返回 runner 当前使用的模型路径。
    #[must_use]
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// 对内存中的 RGB 帧执行真实安全帽 ONNX 推理，并把本帧 raw 输出写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn run_rgb_image_with_output_dir(
        &mut self,
        image: RgbImage,
        output_dir: impl AsRef<Path>,
    ) -> OnnxImageResult<OnnxImageRun> {
        self.run_dynamic_image_with_output_dir(DynamicImage::ImageRgb8(image), output_dir)
    }

    /// 对内存中的图片执行真实安全帽 ONNX 推理，并把本帧 raw 输出写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn run_dynamic_image_with_output_dir(
        &mut self,
        image: DynamicImage,
        output_dir: impl AsRef<Path>,
    ) -> OnnxImageResult<OnnxImageRun> {
        let output_dir = output_dir.as_ref();
        let (prepared, summary) =
            self.session
                .run_dynamic_image(&SAFETY_HELMET_DETECTION_PPE_YOLO11S, &image)?;
        let files = write_inference_artifacts_from_image(
            ALGORITHM_CODE,
            &image,
            &prepared,
            &summary,
            output_dir,
        )?;
        Ok(OnnxImageRun {
            input_path: files.source_input.clone(),
            model_path: self.model_path.clone(),
            files,
            raw_outputs: summary.outputs,
        })
    }
}

/// 使用默认模型和默认输出目录执行安全帽检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_safety_helmet_detection_from_path(
    image_path: impl AsRef<Path>,
) -> OnnxImageResult<OnnxImageRun> {
    let workspace_root = workspace_root()?;
    run_safety_helmet_detection_from_path_with_output(
        image_path,
        workspace_root.join(DEFAULT_RESULT_DIR),
    )
}

/// 使用默认模型和指定输出目录执行安全帽检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_safety_helmet_detection_from_path_with_output(
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> OnnxImageResult<OnnxImageRun> {
    run_real_image_model(
        ALGORITHM_CODE,
        &SAFETY_HELMET_DETECTION_PPE_YOLO11S,
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
