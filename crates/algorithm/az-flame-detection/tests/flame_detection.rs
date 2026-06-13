use std::path::{Path, PathBuf};

use az_flame_detection::error::FlameDetectionResult;
use az_flame_detection::logic_flame_detection::assist::run_flame_detection_from_path_with_output;

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

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
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
