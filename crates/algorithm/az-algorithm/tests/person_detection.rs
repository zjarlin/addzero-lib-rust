use std::path::{Path, PathBuf};

use anyhow::Context;
use az_algorithm::components::person_detection::assist::{
    detect_persons_from_base64_with_options, detect_persons_from_path_with_options,
    detect_persons_in_video_from_path,
};
use az_algorithm::components::person_detection::model::{
    DEFAULT_SCORE_THRESHOLD, PersonDetectionModelKind, PersonDetectionOptions, PersonDetectionRun,
    PersonVideoDetectionOptions, PersonVideoDetectionRun,
};
use base64::Engine;

const USER_VIDEO_YOLO_SCORE_THRESHOLD: f32 = 0.01;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/person_detection/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn model_path() -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/person_detection/models")
            .join("coco_ssd_mobilenet_v1_10.onnx"),
    )
    .expect("人员检测模型必须存在")
}

fn yolo_model_path() -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/person_detection/models")
            .join("yolo11n_coco.onnx"),
    )
    .expect("YOLO11n 人员检测模型必须存在")
}

fn options(output_name: &str) -> PersonDetectionOptions {
    PersonDetectionOptions {
        model_path: model_path(),
        model_kind: PersonDetectionModelKind::CocoSsdMobileNetV1,
        output_dir: workspace_root()
            .join("target/az-algorithm-results")
            .join(output_name),
        score_threshold: DEFAULT_SCORE_THRESHOLD,
    }
}

#[expect(
    clippy::dbg_macro,
    reason = "测试需要直接打印调用方最关心的输入、模型、输出绝对路径"
)]
fn assert_real_outputs_exist(result: &PersonDetectionRun) -> anyhow::Result<()> {
    dbg!(&result.input_path);
    dbg!(&result.model_path);
    dbg!(&result.files.source_input);
    dbg!(&result.files.model_input_preview);
    dbg!(&result.files.raw_outputs_json);
    dbg!(&result.files.detected_persons_json);
    dbg!(&result.files.detected_persons_image);

    assert!(
        !result.persons.is_empty(),
        "真实人员图片必须至少检测到一个人员框"
    );
    assert_existing_file(&result.files.source_input);
    assert_existing_file(&result.files.model_input_preview);
    assert_existing_file(&result.files.raw_outputs_json);
    assert_existing_file(&result.files.detected_persons_json);
    assert_existing_file(&result.files.detected_persons_image);
    Ok(())
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[expect(
    clippy::dbg_macro,
    reason = "测试需要直接打印真实视频输入、模型、输出绝对路径"
)]
fn assert_real_video_outputs_exist(result: &PersonVideoDetectionRun) {
    dbg!(&result.input_video_path);
    dbg!(&result.model_path);
    dbg!(&result.files.source_input_video);
    dbg!(&result.files.extracted_frame_dir);
    dbg!(&result.files.annotated_frame_dir);
    dbg!(&result.files.frame_detections_json);
    dbg!(&result.files.annotated_video);

    assert_existing_file(&result.files.source_input_video);
    assert!(
        result.files.extracted_frame_dir.is_dir(),
        "抽帧目录必须存在：{}",
        result.files.extracted_frame_dir.display()
    );
    assert!(
        result.files.annotated_frame_dir.is_dir(),
        "标注帧目录必须存在：{}",
        result.files.annotated_frame_dir.display()
    );
    assert_existing_file(&result.files.frame_detections_json);
    assert_existing_file(&result.files.annotated_video);
    assert!(!result.frames.is_empty(), "真实视频必须至少处理一帧");
    assert!(
        result.frames.iter().any(|frame| !frame.persons.is_empty()),
        "YOLO11n 真实视频抽帧必须至少检测到一个人员框"
    );
}

#[test]
fn detect_persons_from_path_should_write_marked_image() -> anyhow::Result<()> {
    // 输入：绝对图片路径。
    //
    // 输出：
    // target/az-algorithm-results/person_detection_path/source_input.jpg
    // target/az-algorithm-results/person_detection_path/model_input_preview.png
    // target/az-algorithm-results/person_detection_path/raw_outputs.json
    // target/az-algorithm-results/person_detection_path/detected_persons.json
    // target/az-algorithm-results/person_detection_path/detected_persons.png
    let result = detect_persons_from_path_with_options(
        fixture_path("person_vehicle.jpg"),
        &options("person_detection_path"),
    )?;

    assert_real_outputs_exist(&result)
}

#[test]
fn detect_persons_from_base64_should_write_marked_image() -> anyhow::Result<()> {
    // 输入：base64 图片字符串。
    //
    // 输出：
    // target/az-algorithm-results/person_detection_base64/detected_persons.png
    let bytes = std::fs::read(fixture_path("person_vehicle.jpg")).with_context(|| {
        format!(
            "filesystem error at `{}`",
            (fixture_path("person_vehicle.jpg")).display()
        )
    })?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let result =
        detect_persons_from_base64_with_options(&encoded, &options("person_detection_base64"))?;

    assert_real_outputs_exist(&result)
}

#[test]
fn detect_persons_in_user_video_should_write_annotated_video() -> anyhow::Result<()> {
    // 输入：/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4
    //
    // 输出：
    // target/az-algorithm-results/person_detection_user_video/source_input.mp4
    // target/az-algorithm-results/person_detection_user_video/extracted_frames
    // target/az-algorithm-results/person_detection_user_video/annotated_frames
    // target/az-algorithm-results/person_detection_user_video/frame_detections.json
    // target/az-algorithm-results/person_detection_user_video/annotated_persons.mp4
    let user_video = PathBuf::from("/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4");
    if !user_video.is_file() {
        eprintln!("跳过真实视频测试，用户视频不存在：{}", user_video.display());
        return Ok(());
    }

    let result = detect_persons_in_video_from_path(
        &user_video,
        &PersonVideoDetectionOptions {
            model_path: yolo_model_path(),
            model_kind: PersonDetectionModelKind::Yolo11nCoco,
            output_dir: workspace_root()
                .join("target/az-algorithm-results")
                .join("person_detection_user_video"),
            ffmpeg_path: PathBuf::from("/opt/homebrew/bin/ffmpeg"),
            sample_fps: 1,
            max_frames: Some(6),
            score_threshold: USER_VIDEO_YOLO_SCORE_THRESHOLD,
        },
    )?;

    assert_real_video_outputs_exist(&result);
    Ok(())
}
