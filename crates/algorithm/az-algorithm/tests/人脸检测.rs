use std::path::{Path, PathBuf};

use anyhow::Context;
use az_algorithm::components::face_detection::assist::{
    detect_faces_from_base64_with_options, detect_faces_from_bytes_with_options,
    detect_faces_from_path_with_options,
};
use az_algorithm::components::face_detection::model::{FaceDetectionOptions, FaceDetectionRun};
use base64::Engine;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/face_detection/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn model_path() -> PathBuf {
    let path = "face_detection_scrfd_500m.onnx";
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/face_detection/models")
            .join(path),
    )
    .expect("人脸检测模型必须存在")
}

fn options(output_name: &str) -> FaceDetectionOptions {
    FaceDetectionOptions {
        model_path: model_path(),
        output_dir: workspace_root()
            .join("target/az-algorithm-results")
            .join(output_name),
        score_threshold: 0.5,
        nms_threshold: 0.4,
    }
}

#[expect(
    clippy::dbg_macro,
    reason = "测试需要直接打印调用方最关心的输入、模型、输出绝对路径"
)]
fn assert_real_outputs_exist(result: &FaceDetectionRun) -> anyhow::Result<()> {
    dbg!(&result.input_path);
    // dbg!(&result.model_path);
    dbg!(&result.files.source_input);
    dbg!(&result.files.model_input_preview);
    dbg!(&result.files.raw_outputs_json);
    dbg!(&result.files.detected_faces_json);
    dbg!(&result.files.detected_faces_image);

    assert!(
        !result.faces.is_empty(),
        "真实人脸图片必须至少检测到一个人脸框"
    );
    assert_existing_file(&result.files.source_input);
    assert_existing_file(&result.files.model_input_preview);
    assert_existing_file(&result.files.raw_outputs_json);
    assert_existing_file(&result.files.detected_faces_json);
    assert_existing_file(&result.files.detected_faces_image);
    Ok(())
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[test]
fn detect_faces_from_path_should_write_marked_image() -> anyhow::Result<()> {
    // 输入：绝对图片路径。
    //
    // 输出：
    // target/az-algorithm-results/face_detection_path/source_input.jpg
    // target/az-algorithm-results/face_detection_path/model_input_preview.png
    // target/az-algorithm-results/face_detection_path/raw_outputs.json
    // target/az-algorithm-results/face_detection_path/detected_faces.json
    // target/az-algorithm-results/face_detection_path/detected_faces.png
    let result = detect_faces_from_path_with_options(
        fixture_path("face.jpg"),
        &options("face_detection_path"),
    )?;

    assert_real_outputs_exist(&result)
}

#[test]
fn detect_faces_from_bytes_should_write_marked_image() -> anyhow::Result<()> {
    // 输入：图片二进制。
    //
    // 输出：
    // target/az-algorithm-results/face_detection_bytes/detected_faces.png
    let bytes = std::fs::read(fixture_path("face.jpg")).with_context(|| {
        format!(
            "filesystem error at `{}`",
            (fixture_path("face.jpg")).display()
        )
    })?;
    let result = detect_faces_from_bytes_with_options(&bytes, &options("face_detection_bytes"))?;

    assert_real_outputs_exist(&result)
}

#[test]
fn detect_faces_from_base64_should_write_marked_image() -> anyhow::Result<()> {
    // 输入：base64 图片字符串。
    //
    // 输出：
    // target/az-algorithm-results/face_detection_base64/detected_faces.png
    let bytes = std::fs::read(fixture_path("face.jpg")).with_context(|| {
        format!(
            "filesystem error at `{}`",
            (fixture_path("face.jpg")).display()
        )
    })?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let result =
        detect_faces_from_base64_with_options(&encoded, &options("face_detection_base64"))?;

    assert_real_outputs_exist(&result)
}
