use az_algorithm::components::worker_hit_counting::assist::{
    analyze_worker_hits_in_video_from_path, annotate_worker_hits_video,
    count_worker_hits_by_person_from_visual_observations,
    default_worker_hit_video_analysis_options, record_worker_hit_timeline_from_visual_observations,
};
use az_algorithm::components::worker_hit_counting::model::{
    InvalidHitReason, NormalizedBoundingBox, NormalizedPoint, VisualTargetKind,
    VisualTargetObservation, WorkerActionObservation, WorkerActionState, WorkerHitCountConfig,
    WorkerHitVideoAnalysisOptions, WorkerHitVideoAnalysisRun,
};
use std::path::{Path, PathBuf};

#[test]
fn worker_hit_counting_should_count_only_hits_on_hanging_metal_panel() -> anyhow::Result<()> {
    let result = count_worker_hits_by_person_from_visual_observations(
        &[
            observation(
                1,
                1,
                0,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(1, 2, 260, target(200, VisualTargetKind::ConveyorBody), 0.90),
        ],
        WorkerHitCountConfig::default(),
    )?;

    let worker = &result.workers[0];
    // 关键断言：敲中悬挂金属板才计为有效敲击，敲流水线台体边缘不计数。
    assert_eq!(
        (
            worker.valid_hit_count,
            worker.invalid_candidate_count,
            worker.invalid_candidates[0].reason
        ),
        (1, 1, InvalidHitReason::ContactOnInvalidTarget)
    );
    Ok(())
}

#[test]
fn worker_hit_counting_result_should_return_hit_count_by_person_id() -> anyhow::Result<()> {
    let result = count_worker_hits_by_person_from_visual_observations(
        &[
            observation(
                1,
                1,
                0,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                2,
                1,
                0,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
        ],
        WorkerHitCountConfig::default(),
    )?;

    assert_eq!(result.valid_hit_count_of(2), Some(1));
    Ok(())
}

#[test]
fn worker_hit_counting_should_require_target_response() -> anyhow::Result<()> {
    let result = count_worker_hits_by_person_from_visual_observations(
        &[observation(
            1,
            1,
            0,
            target(100, VisualTargetKind::HangingMetalPanel),
            0.10,
        )],
        WorkerHitCountConfig::default(),
    )?;

    let worker = &result.workers[0];
    assert_eq!(
        (worker.valid_hit_count, worker.invalid_candidates[0].reason),
        (0, InvalidHitReason::MissingTargetResponse)
    );
    Ok(())
}

#[test]
fn worker_hit_counting_should_count_hits_per_person() -> anyhow::Result<()> {
    let result = count_worker_hits_by_person_from_visual_observations(
        &[
            observation(
                1,
                1,
                0,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                2,
                1,
                0,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                1,
                2,
                260,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                2,
                2,
                520,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
        ],
        WorkerHitCountConfig::default(),
    )?;

    let first = result
        .workers
        .iter()
        .find(|worker| worker.person_id == 1)
        .expect("person 1 should exist");
    let second = result
        .workers
        .iter()
        .find(|worker| worker.person_id == 2)
        .expect("person 2 should exist");

    // 关键断言：有效敲击次数必须按人员分别统计，不能把所有人汇总成一个总数。
    assert_eq!((first.valid_hit_count, second.valid_hit_count), (2, 2));
    Ok(())
}

#[test]
fn worker_hit_counting_should_record_action_state_and_hit_events_per_frame() -> anyhow::Result<()> {
    let timeline = record_worker_hit_timeline_from_visual_observations(
        &[
            observation(
                1,
                10,
                1_000,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                1,
                11,
                1_120,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                1,
                12,
                1_260,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                1,
                13,
                1_520,
                target(300, VisualTargetKind::SupportStructure),
                0.90,
            ),
        ],
        WorkerHitCountConfig::default(),
    )?;

    let states = timeline
        .frame_records
        .iter()
        .map(|record| {
            (
                record.frame_index,
                record.state,
                record.valid_hit_count,
                record.new_valid_hit.as_ref().map(|hit| hit.hit_index),
                record
                    .new_invalid_candidate
                    .as_ref()
                    .map(|candidate| candidate.candidate_index),
            )
        })
        .collect::<Vec<_>>();

    // 关键断言：时间线必须能说明每帧动作状态，以及哪一帧新增了第几次敲击。
    assert_eq!(
        states,
        vec![
            (10, WorkerActionState::ValidHit, 1, Some(0), None),
            (11, WorkerActionState::Striking, 1, None, None),
            (12, WorkerActionState::ValidHit, 2, Some(1), None),
            (13, WorkerActionState::InvalidHitCandidate, 2, None, Some(0)),
        ]
    );
    assert_eq!(timeline.final_count.workers[0].valid_hit_count, 2);
    Ok(())
}

#[test]
fn worker_hit_counting_should_report_invalid_candidate_state() -> anyhow::Result<()> {
    let result = count_worker_hits_by_person_from_visual_observations(
        &[observation(
            7,
            10,
            1_000,
            target(300, VisualTargetKind::SupportStructure),
            0.90,
        )],
        WorkerHitCountConfig::default(),
    )?;

    assert_eq!(
        result.workers[0].state,
        WorkerActionState::InvalidHitCandidate
    );
    Ok(())
}

#[test]
fn worker_hit_counting_should_merge_valid_hits_inside_minimum_gap() -> anyhow::Result<()> {
    let result = count_worker_hits_by_person_from_visual_observations(
        &[
            observation(
                7,
                10,
                1_000,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                7,
                11,
                1_120,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
            observation(
                7,
                12,
                1_260,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            ),
        ],
        WorkerHitCountConfig::default(),
    )?;

    // 关键断言：220ms 内的接触峰值是同一次动作，不应重复计数。
    assert_eq!(
        result.workers[0]
            .valid_hits
            .iter()
            .map(|hit| hit.timestamp_ms)
            .collect::<Vec<_>>(),
        vec![1_000, 1_260]
    );
    Ok(())
}

#[test]
fn worker_hit_counting_should_reject_invalid_visual_scores() {
    let err = count_worker_hits_by_person_from_visual_observations(
        &[WorkerActionObservation {
            strike_score: 1.20,
            ..observation(
                1,
                1,
                0,
                target(100, VisualTargetKind::HangingMetalPanel),
                0.90,
            )
        }],
        WorkerHitCountConfig::default(),
    )
    .unwrap_err();

    assert_eq!(
        err.to_string(),
        "invalid visual action input: strike_score must be finite and within 0.0..=1.0"
    );
}

#[test]
fn worker_hit_counting_should_build_default_one_line_video_options() -> anyhow::Result<()> {
    let user_video = PathBuf::from("/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4");
    if !user_video.is_file() {
        eprintln!("跳过默认配置测试，用户视频不存在：{}", user_video.display());
        return Ok(());
    }

    let options = default_worker_hit_video_analysis_options(&user_video)?;

    assert_eq!(
        options.output_dir,
        workspace_root()
            .join("target/az-algorithm-results/worker-hit-counting")
            .join("246f5787eca62dc0b462dbc041da756f")
    );
    Ok(())
}

// 合成视频
#[test]
fn worker_hit_counting_should_reject_missing_video_for_one_line_api() {
    let err =
        annotate_worker_hits_video("/Users/zjarlin/Desktop/not-exists-worker-hit-counting.mp4")
            .unwrap_err();

    assert!(err.to_string().contains("filesystem error at"));
}

#[test]
#[expect(
    clippy::dbg_macro,
    reason = "测试需要直接打印真实视频输入、模型和输出绝对路径"
)]
fn worker_hit_counting_should_analyze_user_video_with_real_pose_model() -> anyhow::Result<()> {
    // 输入：/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4
    //
    // 输出：
    // target/az-algorithm-results/worker_hit_counting_user_video/source_input.mp4
    // target/az-algorithm-results/worker_hit_counting_user_video/pose_frames.json
    // target/az-algorithm-results/worker_hit_counting_user_video/action_observations.json
    // target/az-algorithm-results/worker_hit_counting_user_video/worker_hit_timeline.json
    // target/az-algorithm-results/worker_hit_counting_user_video/annotated_worker_hits.mp4
    let user_video = PathBuf::from("/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4");
    if !user_video.is_file() {
        eprintln!("跳过真实视频测试，用户视频不存在：{}", user_video.display());
        return Ok(());
    }

    let result = analyze_worker_hits_in_video_from_path(
        &user_video,
        &WorkerHitVideoAnalysisOptions {
            pose_model_path: pose_model_path(),
            ffmpeg_path: PathBuf::from("/opt/homebrew/bin/ffmpeg"),
            output_dir: workspace_root()
                .join("target/az-algorithm-results")
                .join("worker_hit_counting_user_video"),
            sample_fps: 1,
            output_fps: 30,
            max_frames: Some(6),
            pose_score_threshold: 0.01,
            keypoint_score_threshold: 0.01,
            target_roi: target(100, VisualTargetKind::HangingMetalPanel),
            hit_count_config: WorkerHitCountConfig {
                strike_score_threshold: 0.05,
                contact_score_threshold: 0.50,
                target_response_score_threshold: 0.50,
                min_hit_gap_ms: 220,
                min_invalid_candidate_gap_ms: 220,
                strike_hold_ms: 180,
            },
        },
    )?;

    dbg!(&result.input_video_path);
    dbg!(&result.pose_model_path);
    dbg!(&result.files.source_input_video);
    dbg!(&result.files.extracted_frame_dir);
    dbg!(&result.files.annotated_frame_dir);
    dbg!(&result.files.pose_frames_json);
    dbg!(&result.files.action_observations_json);
    dbg!(&result.files.worker_hit_timeline_json);
    dbg!(&result.files.annotated_video);

    assert_real_video_outputs_exist(&result);
    Ok(())
}

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn pose_model_path() -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/worker_hit_counting/models")
            .join("yolov8n_pose.onnx"),
    )
    .expect("YOLOv8n pose 模型必须存在")
}

