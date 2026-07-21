//! 人员检测执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use az_onnx::onnx::image::assist::prepare_image_tensor_for_spec;
use base64::Engine;
use half::f16;
use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::components::person_detection::model::{
    COCO_PERSON_CLASS_ID, DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RESULT_DIR, DEFAULT_SCORE_THRESHOLD,
    PERSON_DETECTION_COCO_SSD_MOBILENET_V1, PERSON_DETECTION_YOLO11N_COCO, PersonDetectionBox,
    PersonDetectionModelKind, PersonDetectionOptions, PersonDetectionOutputFiles,
    PersonDetectionOutputSummary, PersonDetectionRun, PersonVideoDetectionOptions,
    PersonVideoDetectionOutputFiles, PersonVideoDetectionRun, PersonVideoFrameDetection,
};
use anyhow::{anyhow, bail};

const OUTPUT_SAMPLE_VALUES: usize = 8;
const DETECTION_BOXES_OUTPUT: &str = "detection_boxes:0";
const DETECTION_CLASSES_OUTPUT: &str = "detection_classes:0";
const DETECTION_SCORES_OUTPUT: &str = "detection_scores:0";
const NUM_DETECTIONS_OUTPUT: &str = "num_detections:0";
const YOLO_OUTPUT: &str = "output0";
const YOLO_COCO_PERSON_CLASS_INDEX: usize = 0;
const YOLO_NMS_THRESHOLD: f32 = 0.45;

impl PersonDetectionOptions {
    /// 使用当前 workspace 下的默认模型和默认输出目录。
    ///
    /// # Errors
    /// 当前工作目录无法定位或模型文件不存在时返回错误。
    pub fn default_workspace() -> anyhow::Result<Self> {
        let workspace_root = workspace_root()?;
        let model_path = workspace_root
            .join("crates/algorithm/algorithm")
            .join(DEFAULT_MODEL_RESOURCE_DIR)
            .join(PERSON_DETECTION_COCO_SSD_MOBILENET_V1.local_file);
        if !model_path.is_file() {
            bail!("model file `{}` is missing", (model_path).display());
        }

        Ok(Self {
            model_path,
            model_kind: PersonDetectionModelKind::CocoSsdMobileNetV1,
            output_dir: workspace_root.join(DEFAULT_RESULT_DIR),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
        })
    }
}

/// 可复用的人员检测模型实例。
///
/// 实时视频场景应在启动时构造一次 runner，然后对每帧调用检测方法，避免每帧重复加载 ONNX 模型。
#[derive(Debug)]
pub struct PersonDetectionRunner {
    options: PersonDetectionOptions,
    session: Session,
}

impl PersonDetectionRunner {
    /// 加载人员检测 ONNX 模型，创建可复用 runner。
    ///
    /// # Errors
    /// 模型文件不存在、阈值非法或 ONNX Runtime 加载模型失败时返回错误。
    pub fn new(options: PersonDetectionOptions) -> anyhow::Result<Self> {
        validate_detection_options(&options)?;
        let mut builder = Session::builder()?;
        let session = builder.commit_from_file(&options.model_path)?;
        Ok(Self { options, session })
    }

    /// 返回 runner 当前使用的配置。
    #[must_use]
    pub const fn options(&self) -> &PersonDetectionOptions {
        &self.options
    }

    /// 对内存中的 RGB 帧执行真实人员检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_rgb_image_with_output_dir(
        &mut self,
        image: RgbImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<PersonDetectionRun> {
        self.detect_dynamic_image_with_output_dir(DynamicImage::ImageRgb8(image), output_dir)
    }

