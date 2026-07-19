//! 视频算法 pipeline 公开模型。

use std::path::PathBuf;

use image::RgbImage;


/// 一帧已经解码成 RGB 的视频画面。
///
/// 实时场景中该帧可以来自 ffmpeg、摄像头、RTSP、WebRTC 或测试中的内存构造。
#[derive(Clone, Debug, PartialEq)]
pub struct VideoFrame {
    /// 原始视频帧序号，从 0 开始。
    pub frame_index: u64,
    /// 当前帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 帧宽，单位像素。
    pub width: u32,
    /// 帧高，单位像素。
    pub height: u32,
    /// RGB 像素数据。
    pub rgb: RgbImage,
}

/// 可序列化的帧元数据。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct VideoFrameMetadata {
    /// 原始视频帧序号，从 0 开始。
    pub frame_index: u64,
    /// 当前帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 帧宽，单位像素。
    pub width: u32,
    /// 帧高，单位像素。
    pub height: u32,
}

impl VideoFrame {
    /// 返回当前帧的可序列化元数据。
    #[must_use]
    pub const fn metadata(&self) -> VideoFrameMetadata {
        VideoFrameMetadata {
            frame_index: self.frame_index,
            timestamp_ms: self.timestamp_ms,
            width: self.width,
            height: self.height,
        }
    }
}

/// 单个算法在帧流中的运行频率。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum VideoAlgorithmSchedule {
    /// 每一帧都执行。
    EveryFrame,
    /// 每 N 帧执行一次，包含第 0 帧。
    EveryNFrames {
        /// 帧间隔，必须大于 0。
        n: u64,
    },
    /// 按目标 fps 从源帧率折算执行间隔。
    TargetFps {
        /// 目标执行帧率，必须大于 0。
        fps: f32,
    },
}

/// 一帧中的目标框，坐标是原视频帧像素坐标。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VideoBoundingBox {
    /// 左上角 x 坐标。
    pub x_min: f32,
    /// 左上角 y 坐标。
    pub y_min: f32,
    /// 右下角 x 坐标。
    pub x_max: f32,
    /// 右下角 y 坐标。
    pub y_max: f32,
}

/// 单个视觉检测目标。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VideoDetection {
    /// 目标标签，例如 `person`、`helmet`、`smoking`。
    pub label: String,
    /// 置信度，范围由具体算法定义。
    pub confidence: f32,
    /// 可选目标框。
    pub bounding_box: Option<VideoBoundingBox>,
    /// 算法自定义结构化字段。
    pub extra: serde_json::Value,
}

/// 单个算法在某帧上产生的事件。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VideoAlgorithmEvent {
    /// 稳定事件 code，例如 `worker_hit`。
    pub event_code: String,
    /// 事件置信度或强度。
    pub score: f32,
    /// 面向日志和排查的简短说明。
    pub message: String,
    /// 算法自定义结构化字段。
    pub extra: serde_json::Value,
}

/// 单个算法处理一帧后的结构化结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VideoAlgorithmFrameResult {
    /// 算法稳定 code。
    pub algorithm_code: String,
    /// 原始视频帧序号。
    pub frame_index: u64,
    /// 当前帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 检测目标列表。
    pub detections: Vec<VideoDetection>,
    /// 事件列表。
    pub events: Vec<VideoAlgorithmEvent>,
    /// 算法原始或补充 JSON 信息。
    pub raw_json: serde_json::Value,
}

impl VideoAlgorithmFrameResult {
    /// 创建一条没有目标和事件的帧结果。
    #[must_use]
    pub fn empty(algorithm_code: impl Into<String>, frame: &VideoFrame) -> Self {
        Self {
            algorithm_code: algorithm_code.into(),
            frame_index: frame.frame_index,
            timestamp_ms: frame.timestamp_ms,
            detections: Vec::new(),
            events: Vec::new(),
            raw_json: serde_json::Value::Null,
        }
    }
}

/// 可挂载到视频 pipeline 的常驻算法实例。
///
/// 实现方应在构造算法实例时加载模型，`process_frame` 内只做当前帧推理和后处理。
pub trait VideoFrameAlgorithm {
    /// 返回算法稳定 code。
    fn code(&self) -> &'static str;

    /// 处理单帧视频画面。
    ///
    /// # Errors
    /// 模型推理、后处理或算法内部 I/O 失败时返回错误。
    fn process_frame(
        &mut self,
        frame: &VideoFrame,
    ) -> anyhow::Result<VideoAlgorithmFrameResult>;
}

/// 一个算法实例及其帧流调度策略。
pub struct VideoAlgorithmBinding<'a> {
    /// 常驻算法实例。
    pub algorithm: &'a mut dyn VideoFrameAlgorithm,
    /// 当前算法在帧流中的运行频率。
    pub schedule: VideoAlgorithmSchedule,
}

/// 视频 pipeline 执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct VideoPipelineOptions {
    /// 本次任务输出根目录。
    pub output_dir: PathBuf,
    /// 输入视频源帧率，单位 fps。
    pub source_fps: f32,
}

/// 单个算法的运行汇总。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VideoAlgorithmRunSummary {
    /// 算法稳定 code。
    pub algorithm_code: String,
    /// 算法运行频率配置。
    pub schedule: VideoAlgorithmSchedule,
    /// 实际处理的帧序号。
    pub processed_frame_indices: Vec<u64>,
    /// 实际处理帧数。
    pub processed_frame_count: usize,
}

/// 视频 pipeline 输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct VideoPipelineOutputFiles {
    /// 按行写出的每帧算法结果。
    pub frame_results_jsonl: PathBuf,
    /// 本次执行汇总 JSON。
    pub summary_json: PathBuf,
}

/// 视频 pipeline 执行结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VideoPipelineRun {
    /// 输入视频源帧率，单位 fps。
    pub source_fps: f32,
    /// 本次任务输出根目录。
    pub output_dir: PathBuf,
    /// 输出文件路径。
    pub files: VideoPipelineOutputFiles,
    /// 输入帧总数。
    pub total_input_frames: usize,
    /// 所有算法的单帧结果。
    pub frame_results: Vec<VideoAlgorithmFrameResult>,
    /// 每个算法的运行汇总。
    pub algorithm_runs: Vec<VideoAlgorithmRunSummary>,
}
