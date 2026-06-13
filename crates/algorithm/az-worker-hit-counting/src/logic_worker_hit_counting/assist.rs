//! 工人有效敲击计数辅助函数。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::error::{WorkerHitCountingError, WorkerHitCountingResult};
use crate::logic_worker_hit_counting::model::{
    DEFAULT_POSE_MODEL_FILE_NAME, DEFAULT_WORKER_HIT_KEYPOINT_SCORE_THRESHOLD,
    DEFAULT_WORKER_HIT_POSE_SCORE_THRESHOLD, DEFAULT_WORKER_HIT_SAMPLE_FPS,
    DEFAULT_WORKER_HIT_TARGET_ROI, HitCandidateClassification, InvalidHitReason,
    InvalidWorkerHitCandidate, NormalizedBoundingBox, NormalizedPoint, PoseKeypoint,
    VisualTargetKind, VisualTargetObservation, WorkerActionAccumulator, WorkerActionFrameRecord,
    WorkerActionObservation, WorkerActionState, WorkerActionTrack, WorkerHitCount,
    WorkerHitCountConfig, WorkerHitRecord, WorkerHitTimeline, WorkerHitVideoAnalysisOptions,
    WorkerHitVideoAnalysisRun, WorkerHitVideoOutputFiles, WorkerPoseDetection, WorkerPoseFrame,
    WorkerTrackId,
};

const POSE_MODEL_INPUT_SHAPE: &[usize] = &[1, 3, 640, 640];
const POSE_OUTPUT_NAME: &str = "output0";
const POSE_OUTPUT_CHANNELS: usize = 56;
const POSE_OUTPUT_CANDIDATES: usize = 8400;
const POSE_NMS_THRESHOLD: f32 = 0.45;
const TRACK_MAX_NORMALIZED_DISTANCE: f32 = 0.20;
const LEFT_WRIST_INDEX: usize = 9;
const RIGHT_WRIST_INDEX: usize = 10;