    /// 对内存中的图片执行真实人员检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_dynamic_image_with_output_dir(
        &mut self,
        image: DynamicImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<PersonDetectionRun> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)
            .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
        let (preview, inference, persons) = self.detect_image(&image)?;
        let files = write_output_files(
            &image,
            &preview,
            None,
            &persons,
            &inference.summaries,
            output_dir,
        )?;

        Ok(PersonDetectionRun {
            input_path: files.source_input.clone(),
            model_path: self.options.model_path.clone(),
            persons,
            files,
            raw_outputs: inference.summaries,
        })
    }

    fn detect_image(
        &mut self,
        image: &DynamicImage,
    ) -> anyhow::Result<(RgbImage, CocoSsdInferenceOutput, Vec<PersonDetectionBox>)> {
        match self.options.model_kind {
            PersonDetectionModelKind::CocoSsdMobileNetV1 => {
                let prepared = prepare_coco_ssd_image(image);
                let inference = run_coco_ssd_session(&mut self.session, prepared.tensor_data)?;
                let persons = decode_coco_ssd_person_boxes(
                    &inference.tensors,
                    image.width(),
                    image.height(),
                    self.options.score_threshold,
                )?;
                Ok((prepared.preview, inference, persons))
            }
            PersonDetectionModelKind::Yolo11nCoco => {
                let prepared = prepare_yolo_image(image);
                let inference = run_yolo11n_session(&mut self.session, prepared.tensor_data)?;
                let persons = decode_yolo_person_boxes(
                    &inference.tensors,
                    image.width(),
                    image.height(),
                    self.options.score_threshold,
                )?;
                Ok((prepared.preview, inference, persons))
            }
        }
    }
}

/// 传入图片绝对路径执行人员检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_persons_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<PersonDetectionRun> {
    let options = PersonDetectionOptions::default_workspace()?;
    detect_persons_from_path_with_options(image_path, &options)
}

/// 传入图片绝对路径和自定义配置执行人员检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_persons_from_path_with_options(
    image_path: impl AsRef<Path>,
    options: &PersonDetectionOptions,
) -> anyhow::Result<PersonDetectionRun> {
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| path_error(image_path.as_ref().to_path_buf(), source))?;
    let image = image::open(&image_path)?;
    run_detection(image, image_path, options)
}

/// 传入图片二进制执行人员检测。
///
/// # Errors
/// 图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_persons_from_bytes(bytes: &[u8]) -> anyhow::Result<PersonDetectionRun> {
    let options = PersonDetectionOptions::default_workspace()?;
    detect_persons_from_bytes_with_options(bytes, &options)
}

/// 传入图片二进制和自定义配置执行人员检测。
///
/// # Errors
/// 图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_persons_from_bytes_with_options(
    bytes: &[u8],
    options: &PersonDetectionOptions,
) -> anyhow::Result<PersonDetectionRun> {
    detect_persons_from_bytes_named(bytes, "input_from_bytes.jpg", options)
}

/// 传入 base64 图片字符串执行人员检测。
///
/// # Errors
/// base64 解码、图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_persons_from_base64(base64_image: &str) -> anyhow::Result<PersonDetectionRun> {
    let options = PersonDetectionOptions::default_workspace()?;
    detect_persons_from_base64_with_options(base64_image, &options)
}

/// 传入 base64 图片字符串和自定义配置执行人员检测。
///
/// # Errors
/// base64 解码、图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_persons_from_base64_with_options(
    base64_image: &str,
    options: &PersonDetectionOptions,
) -> anyhow::Result<PersonDetectionRun> {
    let normalized = base64_image
        .split_once(',')
        .map_or(base64_image, |(_, payload)| payload);
    let bytes = base64::engine::general_purpose::STANDARD.decode(normalized)?;
    detect_persons_from_bytes_named(&bytes, "input_from_base64.jpg", options)
}

/// 兼容旧命名：使用默认模型和默认输出目录执行人员检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_person_detection_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<PersonDetectionRun> {
    detect_persons_from_path(image_path)
}

/// 兼容旧命名：使用默认模型和指定输出目录执行人员检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_person_detection_from_path_with_output(
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> anyhow::Result<PersonDetectionRun> {
    detect_persons_from_path_with_options(
        image_path,
        &PersonDetectionOptions {
            model_path: crate_root()
                .join(DEFAULT_MODEL_RESOURCE_DIR)
                .join(PERSON_DETECTION_COCO_SSD_MOBILENET_V1.local_file),
            model_kind: PersonDetectionModelKind::CocoSsdMobileNetV1,
            output_dir: output_dir.as_ref().to_path_buf(),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
        },
    )
}

