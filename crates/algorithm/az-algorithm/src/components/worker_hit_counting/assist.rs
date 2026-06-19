//! 工人有效敲击计数辅助函数。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ab_glyph::{FontArc, PxScale};
use anyhow::{anyhow, bail};
use az_str::sanitize::sanitize_path_file_stem_or;
use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::{
    draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut, text_size,
};
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::components::worker_hit_counting::model::{
    DEFAULT_POSE_MODEL_FILE_NAME, DEFAULT_WORKER_HIT_KEYPOINT_SCORE_THRESHOLD,
    DEFAULT_WORKER_HIT_OUTPUT_FPS, DEFAULT_WORKER_HIT_POSE_SCORE_THRESHOLD,
    DEFAULT_WORKER_HIT_SAMPLE_FPS, DEFAULT_WORKER_HIT_TARGET_ROI, HitCandidateClassification,
    InvalidHitReason, InvalidWorkerHitCandidate, NormalizedBoundingBox, NormalizedPoint,
    PoseKeypoint, VisualTargetKind, VisualTargetObservation, WorkerActionAccumulator,
    WorkerActionFrameRecord, WorkerActionObservation, WorkerActionState, WorkerActionTrack,
    WorkerHitCount, WorkerHitCountConfig, WorkerHitRecord, WorkerHitTimeline,
    WorkerHitVideoAnalysisOptions, WorkerHitVideoAnalysisRun, WorkerHitVideoOutputFiles,
    WorkerPoseDetection, WorkerPoseFrame, WorkerTrackId,
};

const POSE_MODEL_INPUT_SHAPE: &[usize] = &[1, 3, 640, 640];
const POSE_OUTPUT_NAME: &str = "output0";
const POSE_OUTPUT_CHANNELS: usize = 56;
const POSE_OUTPUT_CANDIDATES: usize = 8400;
const POSE_NMS_THRESHOLD: f32 = 0.30;
const POSE_MIN_BOX_WIDTH: f32 = 0.045;
const POSE_MIN_BOX_HEIGHT: f32 = 0.045;
const POSE_MIN_BOX_AREA: f32 = 0.008;
const POSE_MIN_VISIBLE_KEYPOINTS: usize = 8;
const POSE_MIN_STRONG_KEYPOINTS: usize = 4;
const POSE_EDGE_PARTIAL_MAX_HEIGHT: f32 = 0.09;
const MATERIAL_STACK_FALSE_PERSON_ZONE: NormalizedBoundingBox = NormalizedBoundingBox {
    x: 0.18,
    y: 0.50,
    width: 0.42,
    height: 0.22,
};
const MATERIAL_STACK_FALSE_PERSON_MAX_HEIGHT: f32 = 0.18;
const MATERIAL_STACK_FALSE_PERSON_MAX_AREA: f32 = 0.035;
const TRACK_MAX_NORMALIZED_DISTANCE: f32 = 0.18;
const TRACK_MAX_NORMALIZED_DISTANCE_WITH_MISSES: f32 = 0.28;
const TRACK_MAX_MISSED_FRAMES: u64 = 30;
const TRACKLET_MERGE_MAX_GAP_FRAMES: u64 = 80;
const TRACKLET_MERGE_MAX_ENDPOINT_DISTANCE: f32 = 0.16;
const TRACKLET_MERGE_MAX_VERTICAL_GAP: f32 = 0.12;
const TRACKLET_MERGE_MAX_HORIZONTAL_GAP: f32 = 0.16;
const TRACKLET_MERGE_MAX_AREA_RATIO: f32 = 2.20;
const ANNOTATION_PANEL_HOLD_FRAMES: u64 = 30;
const TARGET_RESPONSE_CONTACT_WINDOW_X: f32 = 0.045;
const TARGET_RESPONSE_CONTACT_WINDOW_Y: f32 = 0.075;
const TARGET_RESPONSE_SAMPLE_STRIDE: u32 = 3;
const TARGET_RESPONSE_BACKGROUND_STRIDE_MULTIPLIER: u32 = 5;
const TARGET_RESPONSE_MIN_SAMPLES: usize = 24;
const TARGET_RESPONSE_BACKGROUND_WEIGHT: f32 = 1.5;
const TARGET_RESPONSE_DIFF_FLOOR: f32 = 0.025;
const TARGET_RESPONSE_TOP_PERCENTILE: f32 = 0.85;
const TARGET_RESPONSE_CHANGED_PIXEL_FLOOR: f32 = 0.08;
const TARGET_RESPONSE_CHANGED_RATIO_BASELINE: f32 = 0.08;
const TARGET_RESPONSE_OTHER_PERSON_MASK_EXPAND: f32 = 0.018;
const LEFT_WRIST_INDEX: usize = 9;
const RIGHT_WRIST_INDEX: usize = 10;

impl WorkerHitCountConfig {
    fn validate(self) -> anyhow::Result<()> {
        validate_unit_score("strike_score_threshold", self.strike_score_threshold)?;
        validate_unit_score("contact_score_threshold", self.contact_score_threshold)?;
        validate_unit_score(
            "target_response_score_threshold",
            self.target_response_score_threshold,
        )?;
        if self.min_hit_gap_ms == 0 {
            bail!("invalid visual action input: min_hit_gap_ms must be greater than 0");
        }
        if self.min_invalid_candidate_gap_ms == 0 {
            bail!(
                "invalid visual action input: min_invalid_candidate_gap_ms must be greater than 0"
            );
        }
        Ok(())
    }
}

impl WorkerActionAccumulator {
    fn from_observation(observation: &WorkerActionObservation) -> Self {
        Self {
            person_id: observation.person_id,
            state: WorkerActionState::Idle,
            hits: Vec::new(),
            invalid_candidates: Vec::new(),
            last_frame_index: observation.frame_index,
            last_seen_timestamp_ms: observation.timestamp_ms,
            last_strike_timestamp_ms: None,
            last_hit_timestamp_ms: None,
            last_invalid_candidate_timestamp_ms: None,
            last_person_box: observation.person_box,
        }
    }

    fn apply_observation(
        &mut self,
        observation: &WorkerActionObservation,
        config: WorkerHitCountConfig,
    ) {
        self.last_frame_index = observation.frame_index;
        self.last_seen_timestamp_ms = observation.timestamp_ms;
        self.last_person_box = observation.person_box;

        let is_striking = observation.strike_score >= config.strike_score_threshold;
        let is_contact = observation.contact_score >= config.contact_score_threshold;

        if is_striking {
            self.last_strike_timestamp_ms = Some(observation.timestamp_ms);
            self.state = WorkerActionState::Striking;
        } else if self.last_strike_timestamp_ms.is_some_and(|timestamp_ms| {
            observation.timestamp_ms.saturating_sub(timestamp_ms) <= config.strike_hold_ms
        }) {
            self.state = WorkerActionState::Striking;
        } else {
            self.state = WorkerActionState::Idle;
        }

        if is_striking && is_contact {
            match classify_hit_candidate(observation, config) {
                HitCandidateClassification::Valid {
                    target,
                    contact_point,
                } => {
                    if self.can_record_hit(observation.timestamp_ms, config) {
                        self.record_hit(observation, target, contact_point);
                        self.state = WorkerActionState::ValidHit;
                    }
                }
                HitCandidateClassification::Invalid { reason } => {
                    if self.can_record_invalid_candidate(observation.timestamp_ms, config) {
                        self.record_invalid_candidate(observation, reason);
                        self.state = WorkerActionState::InvalidHitCandidate;
                    }
                }
            }
        }
    }

    fn can_record_hit(&self, timestamp_ms: u64, config: WorkerHitCountConfig) -> bool {
        self.last_hit_timestamp_ms.is_none_or(|last_hit_ms| {
            timestamp_ms.saturating_sub(last_hit_ms) >= config.min_hit_gap_ms
        })
    }

    fn can_record_invalid_candidate(
        &self,
        timestamp_ms: u64,
        config: WorkerHitCountConfig,
    ) -> bool {
        self.last_invalid_candidate_timestamp_ms
            .is_none_or(|last_invalid_ms| {
                timestamp_ms.saturating_sub(last_invalid_ms) >= config.min_invalid_candidate_gap_ms
            })
    }

    fn record_hit(
        &mut self,
        observation: &WorkerActionObservation,
        target: VisualTargetObservation,
        contact_point: NormalizedPoint,
    ) {
        let hit_index = self.hits.len();
        self.last_hit_timestamp_ms = Some(observation.timestamp_ms);
        self.hits.push(WorkerHitRecord {
            hit_index,
            person_id: observation.person_id,
            frame_index: observation.frame_index,
            timestamp_ms: observation.timestamp_ms,
            person_box: observation.person_box,
            target_id: target.target_id,
            contact_point,
            strike_score: observation.strike_score,
            contact_score: observation.contact_score,
            target_response_score: observation.target_response_score,
        });
    }

    fn record_invalid_candidate(
        &mut self,
        observation: &WorkerActionObservation,
        reason: InvalidHitReason,
    ) {
        let candidate_index = self.invalid_candidates.len();
        self.last_invalid_candidate_timestamp_ms = Some(observation.timestamp_ms);
        self.invalid_candidates.push(InvalidWorkerHitCandidate {
            candidate_index,
            person_id: observation.person_id,
            frame_index: observation.frame_index,
            timestamp_ms: observation.timestamp_ms,
            person_box: observation.person_box,
            contact_point: observation.contact_point,
            contacted_target: observation.contacted_target,
            reason,
            strike_score: observation.strike_score,
            contact_score: observation.contact_score,
            target_response_score: observation.target_response_score,
        });
    }

