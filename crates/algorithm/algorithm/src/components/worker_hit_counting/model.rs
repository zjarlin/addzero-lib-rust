//! 工人有效敲击计数公开模型。

use std::path::PathBuf;

/// 视频中单个人员的稳定跟踪标识。
pub type WorkerTrackId = u64;

/// 工人动作状态。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WorkerActionState {
    /// 没有观察到明确敲击动作。
    Idle,
    /// 观察到敲击准备或挥动过程，但尚未形成一次完整敲击。
    Striking,
    /// 本帧确认形成一次有效敲击事件。
    ValidHit,
    /// 本帧形成了敲击候选，但没有命中有效目标。
    InvalidHitCandidate,
}

/// 视觉目标类型。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum VisualTargetKind {
    /// 悬挂金属板，只有命中该类目标才计为有效敲击。
    HangingMetalPanel,
    /// 流水线台体或边缘，不计为有效敲击。
    ConveyorBody,
    /// 支架、护栏等无效结构。
    SupportStructure,
    /// 未知目标。
    Unknown,
}

/// 敲击候选无效原因。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InvalidHitReason {
    /// 接触点没有命中悬挂金属板。
    ContactOutsideHangingMetalPanel,
    /// 接触点落在明确无效目标上。
    ContactOnInvalidTarget,
    /// 悬挂金属板没有出现足够视觉响应。
    MissingTargetResponse,
    /// 动作置信度或接触置信度不足。
    LowConfidence,
}

/// 归一化二维点。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizedPoint {
    /// 横坐标，范围通常为 0.0 到 1.0。
    pub x: f32,
    /// 纵坐标，范围通常为 0.0 到 1.0。
    pub y: f32,
}

/// 归一化边界框。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizedBoundingBox {
    /// 左上角横坐标，范围通常为 0.0 到 1.0。
    pub x: f32,
    /// 左上角纵坐标，范围通常为 0.0 到 1.0。
    pub y: f32,
    /// 宽度，范围通常为 0.0 到 1.0。
    pub width: f32,
    /// 高度，范围通常为 0.0 到 1.0。
    pub height: f32,
}

/// 接触点所在目标的视觉观测。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VisualTargetObservation {
    /// 目标 ID，由上游检测/跟踪模块维护。
    pub target_id: u64,
    /// 目标类型。
    pub kind: VisualTargetKind,
    /// 目标框。
    pub target_box: NormalizedBoundingBox,
    /// 接触点落入该目标的置信度，范围 0.0 到 1.0。
    pub containment_score: f32,
}

/// 单帧中某个人员的纯视觉观测。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActionObservation {
    /// 稳定人员跟踪 ID。
    pub person_id: WorkerTrackId,
    /// 视频帧序号。
    pub frame_index: u64,
    /// 帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 人员框。
    pub person_box: NormalizedBoundingBox,
    /// 视觉模型或规则输出的敲击动作置信度，范围 0.0 到 1.0。
    pub strike_score: f32,
    /// 工具或手部与目标接触的视觉置信度，范围 0.0 到 1.0。
    pub contact_score: f32,
    /// 工具/手部末端接触点，使用归一化坐标。
    pub contact_point: Option<NormalizedPoint>,
    /// 接触点所在目标。
    pub contacted_target: Option<VisualTargetObservation>,
    /// 悬挂金属板在接触后的视觉响应置信度，范围 0.0 到 1.0。
    pub target_response_score: f32,
}

/// 工人有效敲击计数配置。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHitCountConfig {
    /// 进入 `Striking` 状态所需的敲击动作置信度。
    pub strike_score_threshold: f32,
    /// 形成敲击候选所需的接触置信度。
    pub contact_score_threshold: f32,
    /// 形成有效敲击所需的悬挂金属板响应置信度。
    pub target_response_score_threshold: f32,
    /// 单个人员两次有效敲击之间的最小间隔，单位毫秒。
    pub min_hit_gap_ms: u64,
    /// 单个人员两次无效敲击候选之间的最小记录间隔，单位毫秒。
    pub min_invalid_candidate_gap_ms: u64,
    /// 没有继续观察到敲击动作后，保持 `Striking` 状态的时间，单位毫秒。
    pub strike_hold_ms: u64,
}

impl Default for WorkerHitCountConfig {
    fn default() -> Self {
        Self {
            strike_score_threshold: 0.55,
            contact_score_threshold: 0.70,
            target_response_score_threshold: 0.45,
            min_hit_gap_ms: 220,
            min_invalid_candidate_gap_ms: 220,
            strike_hold_ms: 180,
        }
    }
}

/// 单个人员的一次有效敲击记录。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHitRecord {
    /// 该人员自己的有效敲击序号。
    pub hit_index: usize,
    /// 人员跟踪 ID。
    pub person_id: WorkerTrackId,
    /// 触发敲击事件的帧序号。
    pub frame_index: u64,
    /// 触发敲击事件的时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 当帧人员框。
    pub person_box: NormalizedBoundingBox,
    /// 有效悬挂金属板目标 ID。
    pub target_id: u64,
    /// 有效接触点。
    pub contact_point: NormalizedPoint,
    /// 触发事件时的敲击动作置信度。
    pub strike_score: f32,
    /// 触发事件时的接触置信度。
    pub contact_score: f32,
    /// 触发事件时的悬挂金属板响应置信度。
    pub target_response_score: f32,
}