/// 传入视频绝对路径，抽帧执行真实人员检测，并输出带人员框的标注视频。
///
/// 该接口只做人员检测，不识别敲击动作。敲击动作需要额外的动作、姿态或接触观测输入。
///
/// # Errors
/// 视频文件、ffmpeg、图片推理或输出文件写入失败时返回错误。
pub fn detect_persons_in_video_from_path(
    video_path: impl AsRef<Path>,
    options: &PersonVideoDetectionOptions,
) -> anyhow::Result<PersonVideoDetectionRun> {
    validate_video_options(options)?;
    let video_path = std::fs::canonicalize(video_path.as_ref())
        .map_err(|source| path_error(video_path.as_ref().to_path_buf(), source))?;
    recreate_dir(&options.output_dir)?;

    let files = PersonVideoDetectionOutputFiles {
        source_input_video: options.output_dir.join("source_input.mp4"),
        extracted_frame_dir: options.output_dir.join("extracted_frames"),
        annotated_frame_dir: options.output_dir.join("annotated_frames"),
        frame_detections_json: options.output_dir.join("frame_detections.json"),
        annotated_video: options.output_dir.join("annotated_persons.mp4"),
    };
    fs::create_dir_all(&files.extracted_frame_dir)
        .map_err(|source| path_error(files.extracted_frame_dir.clone(), source))?;
    fs::create_dir_all(&files.annotated_frame_dir)
        .map_err(|source| path_error(files.annotated_frame_dir.clone(), source))?;
    fs::copy(&video_path, &files.source_input_video)
        .map_err(|source| path_error(video_path.clone(), source))?;

    extract_video_frames(&video_path, &files.extracted_frame_dir, options)?;
    let frame_paths = collected_frame_paths(&files.extracted_frame_dir, options.max_frames)?;
    let mut frames = Vec::new();
    for (frame_index, frame_path) in frame_paths.iter().enumerate() {
        let frame_output_dir = options
            .output_dir
            .join("per_frame_detection")
            .join(format!("frame_{frame_index:05}"));
        let image_run = detect_persons_from_path_with_options(
            frame_path,
            &PersonDetectionOptions {
                model_path: options.model_path.clone(),
                model_kind: options.model_kind,
                output_dir: frame_output_dir,
                score_threshold: options.score_threshold,
            },
        )?;
        let annotated_frame_path = files
            .annotated_frame_dir
            .join(format!("frame_{frame_index:05}.png"));
        fs::copy(
            &image_run.files.detected_persons_image,
            &annotated_frame_path,
        )
        .map_err(|source| path_error(image_run.files.detected_persons_image.clone(), source))?;
        frames.push(PersonVideoFrameDetection {
            frame_index,
            timestamp_ms: frame_timestamp_ms(frame_index, options.sample_fps),
            frame_path: frame_path.clone(),
            annotated_frame_path,
            persons: image_run.persons,
        });
    }

    let frame_json = serde_json::to_string_pretty(&frames).map_err(|source| {
        path_error(
            files.frame_detections_json.clone(),
            std::io::Error::other(source.to_string()),
        )
    })?;
    fs::write(&files.frame_detections_json, frame_json)
        .map_err(|source| path_error(files.frame_detections_json.clone(), source))?;
    encode_annotated_video(&files.annotated_frame_dir, &files.annotated_video, options)?;

    Ok(PersonVideoDetectionRun {
        input_video_path: video_path,
        model_path: options.model_path.clone(),
        files,
        frames,
    })
}

fn detect_persons_from_bytes_named(
    bytes: &[u8],
    input_file_name: &str,
    options: &PersonDetectionOptions,
) -> anyhow::Result<PersonDetectionRun> {
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;
    let input_path = options.output_dir.join(input_file_name);
    fs::write(&input_path, bytes).map_err(|source| path_error(input_path.clone(), source))?;
    let image = image::load_from_memory(bytes)?;
    run_detection(image, input_path, options)
}