    fn finish(self) -> WorkerActionTrack {
        WorkerActionTrack {
            person_id: self.person_id,
            state: self.state,
            valid_hit_count: self.hits.len(),
            valid_hits: self.hits,
            invalid_candidate_count: self.invalid_candidates.len(),
            invalid_candidates: self.invalid_candidates,
            last_frame_index: self.last_frame_index,
            last_seen_timestamp_ms: self.last_seen_timestamp_ms,
            last_person_box: self.last_person_box,
        }
    }
}

/// 按人员统计纯视觉有效敲击动作。
///
/// 输入必须是已完成人员检测/跟踪、目标识别和接触点归属后的逐帧观测。
/// 只有接触点命中悬挂金属板且目标出现足够视觉响应时，才计入有效敲击。
/// 命中流水线台体边缘、支架或无目标响应的动作会记录为无效候选，不增加有效次数。
///
/// # Errors
/// 当配置阈值、观测分数、点或框无效时返回错误。
pub fn count_worker_hits_by_person_from_visual_observations(
    observations: &[WorkerActionObservation],
    config: WorkerHitCountConfig,
) -> anyhow::Result<WorkerHitCount> {
    Ok(record_worker_hit_timeline_from_visual_observations(observations, config)?.final_count)
}

/// 按人员生成纯视觉有效敲击动作时间线。
///
/// 该接口在最终统计之外保留每一帧处理后的人员动作状态、累计次数，以及该帧新增的
/// 有效敲击或无效候选，适合后续把动作状态和敲击次数画回视频。
///
/// # Errors
/// 当配置阈值、观测分数、点或框无效时返回错误。
pub fn record_worker_hit_timeline_from_visual_observations(
    observations: &[WorkerActionObservation],
    config: WorkerHitCountConfig,
) -> anyhow::Result<WorkerHitTimeline> {
    config.validate()?;

    let mut sorted_observations = observations.iter().collect::<Vec<_>>();
    sorted_observations.sort_by_key(|observation| {
        (
            observation.timestamp_ms,
            observation.frame_index,
            observation.person_id,
        )
    });

    let mut workers = BTreeMap::<WorkerTrackId, WorkerActionAccumulator>::new();
    let mut frame_records = Vec::new();
    for observation in sorted_observations {
        validate_observation(observation)?;
        let accumulator = workers
            .entry(observation.person_id)
            .or_insert_with(|| WorkerActionAccumulator::from_observation(observation));
        let hit_count_before = accumulator.hits.len();
        let invalid_candidate_count_before = accumulator.invalid_candidates.len();

        accumulator.apply_observation(observation, config);

        frame_records.push(WorkerActionFrameRecord {
            person_id: observation.person_id,
            frame_index: observation.frame_index,
            timestamp_ms: observation.timestamp_ms,
            person_box: observation.person_box,
            state: accumulator.state,
            valid_hit_count: accumulator.hits.len(),
            invalid_candidate_count: accumulator.invalid_candidates.len(),
            new_valid_hit: accumulator.hits.get(hit_count_before).cloned(),
            new_invalid_candidate: accumulator
                .invalid_candidates
                .get(invalid_candidate_count_before)
                .cloned(),
        });
    }

    Ok(WorkerHitTimeline {
        frame_records,
        final_count: WorkerHitCount {
            workers: workers
                .into_values()
                .map(WorkerActionAccumulator::finish)
                .collect(),
        },
    })
}

/// 一行完成工人敲击视频标注。
///
/// 传入原始视频路径，函数会使用 crate 内置 YOLO pose 模型、默认悬挂金属板 ROI
/// 和系统 `ffmpeg` 完成抽帧、纯视觉动作计数、逐帧标注和重新编码。
/// 返回值是标注后视频的绝对路径。
///
/// # Errors
/// 视频文件不存在、ffmpeg 不可用、ONNX 推理失败、图片处理失败或输出文件写入失败时返回错误。
///
/// # Examples
///
/// ```no_run
/// # use az_algorithm::components::worker_hit_counting::assist::annotate_worker_hits_video;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let annotated = annotate_worker_hits_video("/Users/zjarlin/Desktop/input.mp4")?;
/// println!("{}", annotated.display());
/// # Ok(())
/// # }
/// ```
pub fn annotate_worker_hits_video(video_path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let run = analyze_worker_hits_video(video_path)?;
    Ok(run.files.annotated_video)
}

/// 一行完成工人敲击视频标注，并把结果写入调用方指定的输出视频路径。
///
/// 调用方只需要提供输入视频 URL/路径和输出视频 URL/路径；模型、ROI、抽帧和编码配置
/// 继续使用默认值。中间产物会写入输出视频同级的隐藏工作目录。
///
/// # Errors
/// 输入视频文件不存在、输出父目录无法创建、ffmpeg 不可用、ONNX 推理失败、图片处理失败
/// 或输出文件写入失败时返回错误。
///
/// # Examples
///
/// ```no_run
/// # use az_algorithm::components::worker_hit_counting::assist::annotate_worker_hits_video_to_path;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let output = annotate_worker_hits_video_to_path(
///     "/Users/zjarlin/Desktop/input.mp4",
///     "/Users/zjarlin/Desktop/output.mp4",
/// )?;
/// println!("{}", output.display());
/// # Ok(())
/// # }
/// ```
pub fn annotate_worker_hits_video_to_path(
    input_video_url: impl AsRef<Path>,
    output_video_url: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let input_video_path = input_video_url.as_ref();
    let output_video_path = output_video_url.as_ref();
    let output_parent = output_video_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_parent)
        .map_err(|source| path_error(output_parent.to_path_buf(), source))?;

    let mut options = default_worker_hit_video_analysis_options(input_video_path)?;
    options.output_dir = output_parent.join(format!(
        ".{}-worker-hit-counting",
        sanitize_path_file_stem_or(output_video_path, "input_video")
    ));

    let run = analyze_worker_hits_in_video_from_path(input_video_path, &options)?;
    fs::copy(&run.files.annotated_video, output_video_path)
        .map_err(|source| path_error(output_video_path.to_path_buf(), source))?;
    Ok(output_video_path.to_path_buf())
}

/// 一行完成工人敲击视频分析并返回应用层可直接读取的结构体。
///
/// 该接口和 [`annotate_worker_hits_video`] 使用相同默认配置，会同时生成标注视频和
/// `worker_hit_timeline.json`。应用层通常直接读取返回值里的
/// `timeline.final_count.workers`，或调用
/// [`WorkerHitVideoAnalysisRun::valid_hit_count_of`](crate::components::worker_hit_counting::model::WorkerHitVideoAnalysisRun::valid_hit_count_of)。
///
/// # Errors
/// 视频文件不存在、ffmpeg 不可用、ONNX 推理失败、图片处理失败或输出文件写入失败时返回错误。
///
/// # Examples
///
/// ```no_run
/// # use az_algorithm::components::worker_hit_counting::assist::analyze_worker_hits_video;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let run = analyze_worker_hits_video("/Users/zjarlin/Desktop/input.mp4")?;
/// let person_5_hits = run.valid_hit_count_of(5).unwrap_or(0);
/// println!("person 5 hits: {person_5_hits}");
/// # Ok(())
/// # }
/// ```
pub fn analyze_worker_hits_video(
    video_path: impl AsRef<Path>,
) -> anyhow::Result<WorkerHitVideoAnalysisRun> {
    let video_path = video_path.as_ref();
    let options = default_worker_hit_video_analysis_options(video_path)?;
    analyze_worker_hits_in_video_from_path(video_path, &options)
}

/// 为一行视频标注入口生成默认配置。
///
/// 需要现场调 ROI、阈值或抽帧帧率时，调用方可以拿到该默认配置后修改字段，再调用
/// [`analyze_worker_hits_in_video_from_path`]。
///
/// # Errors
/// 输入视频路径无法规范化，或当前工作目录无法读取时返回错误。
pub fn default_worker_hit_video_analysis_options(
    video_path: impl AsRef<Path>,
) -> anyhow::Result<WorkerHitVideoAnalysisOptions> {
    let input_video_path = std::fs::canonicalize(video_path.as_ref())
        .map_err(|source| path_error(video_path.as_ref().to_path_buf(), source))?;
    let output_dir = default_output_root()?
        .join("target")
        .join("az-algorithm-results")
        .join("worker-hit-counting")
        .join(sanitize_path_file_stem_or(&input_video_path, "input_video"));

    Ok(WorkerHitVideoAnalysisOptions {
        pose_model_path: default_pose_model_path(),
        ffmpeg_path: default_ffmpeg_path(),
        output_dir,
        sample_fps: DEFAULT_WORKER_HIT_SAMPLE_FPS,
        output_fps: DEFAULT_WORKER_HIT_OUTPUT_FPS,
        max_frames: None,
        pose_score_threshold: DEFAULT_WORKER_HIT_POSE_SCORE_THRESHOLD,
        keypoint_score_threshold: DEFAULT_WORKER_HIT_KEYPOINT_SCORE_THRESHOLD,
        target_roi: DEFAULT_WORKER_HIT_TARGET_ROI,
        hit_count_config: WorkerHitCountConfig {
            strike_score_threshold: 0.18,
            contact_score_threshold: 0.70,
            target_response_score_threshold: 0.35,
            min_hit_gap_ms: 360,
            min_invalid_candidate_gap_ms: 360,
            strike_hold_ms: 220,
        },
    })
}