/// 无效敲击候选记录。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InvalidWorkerHitCandidate {
    /// 该人员自己的无效候选序号。
    pub candidate_index: usize,
    /// 人员跟踪 ID。
    pub person_id: WorkerTrackId,
    /// 候选帧序号。
    pub frame_index: u64,
    /// 候选时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 当帧人员框。
    pub person_box: NormalizedBoundingBox,
    /// 接触点；如果上游没有定位到接触点则为空。
    pub contact_point: Option<NormalizedPoint>,
    /// 接触目标；如果上游没有归属到目标则为空。
    pub contacted_target: Option<VisualTargetObservation>,
    /// 无效原因。
    pub reason: InvalidHitReason,
    /// 候选动作置信度。
    pub strike_score: f32,
    /// 候选接触置信度。
    pub contact_score: f32,
    /// 候选目标响应置信度。
    pub target_response_score: f32,
}

/// 单个人员的动作统计。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActionTrack {
    /// 人员跟踪 ID。
    pub person_id: WorkerTrackId,
    /// 最近一帧动作状态。
    pub state: WorkerActionState,
    /// 该人员有效敲击次数。
    pub valid_hit_count: usize,
    /// 该人员每一次有效敲击记录。
    pub valid_hits: Vec<WorkerHitRecord>,
    /// 该人员无效敲击候选次数。
    pub invalid_candidate_count: usize,
    /// 该人员每一次无效敲击候选记录。
    pub invalid_candidates: Vec<InvalidWorkerHitCandidate>,
    /// 最近观测帧序号。
    pub last_frame_index: u64,
    /// 最近观测时间戳，单位毫秒。
    pub last_seen_timestamp_ms: u64,
    /// 最近人员框。
    pub last_person_box: NormalizedBoundingBox,
}

/// 按人员分组的工人有效敲击统计结果。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHitCount {
    /// 每个人员各自的动作状态、有效敲击次数、有效记录和无效候选记录。
    pub workers: Vec<WorkerActionTrack>,
}

impl WorkerHitCount {
    /// 按人员 ID 读取该人员的动作统计。
    #[must_use]
    pub fn worker(&self, person_id: WorkerTrackId) -> Option<&WorkerActionTrack> {
        self.workers
            .iter()
            .find(|worker| worker.person_id == person_id)
    }

    /// 按人员 ID 读取有效敲击次数。
    #[must_use]
    pub fn valid_hit_count_of(&self, person_id: WorkerTrackId) -> Option<usize> {
        self.worker(person_id).map(|worker| worker.valid_hit_count)
    }
}

/// 单帧处理后的人员动作状态记录。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActionFrameRecord {
    /// 稳定人员跟踪 ID。
    pub person_id: WorkerTrackId,
    /// 视频帧序号。
    pub frame_index: u64,
    /// 帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 当帧人员框。
    pub person_box: NormalizedBoundingBox,
    /// 当帧处理后的动作状态。
    pub state: WorkerActionState,
    /// 处理完该帧后的累计有效敲击次数。
    pub valid_hit_count: usize,
    /// 处理完该帧后的累计无效候选次数。
    pub invalid_candidate_count: usize,
    /// 如果该帧新增了一次有效敲击，这里记录该敲击事件。
    pub new_valid_hit: Option<WorkerHitRecord>,
    /// 如果该帧新增了一次无效候选，这里记录该候选事件。
    pub new_invalid_candidate: Option<InvalidWorkerHitCandidate>,
}

/// 工人敲击动作时间线。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHitTimeline {
    /// 每帧、每人的动作状态记录。
    pub frame_records: Vec<WorkerActionFrameRecord>,
    /// 全部帧处理完成后的最终计数结果。
    pub final_count: WorkerHitCount,
}

/// YOLOv8n pose 模型文件名。
pub const DEFAULT_POSE_MODEL_FILE_NAME: &str = "yolov8n_pose.onnx";

/// 一行视频标注入口默认抽帧帧率。
pub const DEFAULT_WORKER_HIT_SAMPLE_FPS: u32 = 10;

/// 一行视频标注入口默认输出视频帧率。
pub const DEFAULT_WORKER_HIT_OUTPUT_FPS: u32 = 30;

/// 一行视频标注入口默认 pose person 置信度阈值。
pub const DEFAULT_WORKER_HIT_POSE_SCORE_THRESHOLD: f32 = 0.02;

/// 一行视频标注入口默认关键点置信度阈值。
pub const DEFAULT_WORKER_HIT_KEYPOINT_SCORE_THRESHOLD: f32 = 0.02;

/// 一行视频标注入口默认悬挂金属板区域。
pub const DEFAULT_WORKER_HIT_TARGET_ROI: VisualTargetObservation = VisualTargetObservation {
    target_id: 1,
    kind: VisualTargetKind::HangingMetalPanel,
    target_box: NormalizedBoundingBox {
        x: 0.30,
        y: 0.46,
        width: 0.45,
        height: 0.32,
    },
    containment_score: 1.0,
};