fn run_detection(
    image: DynamicImage,
    input_path: PathBuf,
    options: &PersonDetectionOptions,
) -> anyhow::Result<PersonDetectionRun> {
    validate_detection_options(options)?;
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;

    let (preview, inference, persons) = match options.model_kind {
        PersonDetectionModelKind::CocoSsdMobileNetV1 => {
            let prepared = prepare_image_tensor_for_spec(
                &PERSON_DETECTION_COCO_SSD_MOBILENET_V1,
                &input_path,
            )?;
            let tensor_data = prepared
                .u8_tensor_data()
                .ok_or_else(|| {
                    anyhow!(
                        "invalid tensor shape for `{}`: {}",
                        PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
                        "prepared image does not contain u8 tensor data".to_owned(),
                    )
                })?
                .to_vec();
            let inference = run_coco_ssd_model(&options.model_path, tensor_data)?;
            let persons = decode_coco_ssd_person_boxes(
                &inference.tensors,
                image.width(),
                image.height(),
                options.score_threshold,
            )?;
            (prepared.preview, inference, persons)
        }
        PersonDetectionModelKind::Yolo11nCoco => {
            let prepared = prepare_yolo_image(&image);
            let inference = run_yolo11n_model(&options.model_path, prepared.tensor_data)?;
            let persons = decode_yolo_person_boxes(
                &inference.tensors,
                image.width(),
                image.height(),
                options.score_threshold,
            )?;
            (prepared.preview, inference, persons)
        }
    };
    let files = write_output_files(
        &image,
        &preview,
        Some(&input_path),
        &persons,
        &inference.summaries,
        &options.output_dir,
    )?;

    Ok(PersonDetectionRun {
        input_path,
        model_path: options.model_path.clone(),
        persons,
        files,
        raw_outputs: inference.summaries,
    })
}

fn run_coco_ssd_model(
    model_path: &Path,
    tensor_data: Vec<u8>,
) -> anyhow::Result<CocoSsdInferenceOutput> {
    if !model_path.is_file() {
        bail!("model file `{}` is missing", model_path.display());
    }

    let mut builder = Session::builder()?;
    let mut session = builder.commit_from_file(model_path)?;
    run_coco_ssd_session(&mut session, tensor_data)
}

fn run_coco_ssd_session(
    session: &mut Session,
    tensor_data: Vec<u8>,
) -> anyhow::Result<CocoSsdInferenceOutput> {
    let input_array = ArrayD::from_shape_vec(
        IxDyn(PERSON_DETECTION_COCO_SSD_MOBILENET_V1.input.shape),
        tensor_data,
    )
    .map_err(|source| {
        anyhow!(
            "invalid tensor shape for `{}`: {}",
            PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
            source,
        )
    })?;
    let input = Tensor::from_array(input_array)?;
    let output_names = session
        .outputs()
        .iter()
        .map(|output| output.name().to_owned())
        .collect::<Vec<_>>();
    let outputs = session.run(ort::inputs![input])?;

    collect_coco_ssd_outputs(&output_names, outputs)
}

fn run_yolo11n_model(
    model_path: &Path,
    tensor_data: Vec<f16>,
) -> anyhow::Result<CocoSsdInferenceOutput> {
    if !model_path.is_file() {
        bail!("model file `{}` is missing", model_path.display());
    }

    let mut builder = Session::builder()?;
    let mut session = builder.commit_from_file(model_path)?;
    run_yolo11n_session(&mut session, tensor_data)
}

fn run_yolo11n_session(
    session: &mut Session,
    tensor_data: Vec<f16>,
) -> anyhow::Result<CocoSsdInferenceOutput> {
    let input_array = ArrayD::from_shape_vec(
        IxDyn(PERSON_DETECTION_YOLO11N_COCO.input.shape),
        tensor_data,
    )
    .map_err(|source| {
        anyhow!(
            "invalid tensor shape for `{}`: {}",
            PERSON_DETECTION_YOLO11N_COCO.code,
            source,
        )
    })?;
    let input = Tensor::from_array(input_array)?;
    let output_names = session
        .outputs()
        .iter()
        .map(|output| output.name().to_owned())
        .collect::<Vec<_>>();
    let outputs = session.run(ort::inputs![input])?;

    collect_yolo_outputs(&output_names, outputs)
}

