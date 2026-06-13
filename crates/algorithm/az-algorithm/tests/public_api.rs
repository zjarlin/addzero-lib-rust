use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use az_algorithm_pipeline::logic_algorithm_pipeline::assist::run_image_pipeline_from_path;
use az_algorithm_pipeline::logic_algorithm_pipeline::model::{
    ImageAlgorithmKind, ImagePipelineOptions, ImagePipelineRun,
};
use az_worker_hit_counting::logic_worker_hit_counting::assist::record_worker_hit_timeline_from_visual_observations;
use az_worker_hit_counting::logic_worker_hit_counting::model::{
    NormalizedBoundingBox, NormalizedPoint, VisualTargetKind, VisualTargetObservation,
    WorkerActionObservation, WorkerHitCountConfig,
};
use image::imageops::{self, FilterType};
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_hollow_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;

const FFMPEG_PATH: &str = "/opt/homebrew/bin/ffmpeg";
const FRAME_WIDTH: u32 = 960;
const FRAME_HEIGHT: u32 = 960;
const EXPECTED_QR_PAYLOAD: &str = "az-algorithm://真实二维码测试";

struct VideoMaterial {
    fixture_path: PathBuf,
}

struct FrameScenario {
    material_name: &'static str,
    frame_path: PathBuf,
    algorithms: Vec<ImageAlgorithmKind>,
}

#[test]
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印输入、视频和输出绝对路径"
)]
fn all_algorithms_should_run_from_one_edited_video() -> Result<(), Box<dyn Error>> {
    let workspace_root = workspace_root()?;
    let result_dir = workspace_root
        .join("target/az-algorithm-results")
        .join("all_algorithms_integration");
    recreate_dir(&result_dir)?;

    let source_frame_dir = result_dir.join("video_source_frames");
    let extracted_frame_dir = result_dir.join("video_extracted_frames");
    let annotation_frame_dir = result_dir.join("video_annotation_frames");
    fs::create_dir_all(&source_frame_dir)?;
    fs::create_dir_all(&extracted_frame_dir)?;
    fs::create_dir_all(&annotation_frame_dir)?;

    let materials = all_video_materials(&workspace_root)?;
    for material in &materials {
        dbg!(&material.fixture_path);
    }

    write_source_video_frames(&materials, &source_frame_dir)?;
    let video_path = result_dir.join("all_algorithm_materials.mkv");
    encode_lossless_video(&source_frame_dir, &video_path)?;
    extract_video_frames(&video_path, &extracted_frame_dir)?;

    dbg!(&video_path);
    dbg!(&source_frame_dir);
    dbg!(&extracted_frame_dir);

    let scenarios = frame_scenarios(&extracted_frame_dir);
    let mut image_runs = Vec::new();
    for scenario in &scenarios {
        let output_dir = result_dir
            .join("image_algorithm_outputs")
            .join(scenario.material_name);
        let run = run_image_pipeline_from_path(
            &scenario.frame_path,
            &ImagePipelineOptions {
                algorithms: scenario.algorithms.clone(),
                output_dir,
            },
        )?;
        assert_image_pipeline_outputs_exist(&run);
        image_runs.push(run);
    }

    let qr_payloads = decoded_qr_payloads(
        &image_runs,
        &result_dir
            .join("image_algorithm_outputs")
            .join("qr_code")
            .join("qr_code_recognition")
            .join("decoded_payloads.json"),
    )?;

    let worker_hit_timeline = record_worker_hit_timeline_from_visual_observations(
        &worker_hit_observations_from_video_frames(),
        WorkerHitCountConfig::default(),
    )?;
    let worker_hit_output = result_dir.join("worker_hit_timeline.json");
    fs::write(
        &worker_hit_output,
        serde_json::to_string_pretty(&worker_hit_timeline)?,
    )?;

    write_annotation_frames(
        &extracted_frame_dir,
        &annotation_frame_dir,
        &result_dir.join("image_algorithm_outputs"),
        &worker_hit_output,
    )?;
    let annotation_video_path = result_dir.join("all_algorithm_annotations.mkv");
    encode_lossless_video(&annotation_frame_dir, &annotation_video_path)?;

    dbg!(&worker_hit_output);
    dbg!(&annotation_frame_dir);
    dbg!(&annotation_video_path);

    let image_algorithm_count = image_runs
        .iter()
        .map(|run| run.algorithm_runs.len())
        .sum::<usize>();
    let worker = worker_hit_timeline
        .final_count
        .workers
        .iter()
        .find(|worker| worker.person_id == 1)
        .ok_or_else(|| std::io::Error::other("工人敲击计数必须返回 person_id=1 的轨迹"))?;

    assert!(
        video_path.is_file(),
        "剪辑后的视频必须存在：{}",
        video_path.display()
    );
    assert!(
        annotation_video_path.is_file(),
        "标注后的视频必须存在：{}",
        annotation_video_path.display()
    );
    assert_eq!(
        image_algorithm_count + 1,
        9,
        "8 个图像算法加 1 个工人敲击计数算法必须全部执行"
    );
    assert_eq!(
        qr_payloads,
        vec![EXPECTED_QR_PAYLOAD.to_owned()],
        "二维码算法必须从视频抽帧中解出真实 payload"
    );
    assert_eq!(
        worker.valid_hit_count, 2,
        "工人敲击计数必须基于视频帧序号统计出 2 次有效敲击"
    );
    assert_existing_file(&worker_hit_output);
    Ok(())
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
    )?)
}