fn default_pose_model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("worker_hit_counting")
        .join("models")
        .join(DEFAULT_POSE_MODEL_FILE_NAME)
}

fn default_output_root() -> anyhow::Result<PathBuf> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    std::fs::canonicalize(&workspace_root).map_err(|source| path_error(workspace_root, source))
}

fn default_ffmpeg_path() -> PathBuf {
    ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("ffmpeg"))
}

/// 从真实视频抽帧，使用 YOLO pose 生成动作观测，再按人员输出敲击时间线。
///
/// 该接口仍然依赖业务侧配置 `target_roi`。模型负责输出人体姿态，是否构成敲击由
/// 手腕进入 ROI、连续帧运动幅度和目标区域响应规则决定。
///
/// # Errors
/// 视频文件、ffmpeg、ONNX 推理、图片处理或输出文件写入失败时返回错误。
pub fn analyze_worker_hits_in_video_from_path(
    video_path: impl AsRef<Path>,
    options: &WorkerHitVideoAnalysisOptions,
) -> anyhow::Result<WorkerHitVideoAnalysisRun> {
    validate_video_analysis_options(options)?;
    let video_path = std::fs::canonicalize(video_path.as_ref())
        .map_err(|source| path_error(video_path.as_ref().to_path_buf(), source))?;
    recreate_dir(&options.output_dir)?;

    let files = WorkerHitVideoOutputFiles {
        source_input_video: options.output_dir.join("source_input.mp4"),
        extracted_frame_dir: options.output_dir.join("extracted_frames"),
        annotated_frame_dir: options.output_dir.join("annotated_frames"),
        pose_frames_json: options.output_dir.join("pose_frames.json"),
        action_observations_json: options.output_dir.join("action_observations.json"),
        worker_hit_timeline_json: options.output_dir.join("worker_hit_timeline.json"),
        annotated_video: options.output_dir.join("annotated_worker_hits.mp4"),
    };
    fs::create_dir_all(&files.extracted_frame_dir)
        .map_err(|source| path_error(files.extracted_frame_dir.clone(), source))?;
    fs::create_dir_all(&files.annotated_frame_dir)
        .map_err(|source| path_error(files.annotated_frame_dir.clone(), source))?;
    fs::copy(&video_path, &files.source_input_video)
        .map_err(|source| path_error(video_path.clone(), source))?;

    extract_video_frames(&video_path, &files.extracted_frame_dir, options)?;
    let frame_paths = collected_frame_paths(&files.extracted_frame_dir, options.max_frames)?;
    let mut pose_frames = Vec::new();
    let mut pose_session = WorkerPoseSession::load(&options.pose_model_path)?;
    for (index, frame_path) in frame_paths.iter().enumerate() {
        let frame_index = index as u64;
        let timestamp_ms = frame_timestamp_ms(index, options.sample_fps);
        let image = image::open(frame_path)?;
        let poses =
            pose_session.detect_worker_poses_in_image(&image, options.pose_score_threshold)?;
        let annotated_frame_path = files
            .annotated_frame_dir
            .join(format!("frame_{index:05}.png"));
        pose_frames.push(WorkerPoseFrame {
            frame_index,
            timestamp_ms,
            frame_width: image.width(),
            frame_height: image.height(),
            frame_path: frame_path.clone(),
            annotated_frame_path,
            poses,
        });
    }

    assign_stable_pose_track_ids(&mut pose_frames);
    let action_observations = action_observations_from_pose_frames(
        &pose_frames,
        options.target_roi,
        options.keypoint_score_threshold,
    )?;
    let timeline = record_worker_hit_timeline_from_visual_observations(
        &action_observations,
        options.hit_count_config,
    )?;

    write_worker_hit_annotation_frames(&pose_frames, &timeline, options.target_roi.target_box)?;
    write_json_file(&files.pose_frames_json, &pose_frames)?;
    write_json_file(&files.action_observations_json, &action_observations)?;
    write_json_file(&files.worker_hit_timeline_json, &timeline)?;
    encode_annotated_video(&files.annotated_frame_dir, &files.annotated_video, options)?;

    Ok(WorkerHitVideoAnalysisRun {
        input_video_path: video_path,
        pose_model_path: options.pose_model_path.clone(),
        files,
        pose_frames,
        action_observations,
        timeline,
    })
}

fn classify_hit_candidate(
    observation: &WorkerActionObservation,
    config: WorkerHitCountConfig,
) -> HitCandidateClassification {
    let Some(target) = observation.contacted_target else {
        return HitCandidateClassification::Invalid {
            reason: InvalidHitReason::ContactOutsideHangingMetalPanel,
        };
    };
    let Some(contact_point) = observation.contact_point else {
        return HitCandidateClassification::Invalid {
            reason: InvalidHitReason::ContactOutsideHangingMetalPanel,
        };
    };

    if target.kind != VisualTargetKind::HangingMetalPanel {
        return HitCandidateClassification::Invalid {
            reason: InvalidHitReason::ContactOnInvalidTarget,
        };
    }
    if observation.target_response_score < config.target_response_score_threshold {
        return HitCandidateClassification::Invalid {
            reason: InvalidHitReason::MissingTargetResponse,
        };
    }

    HitCandidateClassification::Valid {
        target,
        contact_point,
    }
}

fn validate_observation(observation: &WorkerActionObservation) -> anyhow::Result<()> {
    validate_unit_score("strike_score", observation.strike_score)?;
    validate_unit_score("contact_score", observation.contact_score)?;
    validate_unit_score("target_response_score", observation.target_response_score)?;
    validate_normalized_box("person_box", observation.person_box)?;
    if let Some(contact_point) = observation.contact_point {
        validate_normalized_point("contact_point", contact_point)?;
    }
    if let Some(target) = observation.contacted_target {
        validate_unit_score("containment_score", target.containment_score)?;
        validate_normalized_box("target_box", target.target_box)?;
    }
    Ok(())
}

fn validate_unit_score(field: &str, value: f32) -> anyhow::Result<()> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        bail!("invalid visual action input: {field} must be finite and within 0.0..=1.0")
    }
}

fn validate_normalized_point(field: &str, point: NormalizedPoint) -> anyhow::Result<()> {
    if point.x.is_finite()
        && point.y.is_finite()
        && (0.0..=1.0).contains(&point.x)
        && (0.0..=1.0).contains(&point.y)
    {
        Ok(())
    } else {
        bail!(
            "invalid visual action input: {field} coordinates must be finite and within 0.0..=1.0"
        )
    }
}

fn validate_normalized_box(field: &str, bbox: NormalizedBoundingBox) -> anyhow::Result<()> {
    let right = bbox.x + bbox.width;
    let bottom = bbox.y + bbox.height;
    if bbox.x.is_finite()
        && bbox.y.is_finite()
        && bbox.width.is_finite()
        && bbox.height.is_finite()
        && bbox.width >= 0.0
        && bbox.height >= 0.0
        && bbox.x >= 0.0
        && bbox.y >= 0.0
        && right <= 1.0
        && bottom <= 1.0
    {
        Ok(())
    } else {
        bail!(
            "invalid visual action input: {field} must be finite and normalized within frame bounds"
        )
    }
}

#[derive(Clone, Debug)]
struct PreparedPoseImage {
    tensor_data: Vec<f32>,
    scale: f32,
    pad_x: f32,
    pad_y: f32,
}

#[derive(Clone, Copy, Debug)]
struct PoseImageTransform {
    scale: f32,
    pad_x: f32,
    pad_y: f32,
}

#[derive(Clone, Debug)]
struct PoseOutputTensor {
    data: Vec<f32>,
}

struct WorkerPoseSession {
    session: Session,
}

impl WorkerPoseSession {
    fn load(model_path: &Path) -> anyhow::Result<Self> {
        if !model_path.is_file() {
            let source =
                std::io::Error::new(std::io::ErrorKind::NotFound, "pose model file not found");
            let path = model_path.to_path_buf();
            let error = path_error(path, source);
            return Err(error);
        }

        let mut builder = Session::builder()?;
        Ok(Self {
            session: builder.commit_from_file(model_path)?,
        })
    }

    fn detect_worker_poses_in_image(
        &mut self,
        image: &DynamicImage,
        pose_score_threshold: f32,
    ) -> anyhow::Result<Vec<WorkerPoseDetection>> {
        let prepared = prepare_pose_image(image);
        let transform = PoseImageTransform {
            scale: prepared.scale,
            pad_x: prepared.pad_x,
            pad_y: prepared.pad_y,
        };
        let output = self.run_pose_model(prepared.tensor_data.clone())?;
        decode_pose_output(
            &output,
            image.width(),
            image.height(),
            transform,
            pose_score_threshold,
        )
    }

    fn run_pose_model(&mut self, tensor_data: Vec<f32>) -> anyhow::Result<PoseOutputTensor> {
        let input_array = ArrayD::from_shape_vec(IxDyn(POSE_MODEL_INPUT_SHAPE), tensor_data)
            .map_err(|source| anyhow!("invalid pose tensor shape: {source}"))?;
        let input = Tensor::from_array(input_array)?;
        let outputs = self.session.run(ort::inputs![input])?;
        let value = outputs.get(POSE_OUTPUT_NAME).ok_or_else(|| {
            anyhow!("invalid pose tensor shape: missing pose output `{POSE_OUTPUT_NAME}`")
        })?;
        let ValueType::Tensor { ty, .. } = value.dtype() else {
            bail!(
                "invalid pose tensor shape: pose output is not tensor: {}",
                value.dtype()
            );
        };
        if !matches!(ty, TensorElementType::Float32) {
            bail!("invalid pose tensor shape: pose output expected f32, got {ty}");
        }
        let (shape, data) = value.try_extract_tensor::<f32>()?;
        let shape = shape.iter().copied().collect::<Vec<_>>();
        if shape.as_slice()
            != [
                1,
                POSE_OUTPUT_CHANNELS as i64,
                POSE_OUTPUT_CANDIDATES as i64,
            ]
        {
            bail!(
                "invalid pose tensor shape: pose output expected [1, {POSE_OUTPUT_CHANNELS}, {POSE_OUTPUT_CANDIDATES}], got {shape:?}"
            );
        }
        Ok(PoseOutputTensor {
            data: data.to_vec(),
        })
    }
}