fn collect_yolo_outputs(
    output_names: &[String],
    outputs: ort::session::SessionOutputs<'_>,
) -> anyhow::Result<CocoSsdInferenceOutput> {
    let mut summaries = Vec::new();
    let mut tensors = Vec::new();

    for (index, (_name, value)) in outputs.iter().enumerate() {
        let output_name = output_names
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("output_{index}"));
        let ValueType::Tensor { ty, .. } = value.dtype() else {
            bail!(
                "unsupported ONNX output tensor type `{}` from output `{}`",
                value.dtype(),
                output_name
            );
        };
        if !matches!(ty, TensorElementType::Float16) {
            bail!(
                "unsupported ONNX output tensor type `{}` from output `{}`",
                ty,
                output_name
            );
        }

        let (shape, data) = value.try_extract_tensor::<f16>()?;
        let data = data.iter().map(|value| value.to_f32()).collect::<Vec<_>>();
        summaries.push(PersonDetectionOutputSummary {
            name: output_name.clone(),
            tensor_type: ty.to_string(),
            shape: shape.iter().copied().collect(),
            element_count: data.len(),
            sample_f32: data.iter().take(OUTPUT_SAMPLE_VALUES).copied().collect(),
        });
        tensors.push(CocoSsdOutputTensor {
            name: output_name,
            shape: shape.iter().copied().collect(),
            data,
        });
    }

    Ok(CocoSsdInferenceOutput { summaries, tensors })
}

fn collect_coco_ssd_outputs(
    output_names: &[String],
    outputs: ort::session::SessionOutputs<'_>,
) -> anyhow::Result<CocoSsdInferenceOutput> {
    let mut summaries = Vec::new();
    let mut tensors = Vec::new();

    for (index, (_name, value)) in outputs.iter().enumerate() {
        let output_name = output_names
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("output_{index}"));
        let ValueType::Tensor { ty, .. } = value.dtype() else {
            bail!(
                "unsupported ONNX output tensor type `{}` from output `{}`",
                value.dtype(),
                output_name
            );
        };
        if !matches!(ty, TensorElementType::Float32) {
            bail!(
                "unsupported ONNX output tensor type `{}` from output `{}`",
                ty,
                output_name
            );
        }

        let (shape, data) = value.try_extract_tensor::<f32>()?;
        let data = data.to_vec();
        summaries.push(PersonDetectionOutputSummary {
            name: output_name.clone(),
            tensor_type: ty.to_string(),
            shape: shape.iter().copied().collect(),
            element_count: data.len(),
            sample_f32: data.iter().take(OUTPUT_SAMPLE_VALUES).copied().collect(),
        });
        tensors.push(CocoSsdOutputTensor {
            name: output_name,
            shape: shape.iter().copied().collect(),
            data,
        });
    }

    Ok(CocoSsdInferenceOutput { summaries, tensors })
}

fn decode_coco_ssd_person_boxes(
    outputs: &[CocoSsdOutputTensor],
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
) -> anyhow::Result<Vec<PersonDetectionBox>> {
    let boxes = require_output(outputs, DETECTION_BOXES_OUTPUT)?;
    let classes = require_output(outputs, DETECTION_CLASSES_OUTPUT)?;
    let scores = require_output(outputs, DETECTION_SCORES_OUTPUT)?;
    let num_detections = require_output(outputs, NUM_DETECTIONS_OUTPUT)?;

    validate_boxes(boxes)?;
    validate_vector(classes, DETECTION_CLASSES_OUTPUT)?;
    validate_vector(scores, DETECTION_SCORES_OUTPUT)?;
    validate_num_detections(num_detections)?;

    let detection_count = num_detections.data[0]
        .round()
        .clamp(0.0, scores.data.len() as f32) as usize;
    let mut persons = Vec::new();
    for index in 0..detection_count {
        let class_id = classes.data[index];
        let confidence = scores.data[index];
        if (class_id - COCO_PERSON_CLASS_ID).abs() > f32::EPSILON || confidence < score_threshold {
            continue;
        }

        let box_index = index * 4;
        let y_min = boxes.data[box_index].clamp(0.0, 1.0) * image_height as f32;
        let x_min = boxes.data[box_index + 1].clamp(0.0, 1.0) * image_width as f32;
        let y_max = boxes.data[box_index + 2].clamp(0.0, 1.0) * image_height as f32;
        let x_max = boxes.data[box_index + 3].clamp(0.0, 1.0) * image_width as f32;
        persons.push(PersonDetectionBox {
            x_min,
            y_min,
            x_max,
            y_max,
            class_id,
            confidence,
        });
    }

    persons.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    Ok(persons)
}

