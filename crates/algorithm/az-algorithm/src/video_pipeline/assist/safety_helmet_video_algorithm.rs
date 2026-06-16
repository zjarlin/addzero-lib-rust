//! 安全帽检测的视频帧适配器。

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::components::safety_helmet_detection::assist::SafetyHelmetDetectionRunner;
use crate::components::safety_helmet_detection::model::{
    ALGORITHM_CODE, SafetyHelmetDetectionOptions,
};
use crate::video_pipeline::model::{
    VideoAlgorithmEvent, VideoAlgorithmFrameResult, VideoBoundingBox, VideoDetection, VideoFrame,
    VideoFrameAlgorithm,
};

/// 将安全帽 PPE YOLO 检测挂到视频逐帧 pipeline。
#[derive(Debug)]
pub struct SafetyHelmetVideoAlgorithm {
    runner: SafetyHelmetDetectionRunner,
    output_dir: PathBuf,
}

impl SafetyHelmetVideoAlgorithm {
    /// 创建安全帽视频帧算法实例。
    ///
    /// # Errors
    /// 模型文件不存在、阈值非法或 ONNX Runtime 加载失败时返回错误。
    pub fn new(
        options: SafetyHelmetDetectionOptions,
        output_dir: impl Into<PathBuf>,
    ) -> anyhow::Result<Self> {
        let runner = SafetyHelmetDetectionRunner::new(options)?;
        Ok(Self {
            runner,
            output_dir: output_dir.into(),
        })
    }

    /// 返回当前模型文件路径。
    #[must_use]
    pub fn model_path(&self) -> &Path {
        self.runner.model_path()
    }

    /// 返回当前算法输出根目录。
    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

impl VideoFrameAlgorithm for SafetyHelmetVideoAlgorithm {
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
                label: detection.detection_class.label().to_owned(),
                confidence: detection.confidence,
                bounding_box: Some(VideoBoundingBox {
                    x_min: detection.x_min,
                    y_min: detection.y_min,
                    x_max: detection.x_max,
                    y_max: detection.y_max,
                }),
                extra: json!({
                    "class_index": detection.class_index,
                    "is_helmet_alarm": detection.detection_class.is_helmet_alarm(),
                }),
            })
            .collect::<Vec<_>>();
        let events = run
            .detections
            .iter()
            .filter(|detection| detection.detection_class.is_helmet_alarm())
            .map(|detection| VideoAlgorithmEvent {
                event_code: "safety_helmet_missing".to_owned(),
                score: detection.confidence,
                message: "检测到未佩戴安全帽".to_owned(),
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
                "detected_safety_helmets_json": run.files.detected_safety_helmets_json,
                "detected_safety_helmets_image": run.files.detected_safety_helmets_image,
                "raw_output_count": run.raw_outputs.len(),
            }),
        })
    }
}
