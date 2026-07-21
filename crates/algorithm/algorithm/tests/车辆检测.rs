use std::path::{Path, PathBuf};

use az_algorithm::components::vehicle_detection::assist::run_vehicle_detection_from_path_with_output;
use image::Rgb;

const VEHICLE_BOX_COLOR: Rgb<u8> = Rgb([255, 70, 40]);

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/vehicle_detection/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn output_dir(output_name: &str) -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join("vehicle_detection")
        .join(output_name)
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

fn assert_image_contains_vehicle_box(path: &Path) -> anyhow::Result<()> {
    let image = image::open(path)?.to_rgb8();
    let box_pixels = image
        .pixels()
        .filter(|pixel| **pixel == VEHICLE_BOX_COLOR)
        .count();
    assert!(
        box_pixels > 0,
        "车辆标注图必须包含检测框颜色像素：{}",
        path.display()
    );
    Ok(())
}

#[expect(
    clippy::dbg_macro,
    reason = "测试需要直接打印车辆检测输入、模型、输出绝对路径"
)]
fn assert_real_outputs_exist(
    result: &az_algorithm::components::vehicle_detection::model::VehicleDetectionRun,
) -> anyhow::Result<()> {
    dbg!(&result.input_path);
    dbg!(&result.model_path);
    dbg!(&result.files.source_input);
    dbg!(&result.files.model_input_preview);
    dbg!(&result.files.raw_outputs_json);
    dbg!(&result.files.detected_vehicles_json);
    dbg!(&result.files.detected_vehicles_image);

    assert!(
        !result.vehicles.is_empty(),
        "真实车辆图片必须至少检测到一个车辆框"
    );
    assert_existing_file(&result.files.source_input);
    assert_existing_file(&result.files.model_input_preview);
    assert_existing_file(&result.files.raw_outputs_json);
    assert_existing_file(&result.files.detected_vehicles_json);
    assert_existing_file(&result.files.detected_vehicles_image);
    assert_image_contains_vehicle_box(&result.files.model_input_preview)?;
    assert_image_contains_vehicle_box(&result.files.detected_vehicles_image)?;
    Ok(())
}

#[test]
fn vehicle_detection_should_run_test_image_and_write_marked_outputs() -> anyhow::Result<()> {
    let cases = [
        ("cars_traffic.jpg", "cars_traffic"),
        ("bus_street.jpg", "bus_street"),
    ];

    for (file_name, output_name) in cases {
        // 输入：crates/algorithm/algorithm/tests/fixtures/vehicle_detection/input/{file_name}
        // 输出：target/az-algorithm-results/vehicle_detection/{output_name}/model_input_preview.png
        // 输出：target/az-algorithm-results/vehicle_detection/{output_name}/detected_vehicles.png
        let result = run_vehicle_detection_from_path_with_output(
            fixture_path(file_name),
            output_dir(output_name),
        )?;

        // 关键断言：验证真实模型输出、车辆框和标注图落盘。
        assert!(!result.raw_outputs.is_empty());
        assert_real_outputs_exist(&result)?;
    }
    Ok(())
}
