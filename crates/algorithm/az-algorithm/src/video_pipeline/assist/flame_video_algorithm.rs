//! 火焰与烟雾检测的视频帧适配器。

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::components::flame_detection::assist::FlameDetectionRunner;
use crate::components::flame_detection::model::{ALGORITHM_CODE, FlameDetectionOptions};
use crate::video_pipeline::model::{
    VideoAlgorithmEvent, VideoAlgorithmFrameResult, VideoBoundingBox, VideoDetection, VideoFrame,
    VideoFrameAlgorithm,
};

/// 将火焰/烟雾 YOLO 检测挂到视频逐帧 pipeline。
#[derive(Debug)]
pub struct FlameVideoAlgorithm {
    runner: FlameDetectionRunner,
    output_dir: PathBuf,
}

impl FlameVideoAlgorithm {
    /// 创建火焰/烟雾视频帧算法实例。
    ///
    /// # Errors
    /// 模型文件不存在、阈值非法或 ONNX Runtime 加载失败时返回错误。
    pub fn new(options: FlameDetectionOptions, output_dir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let runner = FlameDetectionRunner::new(options)?;
        Ok(Self {
            runner,
            output_dir: output_dir.into(),
        })
    }

    /// 返回当前模型文件路径。
    #[must_use]
    pub fn model_path(&self) -> &Path {
        &self.runner.options().model_path
    }

    /// 返回当前算法输出根目录。
    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

impl VideoFrameAlgorithm for FlameVideoAlgorithm {
    fn code(&self) -> &'static str {
        ALGORITHM_CODE
    }

    fn process_frame(
        &mut self,
        frame: &VideoFrame,
    ) -> anyhow::Result<VideoAlgorithmFrameResult> {
        let frame_output_dir = self
            .output_dir
            .join(format!("frame_{:05}", frame.frame_index));
        let run = self
            .runner
            .detect_rgb_image_with_output_dir(frame.rgb.clone(), &frame_output_dir)?;
        let detections = run
            .detections
            .iter()
            .map(|detection| VideoDetection {
                label: detection.detection_class.label().to_ascii_lowercase(),
                confidence: detection.confidence,
                bounding_box: Some(VideoBoundingBox {
                    x_min: detection.x_min,
                    y_min: detection.y_min,
                    x_max: detection.x_max,
                    y_max: detection.y_max,
                }),
                extra: json!({
                    "class_index": detection.class_index,
                }),
            })
            .collect::<Vec<_>>();
        let events = run
            .detections
            .iter()
            .map(|detection| VideoAlgorithmEvent {
                event_code: format!("{}_detected", detection.detection_class.label().to_ascii_lowercase()),
                score: detection.confidence,
                message: format!("检测到{}", detection.detection_class.label()),
                extra: json!({
                    "class_index": detection.class_index,
                    "box": {
                        "x_min": detection.x_min,
                        "y_min": detection.y_min,
                        "x_max": detection.x_max,
                        "y_max": detection.y_max,
                    },
                }),
            })
            .collect::<Vec<_>>();

        Ok(VideoAlgorithmFrameResult {
            algorithm_code: ALGORITHM_CODE.to_owned(),
            frame_index: frame.frame_index,
            timestamp_ms: frame.timestamp_ms,
            detections,
            events,
            raw_json: json!({
                "source_input": run.files.source_input,
                "model_input_preview": run.files.model_input_preview,
                "raw_outputs_json": run.files.raw_outputs_json,
                "detected_flames_json": run.files.detected_flames_json,
                "detected_flames_image": run.files.detected_flames_image,
                "raw_output_count": run.raw_outputs.len(),
            }),
        })
    }
}
