//! raw ONNX 图片模型的视频帧适配器。

use std::path::{Path, PathBuf};

use az_onnx::onnx::image::assist::{
    LocalOnnxSession, write_inference_artifacts_from_image,
};
use az_onnx::onnx::image::model::OnnxImageModelSpec;
use image::DynamicImage;
use serde_json::json;

use anyhow::anyhow;
use crate::video_pipeline::model::{
    VideoAlgorithmFrameResult, VideoFrame, VideoFrameAlgorithm,
};

/// 将任意本地 ONNX 图片模型挂到视频逐帧 pipeline 的 raw 输出适配器。
///
/// 这个适配器只负责真实模型推理和 raw 输出落盘，不把原始张量伪装成检测框。
#[derive(Debug)]
pub struct OnnxRawImageVideoAlgorithm {
    algorithm_code: &'static str,
    model_spec: OnnxImageModelSpec,
    model_path: PathBuf,
    output_dir: PathBuf,
    session: LocalOnnxSession,
}

impl OnnxRawImageVideoAlgorithm {
    /// 创建一个常驻 ONNX Session 的视频帧算法实例。
    ///
    /// # Errors
    /// 模型文件不存在或 ONNX Runtime 加载失败时返回错误。
    pub fn new(
        algorithm_code: &'static str,
        model_spec: OnnxImageModelSpec,
        model_path: impl AsRef<Path>,
        output_dir: impl Into<PathBuf>,
    ) -> anyhow::Result<Self> {
        let model_path = model_path.as_ref().to_path_buf();
        let session = LocalOnnxSession::from_file(&model_path)
            .map_err(|source| anyhow!(source.to_string()))?;
        Ok(Self {
            algorithm_code,
            model_spec,
            model_path,
            output_dir: output_dir.into(),
            session,
        })
    }

    /// 返回当前模型文件路径。
    #[must_use]
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// 返回当前算法输出根目录。
    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

impl VideoFrameAlgorithm for OnnxRawImageVideoAlgorithm {
    fn code(&self) -> &'static str {
        self.algorithm_code
    }

    fn process_frame(
        &mut self,
        frame: &VideoFrame,
    ) -> anyhow::Result<VideoAlgorithmFrameResult> {
        let image = DynamicImage::ImageRgb8(frame.rgb.clone());
        let frame_output_dir = self
            .output_dir
            .join(format!("frame_{:05}", frame.frame_index));
        let (prepared, summary) = self
            .session
            .run_dynamic_image(&self.model_spec, &image)
            .map_err(|source| anyhow!(source.to_string()))?;
        let files = write_inference_artifacts_from_image(
            self.algorithm_code,
            &self.model_spec,
            &image,
            &prepared,
            &summary,
            &frame_output_dir,
        )
        .map_err(|source| anyhow!(source.to_string()))?;

        let value = json!({
            "model_code": self.model_spec.code,
            "model_label": self.model_spec.label,
            "model_path": self.model_path,
            "source_input": files.source_input,
            "model_input_preview": files.model_input_preview,
            "raw_outputs_json": files.raw_outputs_json,
            "raw_output_review": files.raw_output_review,
            "raw_output_count": summary.outputs.len(),
            "说明": "当前适配器只输出真实 ONNX raw 摘要；需要对应算法 crate 实现后处理后才会产生检测框或文本"
        });
        let result = VideoAlgorithmFrameResult {
            algorithm_code: self.algorithm_code.to_owned(),
            frame_index: frame.frame_index,
            timestamp_ms: frame.timestamp_ms,
            detections: Vec::new(),
            events: Vec::new(),
            raw_json: value,
        };
        Ok(result)
    }
}
