//! 二维码识别的视频帧适配器。

use std::fs;
use std::path::{Path, PathBuf};

use crate::components::qr_code_recognition::assist::decode_qr_codes_from_luma8;
use crate::components::qr_code_recognition::model::{
    ALGORITHM_CODE, ImagePoint, QrCodeRecognition,
};
use image::DynamicImage;
use serde_json::json;

use crate::video_pipeline::model::{
    VideoAlgorithmEvent, VideoAlgorithmFrameResult, VideoBoundingBox, VideoDetection, VideoFrame,
    VideoFrameAlgorithm,
};

/// 将纯 Rust 二维码识别挂到视频逐帧 pipeline。
#[derive(Debug)]
pub struct QrCodeVideoAlgorithm {
    output_dir: PathBuf,
}

impl QrCodeVideoAlgorithm {
    /// 创建二维码视频帧算法实例。
    #[must_use]
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
        }
    }

    /// 返回当前算法输出根目录。
    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

impl VideoFrameAlgorithm for QrCodeVideoAlgorithm {
    fn code(&self) -> &'static str {
        ALGORITHM_CODE
    }

    fn process_frame(
        &mut self,
        frame: &VideoFrame,
    ) -> anyhow::Result<VideoAlgorithmFrameResult> {
        let image = DynamicImage::ImageRgb8(frame.rgb.clone()).to_luma8();
        let results = decode_qr_codes_from_luma8(image);
        let frame_output_dir = self.output_dir.join(format!("frame_{:05}", frame.frame_index));
        fs::create_dir_all(&frame_output_dir)
            .map_err(|source| path_error(frame_output_dir.clone(), source))?;
        let decoded_json = frame_output_dir.join("decoded_qr_codes.json");
        let json_text = serde_json::to_string_pretty(&results)?;
        fs::write(&decoded_json, json_text)
            .map_err(|source| path_error(decoded_json.clone(), source))?;

        let detections = results
            .iter()
            .map(qr_code_to_detection)
            .collect::<Vec<_>>();
        let events = results
            .iter()
            .map(|result| VideoAlgorithmEvent {
                event_code: "qr_code_decoded".to_owned(),
                score: 1.0,
                message: result.payload.clone(),
                extra: json!({
                    "version": result.version,
                    "ecc_level": result.ecc_level,
                    "mask": result.mask,
                    "bounds": result.bounds,
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
                "decoded_qr_codes_json": decoded_json,
                "decoded_count": results.len(),
            }),
        })
    }
}

fn qr_code_to_detection(result: &QrCodeRecognition) -> VideoDetection {
    VideoDetection {
        label: "qr_code".to_owned(),
        confidence: 1.0,
        bounding_box: Some(bounds_to_box(&result.bounds)),
        extra: json!({
            "payload": result.payload,
            "version": result.version,
            "ecc_level": result.ecc_level,
            "mask": result.mask,
            "bounds": result.bounds,
        }),
    }
}

fn bounds_to_box(bounds: &[ImagePoint; 4]) -> VideoBoundingBox {
    let (mut x_min, mut y_min) = (f32::MAX, f32::MAX);
    let (mut x_max, mut y_max) = (f32::MIN, f32::MIN);
    for point in bounds {
        let x = point.x as f32;
        let y = point.y as f32;
        x_min = x_min.min(x);
        y_min = y_min.min(y);
        x_max = x_max.max(x);
        y_max = y_max.max(y);
    }
    VideoBoundingBox {
        x_min,
        y_min,
        x_max,
        y_max,
    }
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow::anyhow!("filesystem error at `{}`: {source}", path.display())
}
