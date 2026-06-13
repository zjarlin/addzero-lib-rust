use std::path::{Path, PathBuf};

use az_algorithm_onnx::error::OnnxImageResult;
use az_face_recognition::logic_face_recognition::assist::run_face_recognition_from_path_with_output;

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
        .join("face_recognition")
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[test]
fn face_recognition_should_run_real_image_and_write_outputs() -> OnnxImageResult<()> {
    // 输入图片：
    // crates/algorithm/az-face-recognition/tests/fixtures/input/face.jpg
    //
    // 模型：
    // crates/algorithm/az-face-recognition/resources/models/face_recognition_arcface_resnet100_int8.onnx
    //
    // 输出：
    // target/az-algorithm-results/face_recognition/source_input.jpg
    // target/az-algorithm-results/face_recognition/model_input_preview.png
    // target/az-algorithm-results/face_recognition/raw_outputs.json
    let result =
        run_face_recognition_from_path_with_output(fixture_path("face.jpg"), output_dir())?;

    // 关键断言：验证真实模型输出和落盘文件，不接受只返回 Ok 的假测试。
    assert!(!result.raw_outputs.is_empty());
    assert_existing_file(&result.files.source_input);
    assert_existing_file(&result.files.model_input_preview);
    assert_existing_file(&result.files.raw_outputs_json);
    Ok(())
}