fn prepare_pose_image(image: &DynamicImage) -> PreparedPoseImage {
    let source = image.to_rgb8();
    let source_width = source.width();
    let source_height = source.height();
    let scale = (640.0 / source_width as f32).min(640.0 / source_height as f32);
    let resized_width = (source_width as f32 * scale).round().clamp(1.0, 640.0) as u32;
    let resized_height = (source_height as f32 * scale).round().clamp(1.0, 640.0) as u32;
    let resized =
        image::imageops::resize(&source, resized_width, resized_height, FilterType::Triangle);
    let pad_x = (640 - resized_width) / 2;
    let pad_y = (640 - resized_height) / 2;
    let mut preview = RgbImage::from_pixel(640, 640, Rgb([114, 114, 114]));
    image::imageops::replace(&mut preview, &resized, i64::from(pad_x), i64::from(pad_y));
    let tensor_data = rgb_to_nchw_f32_normalized(&preview);
    PreparedPoseImage {
        tensor_data,
        scale,
        pad_x: pad_x as f32,
        pad_y: pad_y as f32,
    }
}

fn rgb_to_nchw_f32_normalized(image: &RgbImage) -> Vec<f32> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![0.0; channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = f32::from(pixel[0]) / 255.0;
        data[channel_len + index] = f32::from(pixel[1]) / 255.0;
        data[channel_len * 2 + index] = f32::from(pixel[2]) / 255.0;
    }
    data
}

fn decode_pose_output(
    output: &PoseOutputTensor,
    image_width: u32,
    image_height: u32,
    transform: PoseImageTransform,
    score_threshold: f32,
) -> anyhow::Result<Vec<WorkerPoseDetection>> {
    let mut poses = Vec::new();
    for candidate_index in 0..POSE_OUTPUT_CANDIDATES {
        let confidence = pose_value(output, 4, candidate_index);
        if confidence < score_threshold {
            continue;
        }
        let center_x = (pose_value(output, 0, candidate_index) - transform.pad_x) / transform.scale;
        let center_y = (pose_value(output, 1, candidate_index) - transform.pad_y) / transform.scale;
        let width = pose_value(output, 2, candidate_index) / transform.scale;
        let height = pose_value(output, 3, candidate_index) / transform.scale;
        let keypoints = (0..17)
            .map(|keypoint_index| {
                let channel = 5 + keypoint_index * 3;
                PoseKeypoint {
                    x: (pose_value(output, channel, candidate_index) - transform.pad_x)
                        / transform.scale,
                    y: (pose_value(output, channel + 1, candidate_index) - transform.pad_y)
                        / transform.scale,
                    confidence: pose_value(output, channel + 2, candidate_index),
                }
            })
            .collect::<Vec<_>>();
        let pose = WorkerPoseDetection {
            local_person_index: poses.len(),
            person_box: normalized_box_from_pixels(
                center_x - width / 2.0,
                center_y - height / 2.0,
                width,
                height,
                image_width,
                image_height,
            ),
            confidence,
            keypoints,
        };
        if is_valid_worker_pose_candidate(&pose) {
            poses.push(pose);
        }
    }

    Ok(non_maximum_suppression_pose(poses, POSE_NMS_THRESHOLD))
}

fn is_valid_worker_pose_candidate(pose: &WorkerPoseDetection) -> bool {
    let box_area = pose.person_box.width * pose.person_box.height;
    let visible_keypoints = pose
        .keypoints
        .iter()
        .filter(|keypoint| keypoint.confidence >= 0.02)
        .count();
    let strong_keypoints = pose
        .keypoints
        .iter()
        .filter(|keypoint| keypoint.confidence >= 0.10)
        .count();
    let is_bottom_partial = pose.person_box.y + pose.person_box.height >= 0.98
        && pose.person_box.height <= POSE_EDGE_PARTIAL_MAX_HEIGHT;
    let is_top_partial =
        pose.person_box.y <= 0.02 && pose.person_box.height <= POSE_EDGE_PARTIAL_MAX_HEIGHT;
    let is_material_stack_false_person = is_material_stack_false_person_candidate(pose, box_area);
    pose.person_box.width >= POSE_MIN_BOX_WIDTH
        && pose.person_box.height >= POSE_MIN_BOX_HEIGHT
        && box_area >= POSE_MIN_BOX_AREA
        && visible_keypoints >= POSE_MIN_VISIBLE_KEYPOINTS
        && strong_keypoints >= POSE_MIN_STRONG_KEYPOINTS
        && !is_bottom_partial
        && !is_top_partial
        && !is_material_stack_false_person
}

fn is_material_stack_false_person_candidate(pose: &WorkerPoseDetection, box_area: f32) -> bool {
    let center = NormalizedPoint {
        x: pose.person_box.x + pose.person_box.width / 2.0,
        y: pose.person_box.y + pose.person_box.height / 2.0,
    };
    containment_score(MATERIAL_STACK_FALSE_PERSON_ZONE, center) > 0.0
        && pose.person_box.height <= MATERIAL_STACK_FALSE_PERSON_MAX_HEIGHT
        && box_area <= MATERIAL_STACK_FALSE_PERSON_MAX_AREA
}

fn pose_value(output: &PoseOutputTensor, channel: usize, candidate_index: usize) -> f32 {
    output.data[channel * POSE_OUTPUT_CANDIDATES + candidate_index]
}

fn assign_stable_pose_track_ids(pose_frames: &mut [WorkerPoseFrame]) {
    let mut tracker = WorkerPoseTracker::new();
    for frame in &mut *pose_frames {
        let assignments = tracker.assign_frame(frame.frame_index, &frame.poses);
        for (pose, person_id) in frame.poses.iter_mut().zip(assignments) {
            pose.local_person_index = person_id as usize - 1;
        }
    }
    merge_fragmented_pose_track_ids(pose_frames);
}

fn merge_fragmented_pose_track_ids(pose_frames: &mut [WorkerPoseFrame]) {
    let mut summaries = pose_track_summaries(pose_frames);
    summaries.sort_by_key(|summary| (summary.first_frame_index, summary.person_id));
    let mut remap = BTreeMap::<WorkerTrackId, WorkerTrackId>::new();
    for summary in summaries {
        let target_id =
            best_tracklet_merge_target(&summary, &remap, pose_frames).unwrap_or(summary.person_id);
        remap.insert(summary.person_id, target_id);
    }
    for frame in pose_frames {
        for pose in &mut frame.poses {
            let person_id = pose.local_person_index as WorkerTrackId + 1;
            if let Some(target_id) = remap.get(&person_id).copied() {
                pose.local_person_index = target_id as usize - 1;
            }
        }
    }
}

fn best_tracklet_merge_target(
    summary: &PoseTrackSummary,
    remap: &BTreeMap<WorkerTrackId, WorkerTrackId>,
    pose_frames: &[WorkerPoseFrame],
) -> Option<WorkerTrackId> {
    let mut candidates = pose_track_summaries(pose_frames)
        .into_iter()
        .filter(|candidate| candidate.person_id != summary.person_id)
        .filter(|candidate| candidate.last_frame_index < summary.first_frame_index)
        .filter(|candidate| {
            summary
                .first_frame_index
                .saturating_sub(candidate.last_frame_index)
                <= TRACKLET_MERGE_MAX_GAP_FRAMES
        })
        .filter_map(|candidate| {
            let target_id = remap
                .get(&candidate.person_id)
                .copied()
                .unwrap_or(candidate.person_id);
            let distance = normalized_box_center_distance(summary.first_box, candidate.last_box);
            let vertical_gap = (summary.first_box.y - candidate.last_box.y).abs();
            let horizontal_gap = (summary.first_box.x - candidate.last_box.x).abs();
            let area_ratio = normalized_box_area_ratio(summary.first_box, candidate.last_box);
            let compatible_position = distance <= TRACKLET_MERGE_MAX_ENDPOINT_DISTANCE
                && vertical_gap <= TRACKLET_MERGE_MAX_VERTICAL_GAP
                && horizontal_gap <= TRACKLET_MERGE_MAX_HORIZONTAL_GAP
                && area_ratio <= TRACKLET_MERGE_MAX_AREA_RATIO;
            if compatible_position {
                Some((target_id, distance, horizontal_gap, area_ratio))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| left.2.total_cmp(&right.2))
            .then_with(|| left.3.total_cmp(&right.3))
    });
    candidates.first().map(|(target_id, _, _, _)| *target_id)
}

