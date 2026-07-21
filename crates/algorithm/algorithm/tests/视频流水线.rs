use std::path::{Path, PathBuf};

use az_algorithm::components::face_detection::assist::FaceDetectionRunner;
use az_algorithm::components::face_detection::model::{
    FaceDetectionOptions, MODEL_CODE as FACE_DETECTION_MODEL_CODE,
};
use az_algorithm::components::flame_detection::model::{
    ALGORITHM_CODE as FLAME_DETECTION_CODE, DEFAULT_NMS_THRESHOLD as FLAME_DEFAULT_NMS_THRESHOLD,
    DEFAULT_SCORE_THRESHOLD as FLAME_DEFAULT_SCORE_THRESHOLD, FLAME_DETECTION_FIRE_SMOKE_YOLOV8N,
    FlameDetectionOptions,
};
use az_algorithm::components::person_detection::assist::PersonDetectionRunner;
use az_algorithm::components::person_detection::model::{
    ALGORITHM_CODE as PERSON_DETECTION_CODE, DEFAULT_SCORE_THRESHOLD, PersonDetectionModelKind,
    PersonDetectionOptions,
};
use az_algorithm::components::qr_code_recognition::model::ALGORITHM_CODE as QR_CODE_RECOGNITION_CODE;
use az_algorithm::components::safety_helmet_detection::model::{
    ALGORITHM_CODE as SAFETY_HELMET_DETECTION_CODE,
    DEFAULT_NMS_THRESHOLD as SAFETY_HELMET_DEFAULT_NMS_THRESHOLD,
    DEFAULT_SCORE_THRESHOLD as SAFETY_HELMET_DEFAULT_SCORE_THRESHOLD, SafetyHelmetDetectionOptions,
};
use az_algorithm::components::vehicle_detection::model::{
    ALGORITHM_CODE as VEHICLE_DETECTION_CODE, VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1,
};
use az_algorithm::video_pipeline::assist::flame_video_algorithm::FlameVideoAlgorithm;
use az_algorithm::video_pipeline::assist::onnx_raw_image_video_algorithm::OnnxRawImageVideoAlgorithm;
use az_algorithm::video_pipeline::assist::qr_code_video_algorithm::QrCodeVideoAlgorithm;
use az_algorithm::video_pipeline::assist::safety_helmet_video_algorithm::SafetyHelmetVideoAlgorithm;
use az_algorithm::video_pipeline::model::{
    VideoAlgorithmBinding, VideoAlgorithmEvent, VideoAlgorithmFrameResult, VideoAlgorithmSchedule,
    VideoBoundingBox, VideoDetection, VideoFrame, VideoFrameAlgorithm, VideoPipelineOptions,
};
use az_algorithm::video_pipeline::pipeline::run_video_frame_pipeline;
use image::{Rgb, RgbImage};
use serde_json::json;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn output_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join(name)
}

fn person_fixture_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/tests/fixtures/person_detection/input")
            .join("person_vehicle.jpg"),
    )
    .expect("人员检测测试图片必须存在")
}

fn person_model_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/resources/person_detection/models")
            .join("coco_ssd_mobilenet_v1_10.onnx"),
    )
    .expect("人员检测模型必须存在")
}

fn face_fixture_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/tests/fixtures/face_detection/input")
            .join("face.jpg"),
    )
    .expect("人脸检测测试图片必须存在")
}

fn face_model_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/resources/face_detection/models")
            .join("face_detection_scrfd_500m.onnx"),
    )
    .expect("人脸检测模型必须存在")
}

fn safety_helmet_fixture_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/tests/fixtures/safety_helmet_detection/input")
            .join("safety_helmet.jpg"),
    )
    .expect("安全帽检测测试图片必须存在")
}

fn safety_helmet_model_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/resources/safety_helmet_detection/models")
            .join("safety_helmet_detection_ppe_yolo11s.onnx"),
    )
    .expect("安全帽检测模型必须存在")
}

fn vehicle_fixture_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/tests/fixtures/vehicle_detection/input")
            .join("person_vehicle.jpg"),
    )
    .expect("车辆检测测试图片必须存在")
}