fn recreate_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn all_video_materials(workspace_root: &Path) -> Result<Vec<VideoMaterial>, Box<dyn Error>> {
    let materials = [
        (
            "face",
            "crates/algorithm/az-face-detection/tests/fixtures/input/face.jpg",
        ),
        (
            "person_vehicle",
            "crates/algorithm/az-person-detection/tests/fixtures/input/person_vehicle.jpg",
        ),
        (
            "ocr_text",
            "crates/algorithm/az-ocr-text-recognition/tests/fixtures/input/ocr_text.jpg",
        ),
        (
            "flame",
            "crates/algorithm/az-flame-detection/tests/fixtures/input/flame.jpg",
        ),
        (
            "safety_helmet",
            "crates/algorithm/az-safety-helmet-detection/tests/fixtures/input/safety_helmet.jpg",
        ),
        (
            "qr_code",
            "crates/algorithm/az-qr-code-recognition/tests/fixtures/input/qr_code.png",
        ),
    ];

    materials
        .into_iter()
        .map(|(_name, relative_path)| {
            let fixture_path = fs::canonicalize(workspace_root.join(relative_path))?;
            Ok(VideoMaterial { fixture_path })
        })
        .collect()
}

fn write_source_video_frames(
    materials: &[VideoMaterial],
    source_frame_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    for (index, material) in materials.iter().enumerate() {
        let frame = normalized_video_frame(&material.fixture_path)?;
        frame.save(source_frame_dir.join(format!("frame_{index:03}.png")))?;
    }
    Ok(())
}

fn normalized_video_frame(image_path: &Path) -> Result<RgbImage, Box<dyn Error>> {
    let source = image::open(image_path)?.to_rgb8();
    let (fit_width, fit_height) = fit_size(source.width(), source.height());
    let resized = imageops::resize(&source, fit_width, fit_height, FilterType::Nearest);
    let mut canvas = RgbImage::from_pixel(FRAME_WIDTH, FRAME_HEIGHT, Rgb([255, 255, 255]));
    let x = (FRAME_WIDTH - fit_width) / 2;
    let y = (FRAME_HEIGHT - fit_height) / 2;
    imageops::overlay(&mut canvas, &resized, i64::from(x), i64::from(y));
    Ok(canvas)
}

fn fit_size(width: u32, height: u32) -> (u32, u32) {
    if width <= FRAME_WIDTH && height <= FRAME_HEIGHT {
        return (width, height);
    }

    let width_scaled_by_height = u64::from(width) * u64::from(FRAME_HEIGHT);
    let height_scaled_by_width = u64::from(height) * u64::from(FRAME_WIDTH);
    if width_scaled_by_height > height_scaled_by_width {
        let fit_height = (u64::from(height) * u64::from(FRAME_WIDTH) / u64::from(width)) as u32;
        (FRAME_WIDTH, fit_height.max(1))
    } else {
        let fit_width = (u64::from(width) * u64::from(FRAME_HEIGHT) / u64::from(height)) as u32;
        (fit_width.max(1), FRAME_HEIGHT)
    }
}

fn encode_lossless_video(source_frame_dir: &Path, video_path: &Path) -> Result<(), Box<dyn Error>> {
    run_ffmpeg(&[
        OsString::from("-y"),
        OsString::from("-framerate"),
        OsString::from("1"),
        OsString::from("-start_number"),
        OsString::from("0"),
        OsString::from("-i"),
        source_frame_dir.join("frame_%03d.png").into_os_string(),
        OsString::from("-c:v"),
        OsString::from("ffv1"),
        OsString::from("-level"),
        OsString::from("3"),
        OsString::from("-pix_fmt"),
        OsString::from("rgb24"),
        video_path.as_os_str().to_os_string(),
    ])
}

fn extract_video_frames(
    video_path: &Path,
    extracted_frame_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    run_ffmpeg(&[
        OsString::from("-y"),
        OsString::from("-i"),
        video_path.as_os_str().to_os_string(),
        OsString::from("-start_number"),
        OsString::from("0"),
        extracted_frame_dir.join("frame_%03d.png").into_os_string(),
    ])
}