fn pose_track_summaries(pose_frames: &[WorkerPoseFrame]) -> Vec<PoseTrackSummary> {
    let mut summaries = BTreeMap::<WorkerTrackId, PoseTrackSummary>::new();
    for frame in pose_frames {
        for pose in &frame.poses {
            let person_id = pose.local_person_index as WorkerTrackId + 1;
            summaries
                .entry(person_id)
                .and_modify(|summary| {
                    summary.last_frame_index = frame.frame_index;
                    summary.last_box = pose.person_box;
                    summary.frame_count += 1;
                })
                .or_insert(PoseTrackSummary {
                    person_id,
                    first_frame_index: frame.frame_index,
                    last_frame_index: frame.frame_index,
                    first_box: pose.person_box,
                    last_box: pose.person_box,
                    frame_count: 1,
                });
        }
    }
    summaries.into_values().collect()
}

#[derive(Clone, Copy, Debug)]
struct PoseTrackSummary {
    person_id: WorkerTrackId,
    first_frame_index: u64,
    last_frame_index: u64,
    first_box: NormalizedBoundingBox,
    last_box: NormalizedBoundingBox,
    frame_count: usize,
}

fn action_observations_from_pose_frames(
    pose_frames: &[WorkerPoseFrame],
    target_roi: VisualTargetObservation,
    keypoint_score_threshold: f32,
) -> anyhow::Result<Vec<WorkerActionObservation>> {
    validate_unit_score("keypoint_score_threshold", keypoint_score_threshold)?;
    let mut previous_wrists = BTreeMap::<WorkerTrackId, NormalizedPoint>::new();
    let mut observations = Vec::new();
    let mut previous_frame_image: Option<RgbImage> = None;
    for frame in pose_frames {
        let current_frame_image = image::open(&frame.frame_path)?.to_rgb8();
        for pose in &frame.poses {
            let person_id = pose.local_person_index as WorkerTrackId + 1;
            let Some(wrist) = best_wrist_point(
                pose,
                frame.frame_width,
                frame.frame_height,
                keypoint_score_threshold,
            ) else {
                continue;
            };
            let wrist_movement = previous_wrists
                .get(&person_id)
                .map_or(0.0, |previous| normalized_distance(*previous, wrist));
            previous_wrists.insert(person_id, wrist);

            let contact_score = containment_score(target_roi.target_box, wrist);
            let strike_score = (wrist_movement * 8.0).clamp(0.0, 1.0);
            let contacted_target = (contact_score > 0.0).then_some(target_roi);
            let target_response_score = previous_frame_image.as_ref().map_or(0.0, |previous| {
                target_response_score_near_contact(
                    previous,
                    &current_frame_image,
                    frame,
                    target_roi.target_box,
                    wrist,
                )
            });
            observations.push(WorkerActionObservation {
                person_id,
                frame_index: frame.frame_index,
                timestamp_ms: frame.timestamp_ms,
                person_box: pose.person_box,
                strike_score,
                contact_score,
                contact_point: Some(wrist),
                contacted_target,
                target_response_score,
            });
        }
        previous_frame_image = Some(current_frame_image);
    }
    Ok(observations)
}

fn target_response_score_near_contact(
    previous_frame_image: &RgbImage,
    current_frame_image: &RgbImage,
    frame: &WorkerPoseFrame,
    target_box: NormalizedBoundingBox,
    contact_point: NormalizedPoint,
) -> f32 {
    if previous_frame_image.dimensions() != current_frame_image.dimensions() {
        return 0.0;
    }
    if containment_score(target_box, contact_point) <= 0.0 {
        return 0.0;
    }

    let image_width = current_frame_image.width();
    let image_height = current_frame_image.height();
    let Some(target_rect) = normalized_box_to_pixel_rect(target_box, image_width, image_height)
    else {
        return 0.0;
    };
    let contact_window = NormalizedBoundingBox {
        x: contact_point.x - TARGET_RESPONSE_CONTACT_WINDOW_X,
        y: contact_point.y - TARGET_RESPONSE_CONTACT_WINDOW_Y,
        width: TARGET_RESPONSE_CONTACT_WINDOW_X * 2.0,
        height: TARGET_RESPONSE_CONTACT_WINDOW_Y * 2.0,
    };
    let Some(contact_rect) =
        normalized_box_to_pixel_rect(contact_window, image_width, image_height)
    else {
        return 0.0;
    };
    let response_rect = intersect_pixel_rect(target_rect, contact_rect);
    if response_rect.is_empty() {
        return 0.0;
    }

    let masked_person_boxes = person_mask_rects_for_response(frame);
    let mut local_diffs = Vec::new();
    collect_gray_diffs(
        previous_frame_image,
        current_frame_image,
        response_rect,
        TARGET_RESPONSE_SAMPLE_STRIDE,
        &masked_person_boxes,
        &mut local_diffs,
    );
    if local_diffs.len() < TARGET_RESPONSE_MIN_SAMPLES {
        return 0.0;
    }

    let mut background_diffs = Vec::new();
    collect_gray_diffs_excluding_rect(
        previous_frame_image,
        current_frame_image,
        full_image_rect(image_width, image_height),
        target_rect,
        TARGET_RESPONSE_SAMPLE_STRIDE * TARGET_RESPONSE_BACKGROUND_STRIDE_MULTIPLIER,
        &masked_person_boxes,
        &mut background_diffs,
    );

    let local_top_mean =
        top_tail_mean(&mut local_diffs, TARGET_RESPONSE_TOP_PERCENTILE).unwrap_or(0.0);
    let background_mean = mean(&background_diffs).unwrap_or(0.0);
    let changed_pixel_threshold = TARGET_RESPONSE_CHANGED_PIXEL_FLOOR.max(background_mean * 2.5);
    let changed_ratio = changed_pixel_ratio(&local_diffs, changed_pixel_threshold);
    let appearance_delta = (local_top_mean
        - background_mean * TARGET_RESPONSE_BACKGROUND_WEIGHT
        - TARGET_RESPONSE_DIFF_FLOOR)
        .max(0.0);
    let changed_ratio_delta = (changed_ratio - TARGET_RESPONSE_CHANGED_RATIO_BASELINE).max(0.0);

    (appearance_delta * 3.0 + changed_ratio_delta * 2.5).clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PixelRect {
    x_min: u32,
    y_min: u32,
    x_max: u32,
    y_max: u32,
}

impl PixelRect {
    fn is_empty(self) -> bool {
        self.x_min >= self.x_max || self.y_min >= self.y_max
    }

    fn contains(self, x: u32, y: u32) -> bool {
        x >= self.x_min && x < self.x_max && y >= self.y_min && y < self.y_max
    }
}

fn person_mask_rects_for_response(frame: &WorkerPoseFrame) -> Vec<NormalizedBoundingBox> {
    frame
        .poses
        .iter()
        .map(|pose| {
            expand_normalized_box(pose.person_box, TARGET_RESPONSE_OTHER_PERSON_MASK_EXPAND)
        })
        .collect()
}

fn expand_normalized_box(bbox: NormalizedBoundingBox, margin: f32) -> NormalizedBoundingBox {
    let x = (bbox.x - margin).clamp(0.0, 1.0);
    let y = (bbox.y - margin).clamp(0.0, 1.0);
    let right = (bbox.x + bbox.width + margin).clamp(0.0, 1.0);
    let bottom = (bbox.y + bbox.height + margin).clamp(0.0, 1.0);
    NormalizedBoundingBox {
        x,
        y,
        width: (right - x).max(0.0),
        height: (bottom - y).max(0.0),
    }
}

fn normalized_box_to_pixel_rect(
    bbox: NormalizedBoundingBox,
    image_width: u32,
    image_height: u32,
) -> Option<PixelRect> {
    let x_min = (bbox.x.clamp(0.0, 1.0) * image_width as f32).floor() as u32;
    let y_min = (bbox.y.clamp(0.0, 1.0) * image_height as f32).floor() as u32;
    let x_max = ((bbox.x + bbox.width).clamp(0.0, 1.0) * image_width as f32).ceil() as u32;
    let y_max = ((bbox.y + bbox.height).clamp(0.0, 1.0) * image_height as f32).ceil() as u32;
    let rect = PixelRect {
        x_min: x_min.min(image_width),
        y_min: y_min.min(image_height),
        x_max: x_max.min(image_width),
        y_max: y_max.min(image_height),
    };
    (!rect.is_empty()).then_some(rect)
}

fn full_image_rect(image_width: u32, image_height: u32) -> PixelRect {
    PixelRect {
        x_min: 0,
        y_min: 0,
        x_max: image_width,
        y_max: image_height,
    }
}

fn intersect_pixel_rect(left: PixelRect, right: PixelRect) -> PixelRect {
    PixelRect {
        x_min: left.x_min.max(right.x_min),
        y_min: left.y_min.max(right.y_min),
        x_max: left.x_max.min(right.x_max),
        y_max: left.y_max.min(right.y_max),
    }
}

fn collect_gray_diffs(
    previous_frame_image: &RgbImage,
    current_frame_image: &RgbImage,
    rect: PixelRect,
    stride: u32,
    masked_boxes: &[NormalizedBoundingBox],
    output: &mut Vec<f32>,
) {
    collect_gray_diffs_with_filter(
        previous_frame_image,
        current_frame_image,
        rect,
        stride,
        masked_boxes,
        |_, _| true,
        output,
    );
}

fn collect_gray_diffs_excluding_rect(
    previous_frame_image: &RgbImage,
    current_frame_image: &RgbImage,
    rect: PixelRect,
    excluded_rect: PixelRect,
    stride: u32,
    masked_boxes: &[NormalizedBoundingBox],
    output: &mut Vec<f32>,
) {
    collect_gray_diffs_with_filter(
        previous_frame_image,
        current_frame_image,
        rect,
        stride,
        masked_boxes,
        |x, y| !excluded_rect.contains(x, y),
        output,
    );
}

fn collect_gray_diffs_with_filter(
    previous_frame_image: &RgbImage,
    current_frame_image: &RgbImage,
    rect: PixelRect,
    stride: u32,
    masked_boxes: &[NormalizedBoundingBox],
    include_pixel: impl Fn(u32, u32) -> bool,
    output: &mut Vec<f32>,
) {
    if rect.is_empty() {
        return;
    }
    let image_width = current_frame_image.width();
    let image_height = current_frame_image.height();
    let masked_rects = masked_boxes
        .iter()
        .filter_map(|bbox| normalized_box_to_pixel_rect(*bbox, image_width, image_height))
        .collect::<Vec<_>>();
    let stride = stride.max(1) as usize;
    for y in (rect.y_min..rect.y_max).step_by(stride) {
        for x in (rect.x_min..rect.x_max).step_by(stride) {
            if !include_pixel(x, y) || masked_rects.iter().any(|masked| masked.contains(x, y)) {
                continue;
            }
            let previous = previous_frame_image.get_pixel(x, y);
            let current = current_frame_image.get_pixel(x, y);
            output.push(gray_diff(previous, current));
        }
    }
}

fn gray_diff(previous: &Rgb<u8>, current: &Rgb<u8>) -> f32 {
    let previous_gray = f32::from(previous[0]) * 0.299
        + f32::from(previous[1]) * 0.587
        + f32::from(previous[2]) * 0.114;
    let current_gray = f32::from(current[0]) * 0.299
        + f32::from(current[1]) * 0.587
        + f32::from(current[2]) * 0.114;
    (current_gray - previous_gray).abs() / 255.0
}

fn mean(values: &[f32]) -> Option<f32> {
    (!values.is_empty()).then(|| values.iter().sum::<f32>() / values.len() as f32)
}

fn top_tail_mean(values: &mut [f32], start_percentile: f32) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f32::total_cmp);
    let start_index = (values.len() as f32 * start_percentile.clamp(0.0, 1.0)).floor() as usize;
    let start_index = start_index.min(values.len() - 1);
    mean(&values[start_index..])
}