fn vehicle_model_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/resources/vehicle_detection/models")
            .join("coco_ssd_mobilenet_v1_10.onnx"),
    )
    .expect("车辆检测模型必须存在")
}

fn flame_fixture_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/tests/fixtures/flame_detection/input")
            .join("flame.jpg"),
    )
    .expect("火焰检测测试图片必须存在")
}

fn flame_model_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/resources/flame_detection/models")
            .join(FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.local_file),
    )
    .expect("火焰检测模型必须存在")
}

fn qr_code_fixture_path() -> PathBuf {
    std::fs::canonicalize(
        workspace_root()
            .join("crates/algorithm/algorithm/tests/fixtures/qr_code_recognition/input")
            .join("qr_code.png"),
    )
    .expect("二维码测试图片必须存在")
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

fn generated_frames(frame_count: u64, source_fps: f32) -> Vec<VideoFrame> {
    (0..frame_count)
        .map(|frame_index| {
            let mut rgb = RgbImage::new(16, 12);
            for (x, y, pixel) in rgb.enumerate_pixels_mut() {
                *pixel = Rgb([(frame_index as u8).wrapping_mul(11), x as u8, y as u8]);
            }
            VideoFrame {
                frame_index,
                timestamp_ms: ((frame_index as f32 / source_fps) * 1_000.0).round() as u64,
                width: rgb.width(),
                height: rgb.height(),
                rgb,
            }
        })
        .collect()
}

fn frames_from_person_fixture(frame_count: u64, source_fps: f32) -> Vec<VideoFrame> {
    let rgb = image::open(person_fixture_path())
        .expect("人员检测测试图片必须能解码")
        .to_rgb8();
    (0..frame_count)
        .map(|frame_index| VideoFrame {
            frame_index,
            timestamp_ms: ((frame_index as f32 / source_fps) * 1_000.0).round() as u64,
            width: rgb.width(),
            height: rgb.height(),
            rgb: rgb.clone(),
        })
        .collect()
}

fn frames_from_face_fixture(frame_count: u64, source_fps: f32) -> Vec<VideoFrame> {
    let rgb = image::open(face_fixture_path())
        .expect("人脸检测测试图片必须能解码")
        .to_rgb8();
    (0..frame_count)
        .map(|frame_index| VideoFrame {
            frame_index,
            timestamp_ms: ((frame_index as f32 / source_fps) * 1_000.0).round() as u64,
            width: rgb.width(),
            height: rgb.height(),
            rgb: rgb.clone(),
        })
        .collect()
}

fn frames_from_safety_helmet_fixture(frame_count: u64, source_fps: f32) -> Vec<VideoFrame> {
    let rgb = image::open(safety_helmet_fixture_path())
        .expect("安全帽检测测试图片必须能解码")
        .to_rgb8();
    (0..frame_count)
        .map(|frame_index| VideoFrame {
            frame_index,
            timestamp_ms: ((frame_index as f32 / source_fps) * 1_000.0).round() as u64,
            width: rgb.width(),
            height: rgb.height(),
            rgb: rgb.clone(),
        })
        .collect()
}

fn frames_from_composite_algorithm_fixture(frame_count: u64, source_fps: f32) -> Vec<VideoFrame> {
    let width = 1920;
    let height = 1280;
    let mut canvas = RgbImage::from_pixel(width, height, Rgb([245, 245, 245]));

    paste_resized(
        &mut canvas,
        image::open(face_fixture_path()).expect("人脸测试图片必须能解码"),
        0,
        0,
        480,
        480,
    );
    paste_resized(
        &mut canvas,
        image::open(person_fixture_path()).expect("人员测试图片必须能解码"),
        480,
        0,
        360,
        480,
    );
    paste_resized(
        &mut canvas,
        image::open(safety_helmet_fixture_path()).expect("安全帽测试图片必须能解码"),
        840,
        0,
        540,
        360,
    );
    paste_resized(
        &mut canvas,
        image::open(vehicle_fixture_path()).expect("车辆测试图片必须能解码"),
        1380,
        0,
        360,
        480,
    );
    paste_resized(
        &mut canvas,
        image::open(qr_code_fixture_path()).expect("二维码测试图片必须能解码"),
        0,
        520,
        333,
        333,
    );
    paste_resized(
        &mut canvas,
        image::open(flame_fixture_path()).expect("火焰测试图片必须能解码"),
        360,
        520,
        360,
        520,
    );
    (0..frame_count)
        .map(|frame_index| VideoFrame {
            frame_index,
            timestamp_ms: ((frame_index as f32 / source_fps) * 1_000.0).round() as u64,
            width,
            height,
            rgb: canvas.clone(),
        })
        .collect()
}

fn paste_resized(
    canvas: &mut RgbImage,
    image: image::DynamicImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    let resized = image
        .resize_exact(width, height, image::imageops::FilterType::Triangle)
        .to_rgb8();
    for (dx, dy, pixel) in resized.enumerate_pixels() {
        let target_x = x + dx;
        let target_y = y + dy;
        if target_x < canvas.width() && target_y < canvas.height() {
            canvas.put_pixel(target_x, target_y, *pixel);
        }
    }
}

#[derive(Debug)]
struct RecordingAlgorithm {
    code: &'static str,
    processed_frames: Vec<u64>,
}

impl RecordingAlgorithm {
    fn new(code: &'static str) -> Self {
        Self {
            code,
            processed_frames: Vec::new(),
        }
    }
}

impl VideoFrameAlgorithm for RecordingAlgorithm {
    fn code(&self) -> &'static str {
        self.code
    }

    fn process_frame(&mut self, frame: &VideoFrame) -> anyhow::Result<VideoAlgorithmFrameResult> {
        self.processed_frames.push(frame.frame_index);
        Ok(VideoAlgorithmFrameResult {
            algorithm_code: self.code.to_owned(),
            frame_index: frame.frame_index,
            timestamp_ms: frame.timestamp_ms,
            detections: vec![VideoDetection {
                label: format!("{}_sample_detection", self.code),
                confidence: 0.75,
                bounding_box: Some(VideoBoundingBox {
                    x_min: 1.0,
                    y_min: 2.0,
                    x_max: 8.0,
                    y_max: 9.0,
                }),
                extra: json!({
                    "说明": "这是管线调度测试的结构化样例，不是模型识别结果"
                }),
            }],
            events: vec![VideoAlgorithmEvent {
                event_code: format!("{}_processed", self.code),
                score: 1.0,
                message: "帧已被该算法实例处理".to_owned(),
                extra: json!({
                    "frame_width": frame.width,
                    "frame_height": frame.height
                }),
            }],
            raw_json: json!({
                "source": "in_memory_pipeline_test"
            }),
        })
    }
}

