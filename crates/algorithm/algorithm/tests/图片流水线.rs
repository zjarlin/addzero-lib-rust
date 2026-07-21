use std::path::{Path, PathBuf};

use az_algorithm::pipeline::image::assist::run_image_pipeline_from_path;
use az_algorithm::pipeline::image::model::{ImageAlgorithmKind, ImagePipelineOptions};

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pipeline_image/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn ocr_fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/ocr/tests/fixtures/ocr_text_recognition/input")
            .join(file_name),
    )
    .expect("OCR 测试输入图片必须存在")
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
fn image_pipeline_should_stack_face_detection_and_safety_helmet_detection() -> anyhow::Result<()> {
    // 输入图片：crates/algorithm/algorithm/tests/fixtures/pipeline_image/input/safety_helmet.jpg
    //
    // 叠加算法：
    // - face_detection：输出到 target/az-algorithm-results/pipeline_face_helmet/face_detection
    // - safety_helmet_detection：输出到 target/az-algorithm-results/pipeline_face_helmet/safety_helmet_detection
    //
    // 汇总输出：
    // target/az-algorithm-results/pipeline_face_helmet/pipeline_results.json
    let run = run_image_pipeline_from_path(
        fixture_path("safety_helmet.jpg"),
        &ImagePipelineOptions {
            algorithms: vec![
                ImageAlgorithmKind::FaceDetection,
                ImageAlgorithmKind::SafetyHelmetDetection,
            ],
            output_dir: output_dir("pipeline_face_helmet"),
        },
    )?;

    // 关键断言：验证两个算法都在同一次 pipeline 中真实产出文件。
    assert_eq!(run.algorithm_runs.len(), 2);
    assert_existing_file(&run.summary_file);
    for algorithm_run in &run.algorithm_runs {
        assert!(
            !algorithm_run.files.is_empty(),
            "{} 必须产生输出文件",
            algorithm_run.code
        );
        for file in &algorithm_run.files {
            assert_existing_file(file);
        }
    }
    Ok(())
}

#[test]
fn image_pipeline_should_route_ocr_to_az_ocr_crate() -> anyhow::Result<()> {
    let run = run_image_pipeline_from_path(
        ocr_fixture_path("ocr_text.jpg"),
        &ImagePipelineOptions {
            algorithms: vec![ImageAlgorithmKind::OcrTextRecognition],
            output_dir: output_dir("pipeline_ocr_text_recognition"),
        },
    )?;

    // 关键断言：OCR pipeline 分支必须通过 az-ocr 产出可消费文本文件。
    assert_eq!(run.algorithm_runs.len(), 1);
    let algorithm_run = &run.algorithm_runs[0];
    assert_eq!(
        algorithm_run.algorithm,
        ImageAlgorithmKind::OcrTextRecognition
    );
    assert_existing_file(&run.summary_file);
    for file in &algorithm_run.files {
        assert_existing_file(file);
    }
    assert!(
        algorithm_run.files.iter().any(|file| file
            .file_name()
            .is_some_and(|name| name == "recognized_text.txt")),
        "OCR pipeline 必须写出 recognized_text.txt"
    );
    Ok(())
}