impl WorkerHitCountConfig {
    fn validate(self) -> WorkerHitCountingResult<()> {
        validate_unit_score("strike_score_threshold", self.strike_score_threshold)?;
        validate_unit_score("contact_score_threshold", self.contact_score_threshold)?;
        validate_unit_score(
            "target_response_score_threshold",
            self.target_response_score_threshold,
        )?;
        if self.min_hit_gap_ms == 0 {
            return Err(WorkerHitCountingError::invalid_visual_action_input(
                "min_hit_gap_ms must be greater than 0",
            ));
        }
        if self.min_invalid_candidate_gap_ms == 0 {
            return Err(WorkerHitCountingError::invalid_visual_action_input(
                "min_invalid_candidate_gap_ms must be greater than 0",
            ));
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
) -> WorkerHitCountingResult<WorkerHitCount> {
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
) -> WorkerHitCountingResult<WorkerHitTimeline> {
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
) -> WorkerHitCountingResult<WorkerHitVideoAnalysisRun> {
    validate_video_analysis_options(options)?;
    let video_path = std::fs::canonicalize(video_path.as_ref())
        .map_err(|source| WorkerHitCountingError::io(video_path.as_ref().to_path_buf(), source))?;
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
        .map_err(|source| WorkerHitCountingError::io(files.extracted_frame_dir.clone(), source))?;
    fs::create_dir_all(&files.annotated_frame_dir)
        .map_err(|source| WorkerHitCountingError::io(files.annotated_frame_dir.clone(), source))?;
    fs::copy(&video_path, &files.source_input_video)
        .map_err(|source| WorkerHitCountingError::io(video_path.clone(), source))?;

    extract_video_frames(&video_path, &files.extracted_frame_dir, options)?;
    let frame_paths = collected_frame_paths(&files.extracted_frame_dir, options.max_frames)?;
    let mut pose_frames = Vec::new();
    for (index, frame_path) in frame_paths.iter().enumerate() {
        let frame_index = index as u64;
        let timestamp_ms = frame_timestamp_ms(index, options.sample_fps);
        let image = image::open(frame_path)?;
        let poses = detect_worker_poses_in_image(&image, frame_index, options)?;
        let annotated_frame_path = files.annotated_frame_dir.join(format!("frame_{index:05}.png"));
        write_pose_annotation_frame(
            &image,
            &annotated_frame_path,
            &poses,
            options.target_roi.target_box,
        )?;
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

    let action_observations =
        action_observations_from_pose_frames(
            &pose_frames,
            options.target_roi,
            options.keypoint_score_threshold,
        );
    let timeline = record_worker_hit_timeline_from_visual_observations(
        &action_observations,
        options.hit_count_config,
    )?;

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

fn validate_observation(observation: &WorkerActionObservation) -> WorkerHitCountingResult<()> {
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

fn validate_unit_score(field: &str, value: f32) -> WorkerHitCountingResult<()> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(WorkerHitCountingError::invalid_visual_action_input(
            format!("{field} must be finite and within 0.0..=1.0"),
        ))
    }
}

fn validate_normalized_point(field: &str, point: NormalizedPoint) -> WorkerHitCountingResult<()> {
    if point.x.is_finite()
        && point.y.is_finite()
        && (0.0..=1.0).contains(&point.x)
        && (0.0..=1.0).contains(&point.y)
    {
        Ok(())
    } else {
        Err(WorkerHitCountingError::invalid_visual_action_input(
            format!("{field} coordinates must be finite and within 0.0..=1.0"),
        ))
    }
}

fn validate_normalized_box(
    field: &str,
    bbox: NormalizedBoundingBox,
) -> WorkerHitCountingResult<()> {
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
        Err(WorkerHitCountingError::invalid_visual_action_input(
            format!("{field} must be finite and normalized within frame bounds"),
        ))
    }
}

fn detect_worker_poses_in_image(
    image: &DynamicImage,
    _frame_index: u64,
    options: &WorkerHitVideoAnalysisOptions,
) -> WorkerHitCountingResult<Vec<WorkerPoseDetection>> {
    let prepared = prepare_pose_image(image);
    let output = run_pose_model(&options.pose_model_path, prepared.tensor_data)?;
    decode_pose_output(
        &output,
        image.width(),
        image.height(),
        options.pose_score_threshold,
    )
}

fn prepare_pose_image(image: &DynamicImage) -> PreparedPoseImage {
    let preview = image.resize_exact(640, 640, FilterType::Triangle).to_rgb8();
    let tensor_data = rgb_to_nchw_f32_normalized(&preview);
    PreparedPoseImage { tensor_data }
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

fn run_pose_model(
    model_path: &Path,
    tensor_data: Vec<f32>,
) -> WorkerHitCountingResult<PoseOutputTensor> {
    if !model_path.is_file() {
        return Err(WorkerHitCountingError::io(
            model_path.to_path_buf(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "pose model file not found"),
        ));
    }

    let mut builder = Session::builder()?;
    let mut session = builder.commit_from_file(model_path)?;
    let input_array =
        ArrayD::from_shape_vec(IxDyn(POSE_MODEL_INPUT_SHAPE), tensor_data).map_err(|source| {
            WorkerHitCountingError::InvalidPoseTensorShape {
                reason: source.to_string(),
            }
        })?;
    let input = Tensor::from_array(input_array)?;
    let outputs = session.run(ort::inputs![input])?;
    let value =
        outputs
            .get(POSE_OUTPUT_NAME)
            .ok_or_else(|| WorkerHitCountingError::InvalidPoseTensorShape {
                reason: format!("missing pose output `{POSE_OUTPUT_NAME}`"),
            })?;
    let ValueType::Tensor { ty, .. } = value.dtype() else {
        return Err(WorkerHitCountingError::InvalidPoseTensorShape {
            reason: format!("pose output is not tensor: {}", value.dtype()),
        });
    };
    if !matches!(ty, TensorElementType::Float32) {
        return Err(WorkerHitCountingError::InvalidPoseTensorShape {
            reason: format!("pose output expected f32, got {ty}"),
        });
    }
    let (shape, data) = value.try_extract_tensor::<f32>()?;
    let shape = shape.iter().copied().collect::<Vec<_>>();
    if shape.as_slice() != [1, POSE_OUTPUT_CHANNELS as i64, POSE_OUTPUT_CANDIDATES as i64] {
        return Err(WorkerHitCountingError::InvalidPoseTensorShape {
            reason: format!(
                "pose output expected [1, {POSE_OUTPUT_CHANNELS}, {POSE_OUTPUT_CANDIDATES}], got {shape:?}"
            ),
        });
    }
    Ok(PoseOutputTensor {
        data: data.to_vec(),
    })
}

fn decode_pose_output(
    output: &PoseOutputTensor,
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
) -> WorkerHitCountingResult<Vec<WorkerPoseDetection>> {
    let mut poses = Vec::new();
    let width_scale = image_width as f32 / 640.0;
    let height_scale = image_height as f32 / 640.0;
    for candidate_index in 0..POSE_OUTPUT_CANDIDATES {
        let confidence = pose_value(output, 4, candidate_index);
        if confidence < score_threshold {
            continue;
        }
        let center_x = pose_value(output, 0, candidate_index) * width_scale;
        let center_y = pose_value(output, 1, candidate_index) * height_scale;
        let width = pose_value(output, 2, candidate_index) * width_scale;
        let height = pose_value(output, 3, candidate_index) * height_scale;
        let keypoints = (0..17)
            .map(|keypoint_index| {
                let channel = 5 + keypoint_index * 3;
                PoseKeypoint {
                    x: pose_value(output, channel, candidate_index) * width_scale,
                    y: pose_value(output, channel + 1, candidate_index) * height_scale,
                    confidence: pose_value(output, channel + 2, candidate_index),
                }
            })
            .collect::<Vec<_>>();
        poses.push(WorkerPoseDetection {
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
        });
    }

    Ok(non_maximum_suppression_pose(poses, POSE_NMS_THRESHOLD))
}

fn pose_value(output: &PoseOutputTensor, channel: usize, candidate_index: usize) -> f32 {
    output.data[channel * POSE_OUTPUT_CANDIDATES + candidate_index]
}

fn action_observations_from_pose_frames(
    pose_frames: &[WorkerPoseFrame],
    target_roi: VisualTargetObservation,
    keypoint_score_threshold: f32,
) -> Vec<WorkerActionObservation> {
    let mut previous_wrists = BTreeMap::<WorkerTrackId, NormalizedPoint>::new();
    let mut observations = Vec::new();
    for frame in pose_frames {
        for pose in &frame.poses {
            let person_id = pose.local_person_index as WorkerTrackId + 1;
            let Some(wrist) =
                best_wrist_point(pose, frame.frame_width, frame.frame_height, keypoint_score_threshold)
            else {
                continue;
            };
            let wrist_movement = previous_wrists
                .get(&person_id)
                .map_or(0.0, |previous| normalized_distance(*previous, wrist));
            previous_wrists.insert(person_id, wrist);

            let contact_score = containment_score(target_roi.target_box, wrist);
            let strike_score = (wrist_movement * 8.0).clamp(0.0, 1.0);
            let contacted_target = (contact_score > 0.0).then_some(target_roi);
            observations.push(WorkerActionObservation {
                person_id,
                frame_index: frame.frame_index,
                timestamp_ms: frame.timestamp_ms,
                person_box: pose.person_box,
                strike_score,
                contact_score,
                contact_point: Some(wrist),
                contacted_target,
                target_response_score: contact_score,
            });
        }
    }
    observations
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

fn write_pose_annotation_frame(
    image: &DynamicImage,
    output_path: &Path,
    poses: &[WorkerPoseDetection],
    target_roi: NormalizedBoundingBox,
) -> WorkerHitCountingResult<()> {
    let mut canvas = image.to_rgb8();
    draw_normalized_rect(&mut canvas, target_roi, Rgb([255, 215, 0]));
    for pose in poses {
        draw_normalized_rect(&mut canvas, pose.person_box, Rgb([0, 220, 80]));
        for keypoint in &pose.keypoints {
            if keypoint.confidence > 0.10 {
                draw_filled_circle_mut(
                    &mut canvas,
                    (keypoint.x.round() as i32, keypoint.y.round() as i32),
                    3,
                    Rgb([0, 180, 255]),
                );
            }
        }
    }
    canvas.save(output_path)?;
    Ok(())
}

fn draw_normalized_rect(image: &mut RgbImage, rect: NormalizedBoundingBox, color: Rgb<u8>) {
    let x = (rect.x * image.width() as f32).round() as i32;
    let y = (rect.y * image.height() as f32).round() as i32;
    let width = (rect.width * image.width() as f32).round().max(1.0) as u32;
    let height = (rect.height * image.height() as f32).round().max(1.0) as u32;
    draw_hollow_rect_mut(image, Rect::at(x, y).of_size(width, height), color);
}

fn non_maximum_suppression_pose(
    mut poses: Vec<WorkerPoseDetection>,
    nms_threshold: f32,
) -> Vec<WorkerPoseDetection> {
    poses.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    let mut kept: Vec<WorkerPoseDetection> = Vec::new();
    for candidate in poses {
        let overlaps_existing = kept.iter().any(|selected| {
            intersection_over_union(candidate.person_box, selected.person_box) > nms_threshold
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

fn validate_video_analysis_options(
    options: &WorkerHitVideoAnalysisOptions,
) -> WorkerHitCountingResult<()> {
    options.hit_count_config.validate()?;
    validate_unit_score("pose_score_threshold", options.pose_score_threshold)?;
    validate_unit_score("keypoint_score_threshold", options.keypoint_score_threshold)?;
    validate_normalized_box("target_roi.target_box", options.target_roi.target_box)?;
    if options.sample_fps == 0 {
        return Err(WorkerHitCountingError::invalid_visual_action_input(
            "sample_fps must be greater than 0",
        ));
    }
    if !options.pose_model_path.is_file() {
        return Err(WorkerHitCountingError::io(
            options.pose_model_path.clone(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "pose model file not found"),
        ));
    }
    Ok(())
}

fn recreate_dir(path: &Path) -> WorkerHitCountingResult<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|source| WorkerHitCountingError::io(path.to_path_buf(), source))?;
    }
    fs::create_dir_all(path)
        .map_err(|source| WorkerHitCountingError::io(path.to_path_buf(), source))
}

fn extract_video_frames(
    video_path: &Path,
    extracted_frame_dir: &Path,
    options: &WorkerHitVideoAnalysisOptions,
) -> WorkerHitCountingResult<()> {
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
) -> WorkerHitCountingResult<()> {
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
            "-c:v".to_owned(),
            "libx264".to_owned(),
            "-pix_fmt".to_owned(),
            "yuv420p".to_owned(),
            annotated_video.display().to_string(),
        ],
        annotated_video,
    )
}

fn run_ffmpeg(
    ffmpeg_path: &Path,
    args: &[String],
    context_path: &Path,
) -> WorkerHitCountingResult<()> {
    let output = Command::new(ffmpeg_path)
        .args(args)
        .output()
        .map_err(|source| WorkerHitCountingError::io(ffmpeg_path.to_path_buf(), source))?;
    if output.status.success() {
        return Ok(());
    }
    Err(WorkerHitCountingError::io(
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
) -> WorkerHitCountingResult<Vec<PathBuf>> {
    let mut frame_paths = fs::read_dir(extracted_frame_dir)
        .map_err(|source| WorkerHitCountingError::io(extracted_frame_dir.to_path_buf(), source))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| {
                    WorkerHitCountingError::io(extracted_frame_dir.to_path_buf(), source)
                })
        })
        .collect::<WorkerHitCountingResult<Vec<_>>>()?;
    frame_paths.sort();
    if let Some(limit) = max_frames {
        frame_paths.truncate(limit);
    }
    if frame_paths.is_empty() {
        return Err(WorkerHitCountingError::io(
            extracted_frame_dir.to_path_buf(),
            std::io::Error::other("ffmpeg did not extract any video frames"),
        ));
    }
    Ok(frame_paths)
}

fn frame_timestamp_ms(frame_index: usize, sample_fps: u32) -> u64 {
    (frame_index as u64 * 1_000) / u64::from(sample_fps)
}

fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) -> WorkerHitCountingResult<()> {
    let json = serde_json::to_string_pretty(value).map_err(|source| {
        WorkerHitCountingError::io(path.to_path_buf(), std::io::Error::other(source.to_string()))
    })?;
    fs::write(path, json).map_err(|source| WorkerHitCountingError::io(path.to_path_buf(), source))
}

#[derive(Clone, Debug)]
struct PreparedPoseImage {
    tensor_data: Vec<f32>,
}

#[derive(Clone, Debug)]
struct PoseOutputTensor {
    data: Vec<f32>,
}
