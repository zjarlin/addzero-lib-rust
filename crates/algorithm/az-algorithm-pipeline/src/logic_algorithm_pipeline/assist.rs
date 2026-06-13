//! 多算法 pipeline 执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};

use crate::logic_algorithm_pipeline::model::{
    ImageAlgorithmKind, ImageAlgorithmRunSummary, ImagePipelineOptions, ImagePipelineRun,
};

/// 对同一张图片叠加运行多个算法。
///
/// 具体算法仍然由各自独立 crate 承载；这里只负责调度和汇总输出路径。
///
/// # Errors
/// 当任一算法执行失败，或汇总文件写入失败时返回错误。
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印输入、输出的绝对路径"
)]
pub fn run_image_pipeline_from_path(
    image_path: impl AsRef<Path>,
    options: &ImagePipelineOptions,
) -> anyhow::Result<ImagePipelineRun> {
    let input_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| path_error(image_path.as_ref().to_path_buf(), source))?;
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;

    dbg!(&input_path);
    dbg!(&options.output_dir);

    let mut algorithm_runs = Vec::new();
    for algorithm in &options.algorithms {
        algorithm_runs.push(run_one_algorithm(*algorithm, &input_path, &options.output_dir)?);
    }

    let summary_file = options.output_dir.join("pipeline_results.json");
    let run = ImagePipelineRun {
        input_path,
        output_dir: options.output_dir.clone(),
        summary_file: summary_file.clone(),
        algorithm_runs,
    };
    let json = serde_json::to_string_pretty(&run)?;
    fs::write(&summary_file, json)
        .map_err(|source| path_error(summary_file.clone(), source))?;
    dbg!(&summary_file);
    Ok(run)
}

#[expect(
    clippy::dbg_macro,
    reason = "用户要求 pipeline 测试直接打印每个算法输出文件的绝对路径"
)]
fn run_one_algorithm(
    algorithm: ImageAlgorithmKind,
    input_path: &Path,
    root_output_dir: &Path,
) -> anyhow::Result<ImageAlgorithmRunSummary> {
    let output_dir = root_output_dir.join(algorithm.code());
    let files = match algorithm {
        ImageAlgorithmKind::FaceDetection => {
            let options = face_detection_options(output_dir.clone())?;
            let run =
                az_face_detection::logic_face_detection::assist::detect_faces_from_path_with_options(
                    input_path,
                    &options,
                )?;
            vec![
                run.files.source_input,
                run.files.model_input_preview,
                run.files.raw_outputs_json,
                run.files.detected_faces_json,
                run.files.detected_faces_image,
            ]
        }
        ImageAlgorithmKind::FaceRecognition => {
            let run = az_face_recognition::logic_face_recognition::assist::run_face_recognition_from_path_with_output(
                input_path,
                &output_dir,
            )?;
            vec![
                run.files.source_input,
                run.files.model_input_preview,
                run.files.raw_outputs_json,
                run.files.raw_output_review,
            ]
        }
        ImageAlgorithmKind::PersonDetection => {
            let run = az_person_detection::logic_person_detection::assist::run_person_detection_from_path_with_output(
                input_path,
                &output_dir,
            )?;
            vec![
                run.files.source_input,
                run.files.model_input_preview,
                run.files.raw_outputs_json,
                run.files.detected_persons_json,
                run.files.detected_persons_image,
            ]
        }
        ImageAlgorithmKind::OcrTextRecognition => {
            let run = az_ocr_text_recognition::logic_ocr_text_recognition::assist::run_ocr_text_recognition_from_path_with_output(
                input_path,
                output_dir.join("detection"),
                output_dir.join("recognition"),
            )?;
            vec![
                run.detection.files.source_input,
                run.detection.files.model_input_preview,
                run.detection.files.raw_outputs_json,
                run.detection.files.raw_output_review,
                run.recognition.files.source_input,
                run.recognition.files.model_input_preview,
                run.recognition.files.raw_outputs_json,
                run.recognition.files.raw_output_review,
            ]
        }
        ImageAlgorithmKind::FlameDetection => {
            let run = az_flame_detection::logic_flame_detection::assist::run_flame_detection_from_path_with_output(
                input_path,
                &output_dir,
            )?;
            vec![
                run.files.source_input,
                run.files.model_input_preview,
                run.files.raw_outputs_json,
                run.files.detected_flames_json,
                run.files.detected_flames_image,
            ]
        }
        ImageAlgorithmKind::SafetyHelmetDetection => {
            let run = az_safety_helmet_detection::logic_safety_helmet_detection::assist::run_safety_helmet_detection_from_path_with_output(
                input_path,
                &output_dir,
            )?;
            vec![
                run.files.source_input,
                run.files.model_input_preview,
                run.files.raw_outputs_json,
                run.files.raw_output_review,
            ]
        }
        ImageAlgorithmKind::VehicleDetection => {
            let run = az_vehicle_detection::logic_vehicle_detection::assist::run_vehicle_detection_from_path_with_output(
                input_path,
                &output_dir,
            )?;
            vec![
                run.files.source_input,
                run.files.model_input_preview,
                run.files.raw_outputs_json,
                run.files.detected_vehicles_json,
                run.files.detected_vehicles_image,
            ]
        }
        ImageAlgorithmKind::QrCodeRecognition => {
            let results = az_qr_code_recognition::logic_qr_code_recognition::assist::decode_qr_codes_from_path(input_path)?;
            fs::create_dir_all(&output_dir)
                .map_err(|source| path_error(output_dir.clone(), source))?;
            let output_file = output_dir.join("decoded_payloads.json");
            let json = serde_json::to_string_pretty(&results)?;
            fs::write(&output_file, json)
                .map_err(|source| path_error(output_file.clone(), source))?;
            vec![output_file]
        }
    };

    dbg!(algorithm.code());
    dbg!(&output_dir);
    dbg!(&files);

    Ok(ImageAlgorithmRunSummary {
        algorithm,
        code: algorithm.code().to_owned(),
        output_dir,
        files,
    })
}

fn face_detection_options(
    output_dir: PathBuf,
) -> anyhow::Result<az_face_detection::logic_face_detection::model::FaceDetectionOptions> {
    let workspace_root = std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .map_err(|source| path_error(PathBuf::from(env!("CARGO_MANIFEST_DIR")), source))?;
    let model_path = workspace_root
        .join("crates/algorithm/az-face-detection/resources/models")
        .join("face_detection_scrfd_500m.onnx");
    Ok(az_face_detection::logic_face_detection::model::FaceDetectionOptions {
        model_path,
        output_dir,
        score_threshold: 0.5,
        nms_threshold: 0.4,
    })
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow::anyhow!("filesystem error at `{}`: {source}", path.display())
}
