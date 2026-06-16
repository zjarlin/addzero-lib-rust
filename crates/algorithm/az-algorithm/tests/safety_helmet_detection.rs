use std::path::{Path, PathBuf};

use az_algorithm::components::safety_helmet_detection::assist::run_safety_helmet_detection_from_path_with_output;
use az_algorithm::components::safety_helmet_detection::model::SafetyHelmetDetectionClass;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/safety_helmet_detection/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn output_dir() -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join("safety_helmet_detection")
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[test]
fn safety_helmet_detection_should_run_real_image_and_write_outputs() -> anyhow::Result<()> {
    // 输入图片：crates/algorithm/az-algorithm/tests/fixtures/safety_helmet_detection/input/safety_helmet.jpg
    // 输出：target/az-algorithm-results/safety_helmet_detection/detected_safety_helmets.json
    let result = run_safety_helmet_detection_from_path_with_output(
        fixture_path("safety_helmet.jpg"),
        output_dir(),
    )?;

    // 关键断言：验证真实模型输出、PPE 后处理 JSON 和标注图都落盘。
    assert!(!result.raw_outputs.is_empty());
    assert_existing_file(&result.files.source_input);
    assert_existing_file(&result.files.model_input_preview);
    assert_existing_file(&result.files.raw_outputs_json);
    assert_existing_file(&result.files.detected_safety_helmets_json);
    assert_existing_file(&result.files.detected_safety_helmets_image);
    for detection in &result.detections {
        assert_eq!(
            SafetyHelmetDetectionClass::from_class_index(detection.class_index),
            Some(detection.detection_class)
        );
    }
    Ok(())
}