fn changed_pixel_ratio(values: &[f32], threshold: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let changed = values.iter().filter(|value| **value >= threshold).count();
    changed as f32 / values.len() as f32
}

#[derive(Default)]
struct WorkerPoseTracker {
    next_person_id: WorkerTrackId,
    tracks: BTreeMap<WorkerTrackId, WorkerPoseTrackState>,
}

#[derive(Clone, Copy, Debug)]
struct WorkerPoseTrackState {
    person_box: NormalizedBoundingBox,
    last_seen_frame_index: u64,
}

impl WorkerPoseTracker {
    fn new() -> Self {
        Self {
            next_person_id: 1,
            tracks: BTreeMap::new(),
        }
    }

    fn assign_frame(
        &mut self,
        frame_index: u64,
        poses: &[WorkerPoseDetection],
    ) -> Vec<WorkerTrackId> {
        let mut assigned_previous_ids = BTreeSet::new();
        let mut assignments = Vec::with_capacity(poses.len());
        self.prune_stale_tracks(frame_index);
        for pose in poses {
            let person_id = self
                .best_existing_track(frame_index, pose.person_box, &assigned_previous_ids)
                .unwrap_or_else(|| self.next_track_id());
            assigned_previous_ids.insert(person_id);
            self.tracks.insert(
                person_id,
                WorkerPoseTrackState {
                    person_box: pose.person_box,
                    last_seen_frame_index: frame_index,
                },
            );
            assignments.push(person_id);
        }
        assignments
    }

    fn best_existing_track(
        &self,
        frame_index: u64,
        person_box: NormalizedBoundingBox,
        assigned_previous_ids: &BTreeSet<WorkerTrackId>,
    ) -> Option<WorkerTrackId> {
        self.tracks
            .iter()
            .filter(|(person_id, _)| !assigned_previous_ids.contains(person_id))
            .filter_map(|(person_id, previous)| {
                let missed_frames = frame_index.saturating_sub(previous.last_seen_frame_index);
                let iou = intersection_over_union(person_box, previous.person_box);
                let distance = normalized_box_center_distance(person_box, previous.person_box);
                let distance_gate = (TRACK_MAX_NORMALIZED_DISTANCE + missed_frames as f32 * 0.006)
                    .min(TRACK_MAX_NORMALIZED_DISTANCE_WITH_MISSES);
                if iou > 0.03 || distance <= distance_gate {
                    Some((*person_id, iou, distance, missed_frames))
                } else {
                    None
                }
            })
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.2.total_cmp(&left.2))
                    .then_with(|| right.3.cmp(&left.3))
            })
            .map(|(person_id, _, _, _)| person_id)
    }

    fn next_track_id(&mut self) -> WorkerTrackId {
        let person_id = self.next_person_id;
        self.next_person_id += 1;
        person_id
    }

    fn prune_stale_tracks(&mut self, frame_index: u64) {
        self.tracks.retain(|_, track| {
            frame_index.saturating_sub(track.last_seen_frame_index) <= TRACK_MAX_MISSED_FRAMES
        });
    }
}

fn normalized_box_center_distance(
    left: NormalizedBoundingBox,
    right: NormalizedBoundingBox,
) -> f32 {
    let left_center = NormalizedPoint {
        x: left.x + left.width / 2.0,
        y: left.y + left.height / 2.0,
    };
    let right_center = NormalizedPoint {
        x: right.x + right.width / 2.0,
        y: right.y + right.height / 2.0,
    };
    normalized_distance(left_center, right_center)
}

fn normalized_box_area_ratio(left: NormalizedBoundingBox, right: NormalizedBoundingBox) -> f32 {
    let left_area = left.width * left.height;
    let right_area = right.width * right.height;
    let min_area = left_area.min(right_area);
    let max_area = left_area.max(right_area);
    if min_area <= f32::EPSILON {
        return f32::INFINITY;
    }
    max_area / min_area
}

fn best_wrist_point(
    pose: &WorkerPoseDetection,
    frame_width: u32,
    frame_height: u32,
    keypoint_score_threshold: f32,
) -> Option<NormalizedPoint> {
    [LEFT_WRIST_INDEX, RIGHT_WRIST_INDEX]
        .into_iter()
        .filter_map(|index| pose.keypoints.get(index))
        .filter(|keypoint| {
            keypoint.confidence.is_finite() && keypoint.confidence >= keypoint_score_threshold
        })
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
        .map(|keypoint| NormalizedPoint {
            x: (keypoint.x / frame_width as f32).clamp(0.0, 1.0),
            y: (keypoint.y / frame_height as f32).clamp(0.0, 1.0),
        })
}

fn containment_score(target_box: NormalizedBoundingBox, point: NormalizedPoint) -> f32 {
    let within_x = point.x >= target_box.x && point.x <= target_box.x + target_box.width;
    let within_y = point.y >= target_box.y && point.y <= target_box.y + target_box.height;
    if within_x && within_y { 1.0 } else { 0.0 }
}

fn normalized_distance(left: NormalizedPoint, right: NormalizedPoint) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    (dx * dx + dy * dy).sqrt()
}

fn normalized_box_from_pixels(
    x_min: f32,
    y_min: f32,
    width: f32,
    height: f32,
    image_width: u32,
    image_height: u32,
) -> NormalizedBoundingBox {
    let x_min = x_min.clamp(0.0, image_width as f32);
    let y_min = y_min.clamp(0.0, image_height as f32);
    let x_max = (x_min + width).clamp(0.0, image_width as f32);
    let y_max = (y_min + height).clamp(0.0, image_height as f32);
    NormalizedBoundingBox {
        x: x_min / image_width as f32,
        y: y_min / image_height as f32,
        width: ((x_max - x_min) / image_width as f32).clamp(0.0, 1.0),
        height: ((y_max - y_min) / image_height as f32).clamp(0.0, 1.0),
    }
}

fn write_worker_hit_annotation_frames(
    pose_frames: &[WorkerPoseFrame],
    timeline: &WorkerHitTimeline,
    target_roi: NormalizedBoundingBox,
) -> anyhow::Result<()> {
    let records_by_frame = timeline
        .frame_records
        .iter()
        .map(|record| ((record.frame_index, record.person_id), record))
        .collect::<BTreeMap<_, _>>();
    let mut held_records = BTreeMap::<WorkerTrackId, &WorkerActionFrameRecord>::new();
    for frame in pose_frames {
        let image = image::open(&frame.frame_path)?;
        let mut canvas = image.to_rgb8();
        draw_normalized_rect(&mut canvas, target_roi, Rgb([255, 215, 0]));
        draw_text_label(&mut canvas, 8, 8, &["悬挂金属板"], Rgb([255, 215, 0]));
        for pose in &frame.poses {
            let person_id = pose.local_person_index as WorkerTrackId + 1;
            if let Some(record) = records_by_frame.get(&(frame.frame_index, person_id)) {
                held_records.insert(person_id, record);
            }
            let held_record = held_records.get(&person_id).copied().filter(|record| {
                frame.frame_index.saturating_sub(record.frame_index) <= ANNOTATION_PANEL_HOLD_FRAMES
            });
            draw_pose_annotation(&mut canvas, pose, person_id, held_record);
        }
        draw_persistent_worker_summary(&mut canvas, frame.frame_index, &held_records);
        canvas.save(&frame.annotated_frame_path)?;
    }
    Ok(())
}

