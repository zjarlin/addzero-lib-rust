use std::path::{Path, PathBuf};

use az_flame_detection::error::FlameDetectionResult;
use az_flame_detection::logic_flame_detection::assist::{
    detect_flames_in_video_from_path, run_flame_detection_from_path_with_output,
};
use az_flame_detection::logic_flame_detection::model::{
    DEFAULT_NMS_THRESHOLD, DEFAULT_SCORE_THRESHOLD, FLAME_DETECTION_FIRE_SMOKE_YOLOV8N,
    FlameVideoDetectionOptions, FlameVideoDetectionRun,
};

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn output_dir() -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join("flame_detection")
}

fn video_output_dir() -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join("flame_detection_user_video")
}

fn model_path() -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/models")
            .join(FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.local_file),
    )
    .expect("fire/smoke YOLO 模型必须存在")
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

fn assert_existing_dir(path: &Path) {
    assert!(path.is_dir(), "输出目录必须存在：{}", path.display());
}

#[test]
fn flame_detection_should_run_real_image_and_write_outputs() -> FlameDetectionResult<()> {
    // 输入图片：crates/algorithm/az-flame-detection/tests/fixtures/input/flame.jpg
    // 输出：target/az-algorithm-results/flame_detection/detected_flames.png
    let result =
        run_flame_detection_from_path_with_output(fixture_path("flame.jpg"), output_dir())?;

    // 关键断言：验证真实模型输出、后处理检测框和可审阅标注图。
    assert!(!result.raw_outputs.is_empty());
    assert!(!result.detections.is_empty());
    assert_existing_file(&result.files.source_input);
    assert_existing_file(&result.files.model_input_preview);
    assert_existing_file(&result.files.raw_outputs_json);
    assert_existing_file(&result.files.detected_flames_json);
    assert_existing_file(&result.files.detected_flames_image);
    Ok(())
}

#[test]
fn flame_detection_should_analyze_user_video_and_write_annotated_video() -> FlameDetectionResult<()>
{
    // 输入：/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4
    //
    // 输出：
    // target/az-algorithm-results/flame_detection_user_video/source_input.mp4
    // target/az-algorithm-results/flame_detection_user_video/extracted_frames
    // target/az-algorithm-results/flame_detection_user_video/annotated_frames
    // target/az-algorithm-results/flame_detection_user_video/frame_detections.json
    // target/az-algorithm-results/flame_detection_user_video/annotated_flames.mp4
    let user_video = PathBuf::from("/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4");
    if !user_video.is_file() {
        eprintln!("跳过真实视频测试，用户视频不存在：{}", user_video.display());
        return Ok(());
    }

    let result = detect_flames_in_video_from_path(
        &user_video,
        &FlameVideoDetectionOptions {
            model_path: model_path(),
            output_dir: video_output_dir(),
            ffmpeg_path: PathBuf::from("/opt/homebrew/bin/ffmpeg"),
            sample_fps: 1,
            output_fps: 1,
            max_frames: Some(6),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
        },
    )?;

    assert_real_video_outputs_exist(&result);
    Ok(())
}

fn assert_real_video_outputs_exist(result: &FlameVideoDetectionRun) {
    assert_existing_file(&result.files.source_input_video);
    assert_existing_dir(&result.files.extracted_frame_dir);
    assert_existing_dir(&result.files.annotated_frame_dir);
    assert_existing_file(&result.files.frame_detections_json);
    assert_existing_file(&result.files.annotated_video);
    assert!(!result.frames.is_empty(), "真实视频必须至少处理一帧");
    assert!(
        result
            .frames
            .iter()
            .all(|frame| frame.annotated_frame_path.is_file()),
        "每个处理帧都必须生成标注帧，即使该帧没有 fire/smoke 检测框"
    );
}