struct RealPersonDetectionVideoAlgorithm {
    runner: PersonDetectionRunner,
    output_dir: PathBuf,
}

impl RealPersonDetectionVideoAlgorithm {
    fn new(output_dir: PathBuf) -> anyhow::Result<Self> {
        let runner = PersonDetectionRunner::new(PersonDetectionOptions {
            model_path: person_model_path(),
            model_kind: PersonDetectionModelKind::CocoSsdMobileNetV1,
            output_dir: output_dir.join("unused_default_output"),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
        })
        .map_err(|source| anyhow::anyhow!(source.to_string(),))?;
        Ok(Self { runner, output_dir })
    }
}

impl VideoFrameAlgorithm for RealPersonDetectionVideoAlgorithm {
    fn code(&self) -> &'static str {
        PERSON_DETECTION_CODE
    }

    fn process_frame(&mut self, frame: &VideoFrame) -> anyhow::Result<VideoAlgorithmFrameResult> {
        let frame_output_dir = self
            .output_dir
            .join("person_detection_frames")
            .join(format!("frame_{:05}", frame.frame_index));
        let run = self
            .runner
            .detect_rgb_image_with_output_dir(frame.rgb.clone(), frame_output_dir)
            .map_err(|source| anyhow::anyhow!(source.to_string(),))?;

        Ok(VideoAlgorithmFrameResult {
            algorithm_code: PERSON_DETECTION_CODE.to_owned(),
            frame_index: frame.frame_index,
            timestamp_ms: frame.timestamp_ms,
            detections: run
                .persons
                .iter()
                .map(|person| VideoDetection {
                    label: "person".to_owned(),
                    confidence: person.confidence,
                    bounding_box: Some(VideoBoundingBox {
                        x_min: person.x_min,
                        y_min: person.y_min,
                        x_max: person.x_max,
                        y_max: person.y_max,
                    }),
                    extra: json!({
                        "class_id": person.class_id,
                    }),
                })
                .collect(),
            events: Vec::new(),
            raw_json: json!({
                "model_path": run.model_path,
                "source_input": run.files.source_input,
                "detected_persons_json": run.files.detected_persons_json,
                "detected_persons_image": run.files.detected_persons_image,
                "raw_output_count": run.raw_outputs.len(),
            }),
        })
    }
}