fn draw_persistent_worker_summary(
    canvas: &mut RgbImage,
    frame_index: u64,
    held_records: &BTreeMap<WorkerTrackId, &WorkerActionFrameRecord>,
) {
    let mut lines = Vec::new();
    for (person_id, record) in held_records {
        if frame_index.saturating_sub(record.frame_index) > ANNOTATION_PANEL_HOLD_FRAMES {
            continue;
        }
        lines.push(format!(
            "人员{} 有效:{} 无效:{} 状态:{}",
            person_id,
            record.valid_hit_count,
            record.invalid_candidate_count,
            state_label(record.state)
        ));
    }
    if lines.is_empty() {
        return;
    }
    let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    let x = 8;
    let y = canvas.height() as i32 - (line_refs.len() as i32 * 34 + 16);
    draw_text_label(canvas, x, y.max(0), &line_refs, Rgb([255, 255, 255]));
}

fn draw_pose_annotation(
    canvas: &mut RgbImage,
    pose: &WorkerPoseDetection,
    person_id: WorkerTrackId,
    record: Option<&WorkerActionFrameRecord>,
) {
    let state = record.map_or(WorkerActionState::Idle, |record| record.state);
    let color = state_color(state);
    draw_normalized_rect(canvas, pose.person_box, color);
    for keypoint in &pose.keypoints {
        if keypoint.confidence > 0.10 {
            draw_filled_circle_mut(
                canvas,
                (keypoint.x.round() as i32, keypoint.y.round() as i32),
                3,
                Rgb([0, 180, 255]),
            );
        }
    }
    let (valid_hits, invalid_candidates) = record
        .map(|record| (record.valid_hit_count, record.invalid_candidate_count))
        .unwrap_or((0, 0));
    let label_top = format!("人员{person_id}  状态:{}", state_label(state));
    let label_bottom = format!("有效:{valid_hits}  无效:{invalid_candidates}");
    let x = (pose.person_box.x * canvas.width() as f32).round() as i32;
    let y = (pose.person_box.y * canvas.height() as f32).round() as i32 - 52;
    draw_text_label(
        canvas,
        x.max(0),
        y.max(0),
        &[&label_top, &label_bottom],
        color,
    );
}

fn state_color(state: WorkerActionState) -> Rgb<u8> {
    match state {
        WorkerActionState::Idle => Rgb([180, 180, 180]),
        WorkerActionState::Striking => Rgb([0, 180, 255]),
        WorkerActionState::ValidHit => Rgb([0, 220, 80]),
        WorkerActionState::InvalidHitCandidate => Rgb([255, 80, 80]),
    }
}

fn state_label(state: WorkerActionState) -> &'static str {
    match state {
        WorkerActionState::Idle => "未敲击",
        WorkerActionState::Striking => "挥动中",
        WorkerActionState::ValidHit => "有效敲击",
        WorkerActionState::InvalidHitCandidate => "无效敲击",
    }
}

fn draw_normalized_rect(image: &mut RgbImage, rect: NormalizedBoundingBox, color: Rgb<u8>) {
    let x = (rect.x * image.width() as f32).round() as i32;
    let y = (rect.y * image.height() as f32).round() as i32;
    let width = (rect.width * image.width() as f32).round().max(1.0) as u32;
    let height = (rect.height * image.height() as f32).round().max(1.0) as u32;
    draw_hollow_rect_mut(image, Rect::at(x, y).of_size(width, height), color);
}

fn draw_text_label(image: &mut RgbImage, x: i32, y: i32, lines: &[&str], color: Rgb<u8>) {
    if let Some(font) = chinese_annotation_font() {
        draw_chinese_text_label(image, x, y, lines, color, font);
    } else {
        draw_pixel_text_label(image, x, y, lines, color);
    }
}

fn draw_chinese_text_label(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    lines: &[&str],
    color: Rgb<u8>,
    font: &FontArc,
) {
    const PADDING: i32 = 6;
    const LINE_GAP: i32 = 4;

    let scale = PxScale::from(24.0);
    let measured_lines = lines
        .iter()
        .map(|line| {
            let (width, height) = text_size(scale, font, line);
            (*line, width as i32, height as i32)
        })
        .collect::<Vec<_>>();
    let label_width = measured_lines
        .iter()
        .map(|(_, width, _)| *width)
        .max()
        .unwrap_or(0)
        + PADDING * 2;
    let label_height = measured_lines
        .iter()
        .map(|(_, _, height)| *height)
        .sum::<i32>()
        + LINE_GAP * (measured_lines.len().saturating_sub(1) as i32)
        + PADDING * 2;
    if label_width <= PADDING * 2 || label_height <= PADDING * 2 {
        return;
    }

    let x = x.min(image.width() as i32 - label_width).max(0);
    let y = y.min(image.height() as i32 - label_height).max(0);
    draw_filled_rect_mut(
        image,
        Rect::at(x, y).of_size(label_width as u32, label_height as u32),
        Rgb([0, 0, 0]),
    );

    let mut cursor_y = y + PADDING;
    for (line, _, height) in measured_lines {
        draw_text_mut(image, color, x + PADDING, cursor_y, scale, font, line);
        cursor_y += height + LINE_GAP;
    }
}

fn chinese_annotation_font() -> Option<&'static FontArc> {
    static FONT: OnceLock<Option<FontArc>> = OnceLock::new();
    FONT.get_or_init(load_chinese_annotation_font).as_ref()
}

fn load_chinese_annotation_font() -> Option<FontArc> {
    [
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
    ]
    .into_iter()
    .find_map(|path| {
        fs::read(path)
            .ok()
            .and_then(|data| FontArc::try_from_vec(data).ok())
    })
}

fn draw_pixel_text_label(image: &mut RgbImage, x: i32, y: i32, lines: &[&str], color: Rgb<u8>) {
    const GLYPH_WIDTH: i32 = 3;
    const GLYPH_HEIGHT: i32 = 5;
    const SCALE: i32 = 3;
    const GLYPH_GAP: i32 = 2;
    const PADDING: i32 = 3;
    const LINE_GAP: i32 = 4;

    let fallback_lines = lines
        .iter()
        .map(|line| ascii_annotation_fallback(line))
        .collect::<Vec<_>>();
    let visible_chars = fallback_lines
        .iter()
        .map(|line| line.chars().count() as i32)
        .max()
        .unwrap_or(0);
    if visible_chars == 0 {
        return;
    }
    let label_width = visible_chars * (GLYPH_WIDTH * SCALE + GLYPH_GAP) - GLYPH_GAP + PADDING * 2;
    let line_height = GLYPH_HEIGHT * SCALE;
    let label_height = fallback_lines.len() as i32 * line_height
        + fallback_lines.len().saturating_sub(1) as i32 * LINE_GAP
        + PADDING * 2;
    let x = x.min(image.width() as i32 - label_width).max(0);
    let y = y.min(image.height() as i32 - label_height).max(0);
    draw_filled_rect_mut(
        image,
        Rect::at(x, y).of_size(label_width as u32, label_height as u32),
        Rgb([0, 0, 0]),
    );

    let mut top = y + PADDING;
    for line in fallback_lines {
        let mut cursor_x = x + PADDING;
        for ch in line.chars() {
            if let Some(rows) = glyph_rows(ch) {
                draw_glyph(image, cursor_x, top, rows, SCALE, color);
            }
            cursor_x += GLYPH_WIDTH * SCALE + GLYPH_GAP;
        }
        top += line_height + LINE_GAP;
    }
}

fn ascii_annotation_fallback(line: &str) -> String {
    line.replace("悬挂金属板", "TARGET")
        .replace("人员", "P")
        .replace("状态:未敲击", "IDLE")
        .replace("状态:挥动中", "STRIKE")
        .replace("状态:有效敲击", "HIT")
        .replace("状态:无效敲击", "BAD")
        .replace("有效:", "H:")
        .replace("无效:", "X:")
}

fn draw_glyph(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    rows: [&'static str; 5],
    scale: i32,
    color: Rgb<u8>,
) {
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, pixel) in row.bytes().enumerate() {
            if pixel == b'1' {
                draw_filled_rect_mut(
                    image,
                    Rect::at(
                        x + column_index as i32 * scale,
                        y + row_index as i32 * scale,
                    )
                    .of_size(scale as u32, scale as u32),
                    color,
                );
            }
        }
    }
}

fn glyph_rows(ch: char) -> Option<[&'static str; 5]> {
    match ch {
        '0' => Some(["111", "101", "101", "101", "111"]),
        '1' => Some(["010", "110", "010", "010", "111"]),
        '2' => Some(["111", "001", "111", "100", "111"]),
        '3' => Some(["111", "001", "111", "001", "111"]),
        '4' => Some(["101", "101", "111", "001", "001"]),
        '5' => Some(["111", "100", "111", "001", "111"]),
        '6' => Some(["111", "100", "111", "101", "111"]),
        '7' => Some(["111", "001", "010", "010", "010"]),
        '8' => Some(["111", "101", "111", "101", "111"]),
        '9' => Some(["111", "101", "111", "001", "111"]),
        'A' => Some(["010", "101", "111", "101", "101"]),
        'B' => Some(["110", "101", "110", "101", "110"]),
        'D' => Some(["110", "101", "101", "101", "110"]),
        'E' => Some(["111", "100", "110", "100", "111"]),
        'G' => Some(["111", "100", "101", "101", "111"]),
        'H' => Some(["101", "101", "111", "101", "101"]),
        'I' => Some(["111", "010", "010", "010", "111"]),
        'K' => Some(["101", "101", "110", "101", "101"]),
        'L' => Some(["100", "100", "100", "100", "111"]),
        'P' => Some(["110", "101", "110", "100", "100"]),
        'R' => Some(["110", "101", "110", "101", "101"]),
        'S' => Some(["111", "100", "111", "001", "111"]),
        'T' => Some(["111", "010", "010", "010", "010"]),
        'X' => Some(["101", "101", "010", "101", "101"]),
        ':' => Some(["000", "010", "000", "010", "000"]),
        ' ' => Some(["000", "000", "000", "000", "000"]),
        _ => None,
    }
}

