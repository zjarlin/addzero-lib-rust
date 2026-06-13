use std::path::{Path, PathBuf};

use az_ocr_text_recognition::logic_ocr_text_recognition::assist::run_ocr_text_recognition_from_path_with_output;

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

fn output_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join(name)
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[test]
fn ocr_text_recognition_should_run_detection_and_recognition_models() -> anyhow::Result<()> {
    // 输入图片：crates/algorithm/az-ocr-text-recognition/tests/fixtures/input/ocr_text.jpg
    //
    // 输出：
    // target/az-algorithm-results/ocr_text_detection/raw_outputs.json
    // target/az-algorithm-results/ocr_text_recognition/raw_outputs.json
    let result = run_ocr_text_recognition_from_path_with_output(
        fixture_path("ocr_text.jpg"),
        output_dir("ocr_text_detection"),
        output_dir("ocr_text_recognition"),
    )?;

    // 关键断言：验证检测模型和识别模型都真实执行并写出文件。
    assert!(!result.detection.raw_outputs.is_empty());
    assert!(!result.recognition.raw_outputs.is_empty());
    assert_existing_file(&result.detection.files.raw_outputs_json);
    assert_existing_file(&result.recognition.files.raw_outputs_json);
    Ok(())
}