/// 工人敲击视频分析配置。
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerHitVideoAnalysisOptions {
    /// YOLO pose ONNX 模型绝对路径。
    pub pose_model_path: PathBuf,
    /// ffmpeg 可执行文件路径或命令名。
    pub ffmpeg_path: PathBuf,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// 抽帧帧率，单位 fps。
    pub sample_fps: u32,
    /// 标注视频输出帧率，单位 fps。
    pub output_fps: u32,
    /// 最多处理多少张抽帧图。为 `None` 时处理全部抽帧。
    pub max_frames: Option<usize>,
    /// pose person 候选置信度阈值。
    pub pose_score_threshold: f32,
    /// 关键点置信度阈值。
    pub keypoint_score_threshold: f32,
    /// 工具或手腕接触区域，业务侧按现场画面配置。
    pub target_roi: VisualTargetObservation,
    /// 敲击计数配置。
    pub hit_count_config: WorkerHitCountConfig,
}

/// YOLO pose 关键点。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PoseKeypoint {
    /// 关键点横坐标，单位是原图像素。
    pub x: f32,
    /// 关键点纵坐标，单位是原图像素。
    pub y: f32,
    /// 关键点置信度。
    pub confidence: f32,
}

/// 单个姿态人员候选。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerPoseDetection {
    /// pose 候选序号；视频流水线关联跨帧人员后，该值为稳定人员 ID 减一。
    pub local_person_index: usize,
    /// 候选人员框。
    pub person_box: NormalizedBoundingBox,
    /// pose person 置信度。
    pub confidence: f32,
    /// COCO 17 个关键点。
    pub keypoints: Vec<PoseKeypoint>,
}

/// 单帧姿态检测结果。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerPoseFrame {
    /// 抽帧序号。
    pub frame_index: u64,
    /// 帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 当前帧宽度，单位像素。
    pub frame_width: u32,
    /// 当前帧高度，单位像素。
    pub frame_height: u32,
    /// 抽帧图片路径。
    pub frame_path: PathBuf,
    /// 标注后抽帧图片路径。
    pub annotated_frame_path: PathBuf,
    /// 当前帧 pose 人员候选。
    pub poses: Vec<WorkerPoseDetection>,
}

/// 工人敲击视频分析输出文件路径。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHitVideoOutputFiles {
    /// 原始输入视频副本。
    pub source_input_video: PathBuf,
    /// ffmpeg 抽帧目录。
    pub extracted_frame_dir: PathBuf,
    /// 标注帧目录。
    pub annotated_frame_dir: PathBuf,
    /// pose 逐帧结果 JSON。
    pub pose_frames_json: PathBuf,
    /// 由 pose 和 ROI 规则生成的动作观测 JSON。
    pub action_observations_json: PathBuf,
    /// 工人敲击时间线 JSON。
    pub worker_hit_timeline_json: PathBuf,
    /// 标注后视频。
    pub annotated_video: PathBuf,
}

/// 工人敲击视频分析结果。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHitVideoAnalysisRun {
    /// 输入视频绝对路径。
    pub input_video_path: PathBuf,
    /// pose 模型路径。
    pub pose_model_path: PathBuf,
    /// 输出文件路径。
    pub files: WorkerHitVideoOutputFiles,
    /// 逐帧 pose 检测结果。
    pub pose_frames: Vec<WorkerPoseFrame>,
    /// 从 pose 和 ROI 规则生成的动作观测。
    pub action_observations: Vec<WorkerActionObservation>,
    /// 敲击时间线。
    pub timeline: WorkerHitTimeline,
}

impl WorkerHitVideoAnalysisRun {
    /// 按人员 ID 读取有效敲击次数。
    #[must_use]
    pub fn valid_hit_count_of(&self, person_id: WorkerTrackId) -> Option<usize> {
        self.timeline.final_count.valid_hit_count_of(person_id)
    }

    /// 按人员 ID 读取该人员的动作统计。
    #[must_use]
    pub fn worker(&self, person_id: WorkerTrackId) -> Option<&WorkerActionTrack> {
        self.timeline.final_count.worker(person_id)
    }
}

pub(crate) enum HitCandidateClassification {
    Valid {
        target: VisualTargetObservation,
        contact_point: NormalizedPoint,
    },
    Invalid {
        reason: InvalidHitReason,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct WorkerActionAccumulator {
    pub(crate) person_id: WorkerTrackId,
    pub(crate) state: WorkerActionState,
    pub(crate) hits: Vec<WorkerHitRecord>,
    pub(crate) invalid_candidates: Vec<InvalidWorkerHitCandidate>,
    pub(crate) last_frame_index: u64,
    pub(crate) last_seen_timestamp_ms: u64,
    pub(crate) last_strike_timestamp_ms: Option<u64>,
    pub(crate) last_hit_timestamp_ms: Option<u64>,
    pub(crate) last_invalid_candidate_timestamp_ms: Option<u64>,
    pub(crate) last_person_box: NormalizedBoundingBox,
}