fn non_maximum_suppression_pose(
    mut poses: Vec<WorkerPoseDetection>,
    nms_threshold: f32,
) -> Vec<WorkerPoseDetection> {
    poses.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    let mut kept: Vec<WorkerPoseDetection> = Vec::new();
    for candidate in poses {
        let overlaps_existing = kept.iter().any(|selected| {
            let iou = intersection_over_union(candidate.person_box, selected.person_box);
            let center_distance =
                normalized_box_center_distance(candidate.person_box, selected.person_box);
            let containment = smaller_box_containment(candidate.person_box, selected.person_box);
            iou > nms_threshold || center_distance < 0.08 || containment > 0.65
        });
        if !overlaps_existing {
            kept.push(candidate);
        }
    }
    for (index, pose) in kept.iter_mut().enumerate() {
        pose.local_person_index = index;
    }
    kept
}

fn smaller_box_containment(left: NormalizedBoundingBox, right: NormalizedBoundingBox) -> f32 {
    let intersection_x_min = left.x.max(right.x);
    let intersection_y_min = left.y.max(right.y);
    let intersection_x_max = (left.x + left.width).min(right.x + right.width);
    let intersection_y_max = (left.y + left.height).min(right.y + right.height);
    let intersection_width = (intersection_x_max - intersection_x_min).max(0.0);
    let intersection_height = (intersection_y_max - intersection_y_min).max(0.0);
    let intersection = intersection_width * intersection_height;
    let smaller_area = (left.width * left.height).min(right.width * right.height);
    if smaller_area <= 0.0 {
        return 0.0;
    }
    intersection / smaller_area
}

fn intersection_over_union(left: NormalizedBoundingBox, right: NormalizedBoundingBox) -> f32 {
    let intersection_x_min = left.x.max(right.x);
    let intersection_y_min = left.y.max(right.y);
    let intersection_x_max = (left.x + left.width).min(right.x + right.width);
    let intersection_y_max = (left.y + left.height).min(right.y + right.height);
    let intersection_width = (intersection_x_max - intersection_x_min).max(0.0);
    let intersection_height = (intersection_y_max - intersection_y_min).max(0.0);
    let intersection = intersection_width * intersection_height;
    let union = left.width * left.height + right.width * right.height - intersection;
    if union <= 0.0 {
        return 0.0;
    }
    intersection / union
}

fn validate_video_analysis_options(options: &WorkerHitVideoAnalysisOptions) -> anyhow::Result<()> {
    options.hit_count_config.validate()?;
    validate_unit_score("pose_score_threshold", options.pose_score_threshold)?;
    validate_unit_score("keypoint_score_threshold", options.keypoint_score_threshold)?;
    validate_normalized_box("target_roi.target_box", options.target_roi.target_box)?;
    if options.sample_fps == 0 {
        bail!("invalid visual action input: sample_fps must be greater than 0");
    }
    if options.output_fps == 0 {
        bail!("invalid visual action input: output_fps must be greater than 0");
    }
    if !options.pose_model_path.is_file() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "pose model file not found");
        let path = options.pose_model_path.clone();
        let error = path_error(path, source);
        return Err(error);
    }
    Ok(())
}

fn recreate_dir(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| path_error(path.to_path_buf(), source))?;
    }
    fs::create_dir_all(path).map_err(|source| path_error(path.to_path_buf(), source))
}

fn extract_video_frames(
    video_path: &Path,
    extracted_frame_dir: &Path,
    options: &WorkerHitVideoAnalysisOptions,
) -> anyhow::Result<()> {
    run_ffmpeg(
        &options.ffmpeg_path,
        &[
            "-y".to_owned(),
            "-i".to_owned(),
            video_path.display().to_string(),
            "-vf".to_owned(),
            format!("fps={}", options.sample_fps),
            extracted_frame_dir
                .join("frame_%05d.png")
                .display()
                .to_string(),
        ],
        extracted_frame_dir,
    )
}

fn encode_annotated_video(
    annotated_frame_dir: &Path,
    annotated_video: &Path,
    options: &WorkerHitVideoAnalysisOptions,
) -> anyhow::Result<()> {
    run_ffmpeg(
        &options.ffmpeg_path,
        &[
            "-y".to_owned(),
            "-framerate".to_owned(),
            options.sample_fps.to_string(),
            "-i".to_owned(),
            annotated_frame_dir
                .join("frame_%05d.png")
                .display()
                .to_string(),
            "-vf".to_owned(),
            format!("fps={}", options.output_fps),
            "-c:v".to_owned(),
            "libx264".to_owned(),
            "-pix_fmt".to_owned(),
            "yuv420p".to_owned(),
            annotated_video.display().to_string(),
        ],
        annotated_video,
    )
}

fn run_ffmpeg(ffmpeg_path: &Path, args: &[String], context_path: &Path) -> anyhow::Result<()> {
    let output = Command::new(ffmpeg_path)
        .args(args)
        .output()
        .map_err(|source| path_error(ffmpeg_path.to_path_buf(), source))?;
    if output.status.success() {
        return Ok(());
    }
    Err(path_error(
        context_path.to_path_buf(),
        std::io::Error::other(format!(
            "ffmpeg failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )),
    ))
}

fn collected_frame_paths(
    extracted_frame_dir: &Path,
    max_frames: Option<usize>,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut frame_paths = fs::read_dir(extracted_frame_dir)
        .map_err(|source| path_error(extracted_frame_dir.to_path_buf(), source))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| path_error(extracted_frame_dir.to_path_buf(), source))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    frame_paths.sort();
    if let Some(limit) = max_frames {
        frame_paths.truncate(limit);
    }
    if frame_paths.is_empty() {
        let path = extracted_frame_dir.to_path_buf();
        let source = std::io::Error::other("ffmpeg did not extract any video frames");
        let error = path_error(path, source);

        return Err(error);
    }
    Ok(frame_paths)
}

fn frame_timestamp_ms(frame_index: usize, sample_fps: u32) -> u64 {
    (frame_index as u64 * 1_000) / u64::from(sample_fps)
}

fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(|source| {
        path_error(
            path.to_path_buf(),
            std::io::Error::other(source.to_string()),
        )
    })?;
    fs::write(path, json).map_err(|source| path_error(path.to_path_buf(), source))
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow!("filesystem error at `{}`: {source}", path.display())
}

#[cfg(test)]
mod tests {
    use image::{Rgb, RgbImage};

    use crate::components::worker_hit_counting::assist::target_response_score_near_contact;
    use crate::components::worker_hit_counting::model::{
        NormalizedBoundingBox, NormalizedPoint, PoseKeypoint, WorkerPoseDetection, WorkerPoseFrame,
    };

    #[test]
    fn target_response_score_should_ignore_motion_outside_contact_neighborhood() {
        let previous = RgbImage::from_pixel(200, 200, Rgb([40, 40, 40]));
        let mut current = previous.clone();
        for y in 20..60 {
            for x in 20..60 {
                current.put_pixel(x, y, Rgb([230, 230, 230]));
            }
        }
        let frame = response_test_frame();

        let score = target_response_score_near_contact(
            &previous,
            &current,
            &frame,
            response_target_box(),
            NormalizedPoint { x: 0.52, y: 0.52 },
        );

        // 关键断言：接触点远处有人或背景变化时，不能把它当成悬挂板被敲后的响应。
        assert_eq!(score, 0.0);
    }

    #[test]
    fn target_response_score_should_detect_visual_change_near_contact() {
        let previous = RgbImage::from_pixel(200, 200, Rgb([40, 40, 40]));
        let mut current = previous.clone();
        for y in 96..116 {
            for x in 96..116 {
                current.put_pixel(x, y, Rgb([230, 230, 230]));
            }
        }
        let frame = response_test_frame();

        let score = target_response_score_near_contact(
            &previous,
            &current,
            &frame,
            response_target_box(),
            NormalizedPoint { x: 0.52, y: 0.52 },
        );

        // 关键断言：接触点邻域出现明显视觉变化，才允许目标响应分数超过默认阈值。
        assert!(score >= 0.35, "target response score was {score}");
    }

    fn response_test_frame() -> WorkerPoseFrame {
        WorkerPoseFrame {
            frame_index: 1,
            timestamp_ms: 100,
            frame_width: 200,
            frame_height: 200,
            frame_path: "unused.png".into(),
            annotated_frame_path: "unused_annotated.png".into(),
            poses: vec![WorkerPoseDetection {
                local_person_index: 0,
                person_box: NormalizedBoundingBox {
                    x: 0.05,
                    y: 0.05,
                    width: 0.16,
                    height: 0.16,
                },
                confidence: 0.90,
                keypoints: vec![PoseKeypoint {
                    x: 100.0,
                    y: 100.0,
                    confidence: 0.90,
                }],
            }],
        }
    }

    fn response_target_box() -> NormalizedBoundingBox {
        NormalizedBoundingBox {
            x: 0.35,
            y: 0.35,
            width: 0.30,
            height: 0.30,
        }
    }
}