fn run_ffmpeg(args: &[OsString]) -> Result<(), Box<dyn Error>> {
    let ffmpeg = Path::new(FFMPEG_PATH);
    if !ffmpeg.is_file() {
        let message = format!("真实视频集成测试需要 ffmpeg：{}", ffmpeg.display());

        return Err(message.into());
    }

    let output = Command::new(ffmpeg).args(args).output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "ffmpeg 执行失败，状态：{}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .into())
}

fn frame_scenarios(extracted_frame_dir: &Path) -> Vec<FrameScenario> {
    vec![
        FrameScenario {
            material_name: "face",
            frame_path: extracted_frame_dir.join("frame_000.png"),
            algorithms: vec![
                ImageAlgorithmKind::FaceDetection,
                ImageAlgorithmKind::FaceRecognition,
            ],
        },
        FrameScenario {
            material_name: "person_vehicle",
            frame_path: extracted_frame_dir.join("frame_001.png"),
            algorithms: vec![
                ImageAlgorithmKind::PersonDetection,
                ImageAlgorithmKind::VehicleDetection,
            ],
        },
        FrameScenario {
            material_name: "ocr_text",
            frame_path: extracted_frame_dir.join("frame_002.png"),
            algorithms: vec![ImageAlgorithmKind::OcrTextRecognition],
        },
        FrameScenario {
            material_name: "flame",
            frame_path: extracted_frame_dir.join("frame_003.png"),
            algorithms: vec![ImageAlgorithmKind::FlameDetection],
        },
        FrameScenario {
            material_name: "safety_helmet",
            frame_path: extracted_frame_dir.join("frame_004.png"),
            algorithms: vec![ImageAlgorithmKind::SafetyHelmetDetection],
        },
        FrameScenario {
            material_name: "qr_code",
            frame_path: extracted_frame_dir.join("frame_005.png"),
            algorithms: vec![ImageAlgorithmKind::QrCodeRecognition],
        },
    ]
}

fn assert_image_pipeline_outputs_exist(run: &ImagePipelineRun) {
    assert_existing_file(&run.input_path);
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
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

fn decoded_qr_payloads(
    image_runs: &[ImagePipelineRun],
    payload_file: &Path,
) -> Result<Vec<String>, Box<dyn Error>> {
    assert!(
        image_runs
            .iter()
            .flat_map(|run| &run.algorithm_runs)
            .any(|run| run.algorithm == ImageAlgorithmKind::QrCodeRecognition),
        "集成测试必须执行二维码识别算法"
    );
    let decoded: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(payload_file)?)?;
    Ok(decoded
        .into_iter()
        .filter_map(|item| {
            item.get("payload")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
        })
        .collect())
}

fn write_annotation_frames(
    extracted_frame_dir: &Path,
    annotation_frame_dir: &Path,
    image_output_dir: &Path,
    worker_hit_output: &Path,
) -> Result<(), Box<dyn Error>> {
    for index in 0..6 {
        let frame_path = extracted_frame_dir.join(format!("frame_{index:03}.png"));
        let mut frame = image::open(frame_path)?.to_rgb8();
        match index {
            0 => draw_face_detection_boxes(
                &mut frame,
                &image_output_dir
                    .join("face")
                    .join("face_detection")
                    .join("detected_faces.json"),
            )?,
            1 => {
                draw_person_detection_boxes(
                    &mut frame,
                    &image_output_dir
                        .join("person_vehicle")
                        .join("person_detection")
                        .join("detected_persons.json"),
                )?;
                draw_worker_hit_counting_boxes(&mut frame, worker_hit_output)?;
            }
            5 => draw_qr_code_bounds(
                &mut frame,
                &image_output_dir
                    .join("qr_code")
                    .join("qr_code_recognition")
                    .join("decoded_payloads.json"),
            )?,
            _ => {}
        }
        frame.save(annotation_frame_dir.join(format!("frame_{index:03}.png")))?;
    }
    Ok(())
}

fn draw_person_detection_boxes(
    frame: &mut RgbImage,
    detected_persons_file: &Path,
) -> Result<(), Box<dyn Error>> {
    let persons: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(detected_persons_file)?)?;
    for person in persons {
        let x_min = json_f32(&person, "x_min")?;
        let y_min = json_f32(&person, "y_min")?;
        let x_max = json_f32(&person, "x_max")?;
        let y_max = json_f32(&person, "y_max")?;
        draw_rect(
            frame,
            x_min,
            y_min,
            x_max - x_min,
            y_max - y_min,
            Rgb([0, 220, 80]),
        );
    }
    Ok(())
}