fn decode_yolo_person_boxes(
    outputs: &[CocoSsdOutputTensor],
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
) -> anyhow::Result<Vec<PersonDetectionBox>> {
    let output = require_output(outputs, YOLO_OUTPUT)?;
    if output.shape.as_slice() != [1, 84, 8400] || output.data.len() != 84 * 8400 {
        bail!(
            "invalid tensor shape for `{}`: output `{}` expected [1, 84, 8400], got {:?}",
            PERSON_DETECTION_YOLO11N_COCO.code,
            output.name,
            output.shape
        );
    }

    let mut boxes = Vec::new();
    let width_scale = image_width as f32 / 640.0;
    let height_scale = image_height as f32 / 640.0;
    for candidate_index in 0..8400 {
        let confidence = yolo_value(output, 4 + YOLO_COCO_PERSON_CLASS_INDEX, candidate_index);
        if confidence < score_threshold {
            continue;
        }

        let center_x = yolo_value(output, 0, candidate_index) * width_scale;
        let center_y = yolo_value(output, 1, candidate_index) * height_scale;
        let width = yolo_value(output, 2, candidate_index) * width_scale;
        let height = yolo_value(output, 3, candidate_index) * height_scale;
        boxes.push(PersonDetectionBox {
            x_min: (center_x - width / 2.0).clamp(0.0, image_width as f32),
            y_min: (center_y - height / 2.0).clamp(0.0, image_height as f32),
            x_max: (center_x + width / 2.0).clamp(0.0, image_width as f32),
            y_max: (center_y + height / 2.0).clamp(0.0, image_height as f32),
            class_id: COCO_PERSON_CLASS_ID,
            confidence,
        });
    }

    Ok(non_maximum_suppression(boxes, YOLO_NMS_THRESHOLD))
}

fn yolo_value(output: &CocoSsdOutputTensor, channel: usize, candidate_index: usize) -> f32 {
    output.data[channel * 8400 + candidate_index]
}

fn write_output_files(
    image: &DynamicImage,
    preview: &RgbImage,
    input_path: Option<&Path>,
    persons: &[PersonDetectionBox],
    summaries: &[PersonDetectionOutputSummary],
    output_dir: &Path,
) -> anyhow::Result<PersonDetectionOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
    let files = PersonDetectionOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        detected_persons_json: output_dir.join("detected_persons.json"),
        detected_persons_image: output_dir.join("detected_persons.png"),
    };

    if let Some(input_path) = input_path {
        fs::copy(input_path, &files.source_input)
            .map_err(|source| path_error(input_path.to_path_buf(), source))?;
    } else {
        image.save(&files.source_input)?;
    }
    preview.save(&files.model_input_preview)?;

    let raw_json = serde_json::to_string_pretty(summaries).map_err(|source| {
        path_error(
            files.raw_outputs_json.clone(),
            std::io::Error::other(source.to_string()),
        )
    })?;
    fs::write(&files.raw_outputs_json, raw_json)
        .map_err(|source| path_error(files.raw_outputs_json.clone(), source))?;

    let person_json = serde_json::to_string_pretty(persons).map_err(|source| {
        path_error(
            files.detected_persons_json.clone(),
            std::io::Error::other(source.to_string()),
        )
    })?;
    fs::write(&files.detected_persons_json, person_json)
        .map_err(|source| path_error(files.detected_persons_json.clone(), source))?;

    let mut marked_image = image.to_rgb8();
    for person in persons {
        draw_person_box(&mut marked_image, person);
    }
    marked_image.save(&files.detected_persons_image)?;

    Ok(files)
}

fn prepare_coco_ssd_image(image: &DynamicImage) -> PreparedCocoSsdImage {
    let preview = image
        .resize_exact(1200, 1200, FilterType::Triangle)
        .to_rgb8();
    let tensor_data = rgb_to_nhwc_u8(&preview);
    PreparedCocoSsdImage {
        preview,
        tensor_data,
    }
}

fn prepare_yolo_image(image: &DynamicImage) -> PreparedYoloImage {
    let preview = image.resize_exact(640, 640, FilterType::Triangle).to_rgb8();
    let tensor_data = rgb_to_nchw_f16_normalized(&preview);
    PreparedYoloImage {
        preview,
        tensor_data,
    }
}