struct RealFaceDetectionVideoAlgorithm {
    runner: FaceDetectionRunner,
    output_dir: PathBuf,
}

impl RealFaceDetectionVideoAlgorithm {
    fn new(output_dir: PathBuf) -> anyhow::Result<Self> {
        let runner = FaceDetectionRunner::new(FaceDetectionOptions {
            model_path: face_model_path(),
            output_dir: output_dir.join("unused_default_output"),
            score_threshold: 0.5,
            nms_threshold: 0.4,
        })
        .map_err(|source| anyhow::anyhow!(source.to_string(),))?;
        Ok(Self { runner, output_dir })
    }
}

impl VideoFrameAlgorithm for RealFaceDetectionVideoAlgorithm {
    fn code(&self) -> &'static str {
        "face_detection"
    }

    fn process_frame(&mut self, frame: &VideoFrame) -> anyhow::Result<VideoAlgorithmFrameResult> {
        let frame_output_dir = self
            .output_dir
            .join("face_detection_frames")
            .join(format!("frame_{:05}", frame.frame_index));
        let run = self
            .runner
            .detect_rgb_image_with_output_dir(frame.rgb.clone(), frame_output_dir)
            .map_err(|source| anyhow::anyhow!(source.to_string(),))?;

        Ok(VideoAlgorithmFrameResult {
            algorithm_code: self.code().to_owned(),
            frame_index: frame.frame_index,
            timestamp_ms: frame.timestamp_ms,
            detections: run
                .faces
                .iter()
                .map(|face| VideoDetection {
                    label: "face".to_owned(),
                    confidence: face.confidence,
                    bounding_box: Some(VideoBoundingBox {
                        x_min: face.x_min,
                        y_min: face.y_min,
                        x_max: face.x_max,
                        y_max: face.y_max,
                    }),
                    extra: json!({}),
                })
                .collect(),
            events: Vec::new(),
            raw_json: json!({
                "model_code": FACE_DETECTION_MODEL_CODE,
                "model_path": run.model_path,
                "source_input": run.files.source_input,
                "detected_faces_json": run.files.detected_faces_json,
                "detected_faces_image": run.files.detected_faces_image,
                "raw_output_count": run.raw_outputs.len(),
            }),
        })
    }
}
// 视频流水线应在同一帧上调度多个实时算法
#[test]
#[expect(clippy::dbg_macro, reason = "用户要求测试直接打印输入、输出的绝对路径")]
fn video_pipeline_should_schedule_multiple_realtime_algorithms_on_same_frames() -> anyhow::Result<()>
{
    // 这个测试只验证视频实时 pipeline 的调度和落盘行为。
    // 这里的 RecordingAlgorithm 是可观测测试算法，不是人脸、安全帽或抽烟检测模型。
    //
    // 真实接入方式：
    // - 人脸检测 crate 包装成一个 VideoFrameAlgorithm，按 10fps 跑。
    // - 安全帽检测 crate 包装成另一个 VideoFrameAlgorithm，按 5fps 跑。
    // - 抽烟检测 crate 未来独立实现后同样挂进 bindings。
    // - 视频解码层只产生一份 VideoFrame，多个算法共享同一帧。
    let output_dir = output_dir("video_pipeline_schedule");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("清理旧视频 pipeline 测试输出目录必须成功");
    }
    let source_fps = 30.0;
    let frames = generated_frames(10, source_fps);
    let first_frame = frames[0].metadata();
    dbg!(&first_frame);
    dbg!(&output_dir);

    let mut every_frame_algorithm = RecordingAlgorithm::new("face_detection");
    let mut every_three_frames_algorithm = RecordingAlgorithm::new("safety_helmet_detection");
    let mut bindings = [
        VideoAlgorithmBinding {
            algorithm: &mut every_frame_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
        VideoAlgorithmBinding {
            algorithm: &mut every_three_frames_algorithm,
            schedule: VideoAlgorithmSchedule::EveryNFrames { n: 3 },
        },
    ];

    let run = run_video_frame_pipeline(
        frames,
        &mut bindings,
        &VideoPipelineOptions {
            output_dir: output_dir.clone(),
            source_fps,
        },
    )?;

    dbg!(&run.files.frame_results_jsonl);
    dbg!(&run.files.summary_json);

    assert_existing_file(&run.files.frame_results_jsonl);
    assert_existing_file(&run.files.summary_json);
    // 关键断言：同一条 10 帧输入流上，一个算法每帧执行，另一个算法按 0、3、6、9 帧执行。
    assert_eq!(
        (
            run.total_input_frames,
            &run.algorithm_runs[0].processed_frame_indices,
            &run.algorithm_runs[1].processed_frame_indices,
        ),
        (10, &vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], &vec![0, 3, 6, 9])
    );
    Ok(())
}
// 视频流水线应在不重新加载模型的情况下运行真人检测运行器
#[test]
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印真实输入、模型、输出绝对路径"
)]
fn video_pipeline_should_run_real_person_detection_runner_without_reloading_model()
-> anyhow::Result<()> {
    // 这个测试是真实 ONNX 推理测试：
    // - 输入帧来自 az-person-detection 的真实人员测试图片。
    // - PersonDetectionRunner 在测试开始时只构造一次，内部 ONNX Session 复用到多帧。
    // - pipeline 负责把 3 帧视频流送给同一个 runner，并写出 JSONL/summary。
    let output_dir = output_dir("video_pipeline_real_person_detection");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("清理旧真实视频 pipeline 输出目录必须成功");
    }
    let source_fps = 30.0;
    let frames = frames_from_person_fixture(3, source_fps);
    dbg!(&person_fixture_path());
    dbg!(&person_model_path());
    dbg!(&output_dir);

    let mut person_algorithm = RealPersonDetectionVideoAlgorithm::new(output_dir.clone())?;
    let mut bindings = [VideoAlgorithmBinding {
        algorithm: &mut person_algorithm,
        schedule: VideoAlgorithmSchedule::EveryFrame,
    }];

    let run = run_video_frame_pipeline(
        frames,
        &mut bindings,
        &VideoPipelineOptions {
            output_dir: output_dir.clone(),
            source_fps,
        },
    )?;

    dbg!(&run.files.frame_results_jsonl);
    dbg!(&run.files.summary_json);
    assert_existing_file(&run.files.frame_results_jsonl);
    assert_existing_file(&run.files.summary_json);
    // 关键断言：真实人员检测模型应在 3 帧上都执行，并至少返回一个人员框。
    assert!(
        run.frame_results
            .iter()
            .all(|result| !result.detections.is_empty()),
        "真实人员检测 runner 必须在每帧返回人员框"
    );
    Ok(())
}
// 视频流水线应在不重新加载模型的情况下运行真实人脸检测运行器
#[test]
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印真实输入、模型、输出绝对路径"
)]
fn video_pipeline_should_run_real_face_detection_runner_without_reloading_model()
-> anyhow::Result<()> {
    // 这个测试是真实 SCRFD 人脸检测推理测试：
    // - 输入帧来自 az-face-detection 的真实人脸测试图片。
    // - FaceDetectionRunner 在测试开始时只构造一次，内部 ONNX Session 复用到多帧。
    let output_dir = output_dir("video_pipeline_real_face_detection");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("清理旧真实人脸视频 pipeline 输出目录必须成功");
    }
    let source_fps = 30.0;
    let frames = frames_from_face_fixture(3, source_fps);
    dbg!(&face_fixture_path());
    dbg!(&face_model_path());
    dbg!(&output_dir);

    let mut face_algorithm = RealFaceDetectionVideoAlgorithm::new(output_dir.clone())?;
    let mut bindings = [VideoAlgorithmBinding {
        algorithm: &mut face_algorithm,
        schedule: VideoAlgorithmSchedule::EveryFrame,
    }];

    let run = run_video_frame_pipeline(
        frames,
        &mut bindings,
        &VideoPipelineOptions {
            output_dir: output_dir.clone(),
            source_fps,
        },
    )?;

    dbg!(&run.files.frame_results_jsonl);
    dbg!(&run.files.summary_json);
    assert_existing_file(&run.files.frame_results_jsonl);
    assert_existing_file(&run.files.summary_json);
    // 关键断言：真实人脸检测模型应在 3 帧上都执行，并至少返回一个人脸框。
    assert!(
        run.frame_results
            .iter()
            .all(|result| !result.detections.is_empty()),
        "真实人脸检测 runner 必须在每帧返回人脸框"
    );
    Ok(())
}
// 视频流水线应将真实人物和人脸检测叠加在同一帧上
#[test]
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印真实输入、模型、输出绝对路径"
)]
fn video_pipeline_should_stack_real_person_and_face_detection_on_same_frames() -> anyhow::Result<()>
{
    // 这个测试验证“同一条视频帧流叠加多个真实算法”：
    // - 视频帧只构造一次，两个 runner 共享同一批 VideoFrame。
    // - person_detection 每 2 帧跑一次，face_detection 每帧跑一次。
    // - 人员模型在该人脸图上可能没有人员框，所以只断言它真实执行并写出 raw 输出。
    let output_dir = output_dir("video_pipeline_real_person_face_stack");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("清理旧真实叠加 pipeline 输出目录必须成功");
    }
    let source_fps = 30.0;
    let frames = frames_from_face_fixture(4, source_fps);
    dbg!(&face_fixture_path());
    dbg!(&face_model_path());
    dbg!(&person_model_path());
    dbg!(&output_dir);

    let mut person_algorithm = RealPersonDetectionVideoAlgorithm::new(output_dir.clone())?;
    let mut face_algorithm = RealFaceDetectionVideoAlgorithm::new(output_dir.clone())?;
    let mut bindings = [
        VideoAlgorithmBinding {
            algorithm: &mut person_algorithm,
            schedule: VideoAlgorithmSchedule::EveryNFrames { n: 2 },
        },
        VideoAlgorithmBinding {
            algorithm: &mut face_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
    ];

    let run = run_video_frame_pipeline(
        frames,
        &mut bindings,
        &VideoPipelineOptions {
            output_dir: output_dir.clone(),
            source_fps,
        },
    )?;

    dbg!(&run.files.frame_results_jsonl);
    dbg!(&run.files.summary_json);
    assert_existing_file(&run.files.frame_results_jsonl);
    assert_existing_file(&run.files.summary_json);
    // 关键断言：同一帧流里，人员检测按 0、2 帧执行，人脸检测按 0、1、2、3 帧执行。
    assert_eq!(
        (
            &run.algorithm_runs[0].processed_frame_indices,
            &run.algorithm_runs[1].processed_frame_indices,
        ),
        (&vec![0, 2], &vec![0, 1, 2, 3])
    );
    assert!(
        run.frame_results
            .iter()
            .filter(|result| result.algorithm_code == "face_detection")
            .all(|result| !result.detections.is_empty()),
        "真实人脸检测在每次执行时必须返回人脸框"
    );
    assert!(
        run.frame_results
            .iter()
            .filter(|result| result.algorithm_code == PERSON_DETECTION_CODE)
            .all(|result| result.raw_json["raw_output_count"].as_u64().unwrap_or(0) > 0),
        "真实人员检测虽然可能无人员框，但必须真实执行 ONNX 并写出输出"
    );
    Ok(())
}
// 视频管道应该运行真正的安全帽检测运行器
#[test]
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印真实输入、模型、输出绝对路径"
)]
fn video_pipeline_should_run_real_safety_helmet_detection_runner() -> anyhow::Result<()> {
    // 这个测试验证安全帽模型可以进入实时视频管线并执行 YOLO 后处理：
    // - SafetyHelmetVideoAlgorithm 只加载一次 ONNX Session。
    // - 每帧写出 detected_safety_helmets.json 和标注图。
    let output_dir = output_dir("video_pipeline_real_safety_helmet_detection");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("清理旧真实安全帽 pipeline 输出目录必须成功");
    }
    let source_fps = 30.0;
    let frames = frames_from_safety_helmet_fixture(3, source_fps);
    dbg!(&safety_helmet_fixture_path());
    dbg!(&safety_helmet_model_path());
    dbg!(&output_dir);

    let mut helmet_algorithm = SafetyHelmetVideoAlgorithm::new(
        SafetyHelmetDetectionOptions {
            model_path: safety_helmet_model_path(),
            output_dir: output_dir.join("unused_default_output"),
            score_threshold: SAFETY_HELMET_DEFAULT_SCORE_THRESHOLD,
            nms_threshold: SAFETY_HELMET_DEFAULT_NMS_THRESHOLD,
        },
        output_dir.join("safety_helmet_raw_frames"),
    )?;
    let mut bindings = [VideoAlgorithmBinding {
        algorithm: &mut helmet_algorithm,
        schedule: VideoAlgorithmSchedule::EveryFrame,
    }];

    let run = run_video_frame_pipeline(
        frames,
        &mut bindings,
        &VideoPipelineOptions {
            output_dir: output_dir.clone(),
            source_fps,
        },
    )?;

    dbg!(&run.files.frame_results_jsonl);
    dbg!(&run.files.summary_json);
    assert_existing_file(&run.files.frame_results_jsonl);
    assert_existing_file(&run.files.summary_json);
    // 关键断言：安全帽 runner 必须真实执行 ONNX 后处理并写出结构化文件。
    assert!(
        run.frame_results
            .iter()
            .all(|result| result.raw_json["raw_output_count"].as_u64().unwrap_or(0) > 0),
        "安全帽 runner 必须真实执行 ONNX 并写出输出"
    );
    assert!(
        run.frame_results
            .iter()
            .all(|result| result.raw_json["detected_safety_helmets_json"].is_string()),
        "安全帽 runner 必须写出结构化检测 JSON"
    );
    Ok(())
}
// video_pipeline_should_stack_all_frame_image_algorithms_on_one_frame_stream
#[test]
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印真实输入、模型、输出绝对路径"
)]
fn video_pipeline_should_stack_all_frame_image_algorithms_on_one_frame_stream() -> anyhow::Result<()>
{
    // 这个测试验证“其余图像识别怎么做视频实时计算”：
    // - 合成一张包含多种真实素材的测试帧，模拟视频解码层输出的同一帧。
    // - 人员、人脸使用已实现后处理的真实 runner，会输出检测框。
    // - 二维码使用纯 Rust 解码，会输出 payload 和角点框。
    // - 安全帽、火焰使用已实现后处理的真实 runner。
    // - 车辆目前接入 raw ONNX 适配器，只断言真实推理输出。
    let output_dir = output_dir("video_pipeline_all_frame_image_algorithms");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("清理旧全算法视频 pipeline 输出目录必须成功");
    }
    let source_fps = 30.0;
    let frames = frames_from_composite_algorithm_fixture(1, source_fps);
    dbg!(&face_fixture_path());
    dbg!(&person_fixture_path());
    dbg!(&safety_helmet_fixture_path());
    dbg!(&vehicle_fixture_path());
    dbg!(&flame_fixture_path());
    dbg!(&qr_code_fixture_path());
    dbg!(&face_model_path());
    dbg!(&person_model_path());
    dbg!(&safety_helmet_model_path());
    dbg!(&vehicle_model_path());
    dbg!(&flame_model_path());
    dbg!(&output_dir);

    let mut person_algorithm = RealPersonDetectionVideoAlgorithm::new(output_dir.clone())?;
    let mut face_algorithm = RealFaceDetectionVideoAlgorithm::new(output_dir.clone())?;
    let mut qr_code_algorithm = QrCodeVideoAlgorithm::new(output_dir.join("qr_code_frames"));
    let mut safety_helmet_algorithm = SafetyHelmetVideoAlgorithm::new(
        SafetyHelmetDetectionOptions {
            model_path: safety_helmet_model_path(),
            output_dir: output_dir.join("unused_safety_helmet_output"),
            score_threshold: SAFETY_HELMET_DEFAULT_SCORE_THRESHOLD,
            nms_threshold: SAFETY_HELMET_DEFAULT_NMS_THRESHOLD,
        },
        output_dir.join("safety_helmet_frames"),
    )?;
    let mut vehicle_algorithm = OnnxRawImageVideoAlgorithm::new(
        VEHICLE_DETECTION_CODE,
        VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1,
        vehicle_model_path(),
        output_dir.join("vehicle_raw_frames"),
    )?;
    let mut flame_algorithm = FlameVideoAlgorithm::new(
        FlameDetectionOptions {
            model_path: flame_model_path(),
            output_dir: output_dir.join("unused_flame_output"),
            score_threshold: FLAME_DEFAULT_SCORE_THRESHOLD,
            nms_threshold: FLAME_DEFAULT_NMS_THRESHOLD,
        },
        output_dir.join("flame_frames"),
    )?;
    let mut bindings = [
        VideoAlgorithmBinding {
            algorithm: &mut person_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
        VideoAlgorithmBinding {
            algorithm: &mut face_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
        VideoAlgorithmBinding {
            algorithm: &mut qr_code_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
        VideoAlgorithmBinding {
            algorithm: &mut safety_helmet_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
        VideoAlgorithmBinding {
            algorithm: &mut vehicle_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
        VideoAlgorithmBinding {
            algorithm: &mut flame_algorithm,
            schedule: VideoAlgorithmSchedule::EveryFrame,
        },
    ];

    let run = run_video_frame_pipeline(
        frames,
        &mut bindings,
        &VideoPipelineOptions {
            output_dir: output_dir.clone(),
            source_fps,
        },
    )?;

    dbg!(&run.files.frame_results_jsonl);
    dbg!(&run.files.summary_json);
    assert_existing_file(&run.files.frame_results_jsonl);
    assert_existing_file(&run.files.summary_json);
    // 关键断言：一帧输入流上，6 个算法各执行一次，证明可以在实时视频帧上叠加。
    assert_eq!(
        run.frame_results
            .iter()
            .map(|result| result.algorithm_code.as_str())
            .collect::<Vec<_>>(),
        vec![
            PERSON_DETECTION_CODE,
            "face_detection",
            QR_CODE_RECOGNITION_CODE,
            SAFETY_HELMET_DETECTION_CODE,
            VEHICLE_DETECTION_CODE,
            FLAME_DETECTION_CODE,
        ]
    );
    assert!(
        run.frame_results
            .iter()
            .find(|result| result.algorithm_code == "face_detection")
            .is_some_and(|result| !result.detections.is_empty()),
        "人脸检测必须返回真实人脸框"
    );
    assert!(
        run.frame_results
            .iter()
            .find(|result| result.algorithm_code == QR_CODE_RECOGNITION_CODE)
            .is_some_and(|result| {
                !result.detections.is_empty()
                    && result
                        .events
                        .iter()
                        .any(|event| event.message == "az-algorithm://真实二维码测试")
            }),
        "二维码算法必须返回真实 payload 和角点框"
    );
    for structured_code in [SAFETY_HELMET_DETECTION_CODE, FLAME_DETECTION_CODE] {
        assert!(
            run.frame_results
                .iter()
                .find(|result| result.algorithm_code == structured_code)
                .is_some_and(
                    |result| result.raw_json["raw_output_count"].as_u64().unwrap_or(0) > 0
                ),
            "{structured_code} 必须真实执行 ONNX 并写出结构化后处理输出"
        );
    }
    assert!(
        run.frame_results
            .iter()
            .find(|result| result.algorithm_code == VEHICLE_DETECTION_CODE)
            .is_some_and(|result| {
                result.detections.is_empty()
                    && result.raw_json["raw_output_count"].as_u64().unwrap_or(0) > 0
            }),
        "{VEHICLE_DETECTION_CODE} 当前必须只输出真实 raw ONNX 摘要，不伪造检测框"
    );
    Ok(())
}