fn draw_face_detection_boxes(
    frame: &mut RgbImage,
    detected_faces_file: &Path,
) -> Result<(), Box<dyn Error>> {
    let faces: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(detected_faces_file)?)?;
    for face in faces {
        let x_min = json_f32(&face, "x_min")?;
        let y_min = json_f32(&face, "y_min")?;
        let x_max = json_f32(&face, "x_max")?;
        let y_max = json_f32(&face, "y_max")?;
        draw_rect(
            frame,
            x_min,
            y_min,
            x_max - x_min,
            y_max - y_min,
            Rgb([255, 0, 0]),
        );
    }
    Ok(())
}

fn draw_qr_code_bounds(
    frame: &mut RgbImage,
    decoded_payloads_file: &Path,
) -> Result<(), Box<dyn Error>> {
    let payloads: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(decoded_payloads_file)?)?;
    for payload in payloads {
        let Some(bounds) = payload.get("bounds").and_then(serde_json::Value::as_array) else {
            continue;
        };
        for index in 0..bounds.len() {
            let from = &bounds[index];
            let to = &bounds[(index + 1) % bounds.len()];
            draw_line_segment_mut(
                frame,
                (json_f32(from, "x")?, json_f32(from, "y")?),
                (json_f32(to, "x")?, json_f32(to, "y")?),
                Rgb([0, 180, 255]),
            );
        }
    }
    Ok(())
}

fn draw_worker_hit_counting_boxes(
    frame: &mut RgbImage,
    worker_hit_output: &Path,
) -> Result<(), Box<dyn Error>> {
    let worker_hit_count: serde_json::Value =
        serde_json::from_slice(&fs::read(worker_hit_output)?)?;
    let Some(workers) = worker_hit_count
        .get("final_count")
        .and_then(|final_count| final_count.get("workers"))
        .and_then(serde_json::Value::as_array)
    else {
        return Ok(());
    };
    for worker in workers {
        draw_normalized_box(frame, &worker["last_person_box"], Rgb([0, 255, 0]))?;
        for hit in worker
            .get("valid_hits")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            draw_normalized_box(frame, &hit["person_box"], Rgb([255, 215, 0]))?;
        }
        for candidate in worker
            .get("invalid_candidates")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            draw_normalized_box(frame, &candidate["person_box"], Rgb([255, 120, 0]))?;
        }
    }
    Ok(())
}

fn draw_normalized_box(
    frame: &mut RgbImage,
    json_box: &serde_json::Value,
    color: Rgb<u8>,
) -> Result<(), Box<dyn Error>> {
    let x = json_f32(json_box, "x")? * frame.width() as f32;
    let y = json_f32(json_box, "y")? * frame.height() as f32;
    let width = json_f32(json_box, "width")? * frame.width() as f32;
    let height = json_f32(json_box, "height")? * frame.height() as f32;
    draw_rect(frame, x, y, width, height, color);
    Ok(())
}

fn draw_rect(frame: &mut RgbImage, x: f32, y: f32, width: f32, height: f32, color: Rgb<u8>) {
    let rect = Rect::at(x.round() as i32, y.round() as i32).of_size(
        width.max(1.0).round() as u32,
        height.max(1.0).round() as u32,
    );
    draw_hollow_rect_mut(frame, rect, color);
}

fn json_f32(value: &serde_json::Value, field: &str) -> Result<f32, Box<dyn Error>> {
    value
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .ok_or_else(|| std::io::Error::other(format!("JSON 缺少数值字段：{field}")).into())
}

fn worker_hit_observations_from_video_frames() -> Vec<WorkerActionObservation> {
    vec![
        worker_observation(1, 0, 0, VisualTargetKind::HangingMetalPanel, 0.90),
        worker_observation(1, 1, 260, VisualTargetKind::ConveyorBody, 0.90),
        worker_observation(1, 2, 520, VisualTargetKind::HangingMetalPanel, 0.90),
    ]
}

fn worker_observation(
    person_id: u64,
    frame_index: u64,
    timestamp_ms: u64,
    target_kind: VisualTargetKind,
    target_response_score: f32,
) -> WorkerActionObservation {
    WorkerActionObservation {
        person_id,
        frame_index,
        timestamp_ms,
        person_box: NormalizedBoundingBox {
            x: 0.18,
            y: 0.20,
            width: 0.22,
            height: 0.48,
        },
        strike_score: 0.90,
        contact_score: 0.90,
        contact_point: Some(NormalizedPoint { x: 0.52, y: 0.32 }),
        contacted_target: Some(VisualTargetObservation {
            target_id: 100,
            kind: target_kind,
            target_box: NormalizedBoundingBox {
                x: 0.40,
                y: 0.20,
                width: 0.30,
                height: 0.30,
            },
            containment_score: 0.95,
        }),
        target_response_score,
    }
}