fn rgb_to_nhwc_u8(image: &RgbImage) -> Vec<u8> {
    image.pixels().flat_map(|pixel| pixel.0).collect()
}

fn rgb_to_nchw_f16_normalized(image: &RgbImage) -> Vec<f16> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![f16::from_f32(0.0); channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = f16::from_f32(f32::from(pixel[0]) / 255.0);
        data[channel_len + index] = f16::from_f32(f32::from(pixel[1]) / 255.0);
        data[channel_len * 2 + index] = f16::from_f32(f32::from(pixel[2]) / 255.0);
    }
    data
}

fn draw_person_box(image: &mut RgbImage, person: &PersonDetectionBox) {
    let x = person.x_min.round() as i32;
    let y = person.y_min.round() as i32;
    let width = (person.x_max - person.x_min).round().max(1.0) as u32;
    let height = (person.y_max - person.y_min).round().max(1.0) as u32;
    draw_hollow_rect_mut(
        image,
        Rect::at(x, y).of_size(width, height),
        Rgb([0, 220, 80]),
    );
}

fn non_maximum_suppression(
    mut boxes: Vec<PersonDetectionBox>,
    nms_threshold: f32,
) -> Vec<PersonDetectionBox> {
    boxes.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));

    let mut kept = Vec::new();
    for candidate in boxes {
        let overlaps_existing = kept
            .iter()
            .any(|selected| intersection_over_union(&candidate, selected) > nms_threshold);
        if !overlaps_existing {
            kept.push(candidate);
        }
    }

    kept
}

fn intersection_over_union(left: &PersonDetectionBox, right: &PersonDetectionBox) -> f32 {
    let intersection_x_min = left.x_min.max(right.x_min);
    let intersection_y_min = left.y_min.max(right.y_min);
    let intersection_x_max = left.x_max.min(right.x_max);
    let intersection_y_max = left.y_max.min(right.y_max);
    let intersection_width = (intersection_x_max - intersection_x_min).max(0.0);
    let intersection_height = (intersection_y_max - intersection_y_min).max(0.0);
    let intersection = intersection_width * intersection_height;
    let union = area(left) + area(right) - intersection;

    if union <= 0.0 {
        return 0.0;
    }

    intersection / union
}

fn area(person_box: &PersonDetectionBox) -> f32 {
    let width = (person_box.x_max - person_box.x_min).max(0.0);
    let height = (person_box.y_max - person_box.y_min).max(0.0);
    width * height
}

fn require_output<'a>(
    outputs: &'a [CocoSsdOutputTensor],
    output_name: &str,
) -> anyhow::Result<&'a CocoSsdOutputTensor> {
    outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or_else(|| anyhow!("missing ONNX output `{}`", output_name.to_owned(),))
}

fn validate_boxes(boxes: &CocoSsdOutputTensor) -> anyhow::Result<()> {
    if boxes.shape.as_slice() == [1, 100, 4] && boxes.data.len() == 400 {
        return Ok(());
    }

    bail!(
        "invalid tensor shape for `{}`: output `{}` expected [1, 100, 4], got {:?}",
        PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
        boxes.name,
        boxes.shape
    )
}

fn validate_vector(output: &CocoSsdOutputTensor, expected_name: &str) -> anyhow::Result<()> {
    if output.shape.as_slice() == [1, 100] && output.data.len() == 100 {
        return Ok(());
    }

    bail!(
        "invalid tensor shape for `{}`: output `{}` expected [1, 100], got {:?}",
        PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
        expected_name,
        output.shape
    )
}

fn validate_num_detections(output: &CocoSsdOutputTensor) -> anyhow::Result<()> {
    if output.shape.as_slice() == [1] && output.data.len() == 1 {
        return Ok(());
    }

    bail!(
        "invalid tensor shape for `{}`: output `{}` expected [1], got {:?}",
        PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
        output.name,
        output.shape
    )
}

fn validate_video_options(options: &PersonVideoDetectionOptions) -> anyhow::Result<()> {
    if !options.model_path.is_file() {
        bail!("model file `{}` is missing", options.model_path.display());
    }
    if options.sample_fps == 0 {
        bail!(
            "invalid tensor shape for `{}`: {}",
            PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
            "sample_fps must be greater than 0"
        );
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            PERSON_DETECTION_COCO_SSD_MOBILENET_V1.code,
            "score_threshold must be finite and within 0.0..=1.0"
        );
    }
    Ok(())
}