fn assert_real_video_outputs_exist(result: &WorkerHitVideoAnalysisRun) {
    assert_existing_file(&result.files.source_input_video);
    assert_existing_file(&result.files.pose_frames_json);
    assert_existing_file(&result.files.action_observations_json);
    assert_existing_file(&result.files.worker_hit_timeline_json);
    assert_existing_file(&result.files.annotated_video);
    assert!(!result.pose_frames.is_empty(), "真实视频必须至少处理一帧");
    assert!(
        result
            .pose_frames
            .iter()
            .any(|frame| !frame.poses.is_empty()),
        "真实视频抽帧必须至少产生一个 pose 候选"
    );
    assert!(
        !result.action_observations.is_empty(),
        "pose + ROI 规则必须产生动作观测，计数器才能工作"
    );
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

fn observation(
    person_id: u64,
    frame_index: u64,
    timestamp_ms: u64,
    contacted_target: VisualTargetObservation,
    target_response_score: f32,
) -> WorkerActionObservation {
    WorkerActionObservation {
        person_id,
        frame_index,
        timestamp_ms,
        person_box: NormalizedBoundingBox {
            x: person_id as f32 * 0.01,
            y: 0.20,
            width: 0.10,
            height: 0.30,
        },
        strike_score: 0.90,
        contact_score: 0.90,
        contact_point: Some(NormalizedPoint { x: 0.52, y: 0.32 }),
        contacted_target: Some(contacted_target),
        target_response_score,
    }
}

fn target(target_id: u64, kind: VisualTargetKind) -> VisualTargetObservation {
    VisualTargetObservation {
        target_id,
        kind,
        target_box: NormalizedBoundingBox {
            x: 0.40,
            y: 0.20,
            width: 0.30,
            height: 0.30,
        },
        containment_score: 0.95,
    }
}