fn validate_detection_options(options: &PersonDetectionOptions) -> anyhow::Result<()> {
    if !options.model_path.is_file() {
        bail!("model file `{}` is missing", options.model_path.display());
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            options.model_kind.spec().code,
            "score_threshold must be finite and within 0.0..=1.0"
        );
    }
    Ok(())
}

fn recreate_dir(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| path_error(path.to_path_buf(), source))?;
    }
    fs::create_dir_all(path).map_err(|source| path_error(path.to_path_buf(), source))
}

fn extract_video_frames(
    video_path: &Path,
    extracted_frame_dir: &Path,
    options: &PersonVideoDetectionOptions,
) -> anyhow::Result<()> {
    run_ffmpeg(
        &options.ffmpeg_path,
        &[
            "-y".to_owned(),
            "-i".to_owned(),
            video_path.display().to_string(),
            "-vf".to_owned(),
            format!("fps={}", options.sample_fps),
            extracted_frame_dir
                .join("frame_%05d.png")
                .display()
                .to_string(),
        ],
        extracted_frame_dir,
    )
}

fn encode_annotated_video(
    annotated_frame_dir: &Path,
    annotated_video: &Path,
    options: &PersonVideoDetectionOptions,
) -> anyhow::Result<()> {
    run_ffmpeg(
        &options.ffmpeg_path,
        &[
            "-y".to_owned(),
            "-framerate".to_owned(),
            options.sample_fps.to_string(),
            "-i".to_owned(),
            annotated_frame_dir
                .join("frame_%05d.png")
                .display()
                .to_string(),
            "-c:v".to_owned(),
            "libx264".to_owned(),
            "-pix_fmt".to_owned(),
            "yuv420p".to_owned(),
            annotated_video.display().to_string(),
        ],
        annotated_video,
    )
}

fn run_ffmpeg(ffmpeg_path: &Path, args: &[String], context_path: &Path) -> anyhow::Result<()> {
    let output = Command::new(ffmpeg_path)
        .args(args)
        .output()
        .map_err(|source| path_error(ffmpeg_path.to_path_buf(), source))?;
    if output.status.success() {
        return Ok(());
    }

    Err(path_error(
        context_path.to_path_buf(),
        std::io::Error::other(format!(
            "ffmpeg failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )),
    ))
}

fn collected_frame_paths(
    extracted_frame_dir: &Path,
    max_frames: Option<usize>,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut frame_paths = fs::read_dir(extracted_frame_dir)
        .map_err(|source| path_error(extracted_frame_dir.to_path_buf(), source))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| path_error(extracted_frame_dir.to_path_buf(), source))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    frame_paths.sort();
    if let Some(limit) = max_frames {
        frame_paths.truncate(limit);
    }
    if frame_paths.is_empty() {
        let path = extracted_frame_dir.to_path_buf();
        let source = std::io::Error::other("ffmpeg did not extract any video frames");
        let error = path_error(path, source);

        return Err(error);
    }
    Ok(frame_paths)
}

fn frame_timestamp_ms(frame_index: usize, sample_fps: u32) -> u64 {
    (frame_index as u64 * 1_000) / u64::from(sample_fps)
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .map_err(|source| path_error(PathBuf::from(env!("CARGO_MANIFEST_DIR")), source))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Clone, Debug, PartialEq)]
struct CocoSsdOutputTensor {
    name: String,
    shape: Vec<i64>,
    data: Vec<f32>,
}

#[derive(Clone, Debug)]
struct PreparedCocoSsdImage {
    preview: RgbImage,
    tensor_data: Vec<u8>,
}

#[derive(Clone, Debug)]
struct PreparedYoloImage {
    preview: RgbImage,
    tensor_data: Vec<f16>,
}

#[derive(Clone, Debug)]
struct CocoSsdInferenceOutput {
    summaries: Vec<PersonDetectionOutputSummary>,
    tensors: Vec<CocoSsdOutputTensor>,
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow!("filesystem error at `{}`: {source}", path.display())
}
