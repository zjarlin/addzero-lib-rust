//! 人脸检测执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use anyhow::{anyhow, bail};
use crate::components::face_detection::model::{
    DEFAULT_MODEL_FILE_NAME, DEFAULT_RESULT_DIR, FaceDetectionBox, FaceDetectionOptions,
    FaceDetectionOutputFiles, FaceDetectionOutputSummary, FaceDetectionRun, MODEL_CODE,
    MODEL_INPUT_HEIGHT, MODEL_INPUT_SHAPE, MODEL_INPUT_WIDTH,
};

const ANCHOR_COUNT: usize = 2;
const STRIDES: [usize; 3] = [8, 16, 32];
const OUTPUT_SAMPLE_VALUES: usize = 8;

impl FaceDetectionOptions {
    /// 使用当前 workspace 下的默认模型和默认输出目录。
    ///
    /// # Errors
    /// 当前工作目录无法定位或模型文件不存在时返回错误。
    pub fn default_workspace() -> anyhow::Result<Self> {
        let workspace_root = std::env::current_dir()
            .map_err(|source| path_error(PathBuf::from("."), source))?;
        let model_path = workspace_root
            .join("crates/algorithm/algorithm/resources/face_detection/models")
            .join(DEFAULT_MODEL_FILE_NAME);
        if !model_path.is_file() {
            bail!("model file `{}` is missing", (model_path).display());
        }

        Ok(Self {
            model_path,
            output_dir: workspace_root.join(DEFAULT_RESULT_DIR),
            score_threshold: 0.5,
            nms_threshold: 0.4,
        })
    }
}

/// 可复用的人脸检测模型实例。
///
/// 实时视频场景应在启动时构造一次 runner，然后对每帧调用检测方法，避免每帧重复加载 ONNX 模型。
#[derive(Debug)]
pub struct FaceDetectionRunner {
    options: FaceDetectionOptions,
    session: Session,
}

impl FaceDetectionRunner {
    /// 加载 SCRFD ONNX 模型，创建可复用 runner。
    ///
    /// # Errors
    /// 模型文件不存在、阈值非法或 ONNX Runtime 加载模型失败时返回错误。
    pub fn new(options: FaceDetectionOptions) -> anyhow::Result<Self> {
        validate_detection_options(&options)?;
        let mut builder = Session::builder()?;
        let session = builder.commit_from_file(&options.model_path)?;
        Ok(Self { options, session })
    }

    /// 返回 runner 当前使用的配置。
    #[must_use]
    pub const fn options(&self) -> &FaceDetectionOptions {
        &self.options
    }

    /// 对内存中的 RGB 帧执行真实人脸检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_rgb_image_with_output_dir(
        &mut self,
        image: RgbImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<FaceDetectionRun> {
        self.detect_dynamic_image_with_output_dir(DynamicImage::ImageRgb8(image), output_dir)
    }

    /// 对内存中的图片执行真实人脸检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_dynamic_image_with_output_dir(
        &mut self,
        image: DynamicImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<FaceDetectionRun> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)
            .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
        let (preview, inference, faces) = self.detect_image(&image)?;
        let files = write_output_files(
            &image,
            &preview,
            None,
            &faces,
            &inference.summaries,
            output_dir,
        )?;

        Ok(FaceDetectionRun {
            input_path: files.source_input.clone(),
            model_path: self.options.model_path.clone(),
            faces,
            files,
            raw_outputs: inference.summaries,
        })
    }

    fn detect_image(
        &mut self,
        image: &DynamicImage,
    ) -> anyhow::Result<(RgbImage, ScrfdInferenceOutput, Vec<FaceDetectionBox>)> {
        let prepared = prepare_image(image);
        let inference = run_scrfd_session(&mut self.session, prepared.tensor_data)?;
        let faces = decode_scrfd_face_boxes(
            &inference.tensors,
            MODEL_INPUT_WIDTH,
            MODEL_INPUT_HEIGHT,
            image.width(),
            image.height(),
            self.options.score_threshold,
            self.options.nms_threshold,
        )?;
        Ok((prepared.preview, inference, faces))
    }
}

/// 传入图片绝对路径执行人脸检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_faces_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<FaceDetectionRun> {
    let options = FaceDetectionOptions::default_workspace()?;
    detect_faces_from_path_with_options(image_path, &options)
}

/// 传入图片绝对路径和自定义配置执行人脸检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_faces_from_path_with_options(
    image_path: impl AsRef<Path>,
    options: &FaceDetectionOptions,
) -> anyhow::Result<FaceDetectionRun> {
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| path_error(image_path.as_ref().to_path_buf(), source))?;
    let image = image::open(&image_path)?;
    run_detection(image, image_path, options)
}

/// 传入图片二进制执行人脸检测。
///
/// # Errors
/// 图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_faces_from_bytes(bytes: &[u8]) -> anyhow::Result<FaceDetectionRun> {
    let options = FaceDetectionOptions::default_workspace()?;
    detect_faces_from_bytes_with_options(bytes, &options)
}

/// 传入图片二进制和自定义配置执行人脸检测。
///
/// # Errors
/// 图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_faces_from_bytes_with_options(
    bytes: &[u8],
    options: &FaceDetectionOptions,
) -> anyhow::Result<FaceDetectionRun> {
    detect_faces_from_bytes_named(bytes, "input_from_bytes.jpg", options)
}

/// 传入 base64 图片字符串执行人脸检测。
///
/// # Errors
/// base64 解码、图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_faces_from_base64(base64_image: &str) -> anyhow::Result<FaceDetectionRun> {
    let options = FaceDetectionOptions::default_workspace()?;
    detect_faces_from_base64_with_options(base64_image, &options)
}

/// 传入 base64 图片字符串和自定义配置执行人脸检测。
///
/// # Errors
/// base64 解码、图片解码、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_faces_from_base64_with_options(
    base64_image: &str,
    options: &FaceDetectionOptions,
) -> anyhow::Result<FaceDetectionRun> {
    let normalized = base64_image
        .split_once(',')
        .map_or(base64_image, |(_, payload)| payload);
    let bytes = base64::engine::general_purpose::STANDARD.decode(normalized)?;
    detect_faces_from_bytes_named(&bytes, "input_from_base64.jpg", options)
}

fn detect_faces_from_bytes_named(
    bytes: &[u8],
    input_file_name: &str,
    options: &FaceDetectionOptions,
) -> anyhow::Result<FaceDetectionRun> {
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;
    let input_path = options.output_dir.join(input_file_name);
    fs::write(&input_path, bytes)
        .map_err(|source| path_error(input_path.clone(), source))?;
    let image = image::load_from_memory(bytes)?;
    run_detection(image, input_path, options)
}

fn run_detection(
    image: DynamicImage,
    input_path: PathBuf,
    options: &FaceDetectionOptions,
) -> anyhow::Result<FaceDetectionRun> {
    validate_detection_options(options)?;
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;

    let prepared = prepare_image(&image);
    let inference = run_scrfd_model(&options.model_path, prepared.tensor_data)?;
    let faces = decode_scrfd_face_boxes(
        &inference.tensors,
        MODEL_INPUT_WIDTH,
        MODEL_INPUT_HEIGHT,
        image.width(),
        image.height(),
        options.score_threshold,
        options.nms_threshold,
    )?;
    let files = write_output_files(
        &image,
        &prepared.preview,
        Some(&input_path),
        &faces,
        &inference.summaries,
        &options.output_dir,
    )?;

    Ok(FaceDetectionRun {
        input_path,
        model_path: options.model_path.clone(),
        faces,
        files,
        raw_outputs: inference.summaries,
    })
}

fn prepare_image(image: &DynamicImage) -> PreparedFaceImage {
    let preview = image
        .resize_exact(MODEL_INPUT_WIDTH, MODEL_INPUT_HEIGHT, FilterType::Triangle)
        .to_rgb8();
    let tensor_data = rgb_to_nchw_f32(&preview);

    PreparedFaceImage {
        preview,
        tensor_data,
    }
}

fn rgb_to_nchw_f32(image: &RgbImage) -> Vec<f32> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![0.0; channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = normalize_scrfd_pixel(pixel[0]);
        data[channel_len + index] = normalize_scrfd_pixel(pixel[1]);
        data[channel_len * 2 + index] = normalize_scrfd_pixel(pixel[2]);
    }
    data
}

fn normalize_scrfd_pixel(value: u8) -> f32 {
    (f32::from(value) - 127.5) / 128.0
}

fn run_scrfd_model(
    model_path: &Path,
    tensor_data: Vec<f32>,
) -> anyhow::Result<ScrfdInferenceOutput> {
    if !model_path.is_file() {
        bail!("model file `{}` is missing", model_path.display());
    }

    let mut builder = Session::builder()?;
    let mut session = builder.commit_from_file(model_path)?;
    run_scrfd_session(&mut session, tensor_data)
}

fn run_scrfd_session(
    session: &mut Session,
    tensor_data: Vec<f32>,
) -> anyhow::Result<ScrfdInferenceOutput> {
    let input_array =
        ArrayD::from_shape_vec(IxDyn(MODEL_INPUT_SHAPE), tensor_data)
            .map_err(|source| anyhow!("invalid tensor shape for `{}`: {}", MODEL_CODE, source))?;
    let input = Tensor::from_array(input_array)?;
    let output_names = session
        .outputs()
        .iter()
        .map(|output| output.name().to_owned())
        .collect::<Vec<_>>();
    let outputs = session.run(ort::inputs![input])?;

    collect_scrfd_outputs(&output_names, outputs)
}

fn collect_scrfd_outputs(
    output_names: &[String],
    outputs: ort::session::SessionOutputs<'_>,
) -> anyhow::Result<ScrfdInferenceOutput> {
    let mut summaries = Vec::new();
    let mut tensors = Vec::new();

    for (index, (_name, value)) in outputs.iter().enumerate() {
        let output_name = output_names
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("output_{index}"));
        let ValueType::Tensor { ty, .. } = value.dtype() else {
            bail!("unsupported ONNX output tensor type `{}` from output `{}`", value.dtype(), output_name);
        };
        if !matches!(ty, TensorElementType::Float32) {
            bail!("unsupported ONNX output tensor type `{}` from output `{}`", ty, output_name);
        }

        let (shape, data) = value.try_extract_tensor::<f32>()?;
        let data = data.to_vec();
        summaries.push(FaceDetectionOutputSummary {
            name: output_name.clone(),
            tensor_type: ty.to_string(),
            shape: shape.iter().copied().collect(),
            element_count: data.len(),
            sample_f32: data.iter().take(OUTPUT_SAMPLE_VALUES).copied().collect(),
        });
        tensors.push(ScrfdOutputTensor {
            name: output_name,
            shape: shape.iter().copied().collect(),
            data,
        });
    }

    Ok(ScrfdInferenceOutput { summaries, tensors })
}

fn decode_scrfd_face_boxes(
    outputs: &[ScrfdOutputTensor],
    input_width: u32,
    input_height: u32,
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
    nms_threshold: f32,
) -> anyhow::Result<Vec<FaceDetectionBox>> {
    let mut boxes = Vec::new();

    for stride in STRIDES {
        let scores = require_output(outputs, &format!("score_{stride}"))?;
        let distances = require_output(outputs, &format!("bbox_{stride}"))?;
        decode_scrfd_stride(
            scores,
            distances,
            ScrfdGeometry {
                stride,
                input_width,
                input_height,
                image_width,
                image_height,
            },
            score_threshold,
            &mut boxes,
        )?;
    }

    Ok(non_maximum_suppression(boxes, nms_threshold))
}

fn decode_scrfd_stride(
    scores: &ScrfdOutputTensor,
    distances: &ScrfdOutputTensor,
    geometry: ScrfdGeometry,
    score_threshold: f32,
    boxes: &mut Vec<FaceDetectionBox>,
) -> anyhow::Result<()> {
    let feature_width = geometry.input_width as usize / geometry.stride;
    let feature_height = geometry.input_height as usize / geometry.stride;
    let expected_anchor_count = feature_width * feature_height * ANCHOR_COUNT;

    validate_scores(scores, expected_anchor_count)?;
    validate_distances(distances, expected_anchor_count)?;

    let x_scale = geometry.image_width as f32 / geometry.input_width as f32;
    let y_scale = geometry.image_height as f32 / geometry.input_height as f32;
    let stride = geometry.stride as f32;

    for row in 0..feature_height {
        for column in 0..feature_width {
            for anchor in 0..ANCHOR_COUNT {
                let index = ((row * feature_width + column) * ANCHOR_COUNT) + anchor;
                let confidence = scores.data[index];
                if confidence < score_threshold {
                    continue;
                }

                let distance_index = index * 4;
                let center_x = column as f32 * stride;
                let center_y = row as f32 * stride;
                let x_min = (center_x - distances.data[distance_index] * stride) * x_scale;
                let y_min = (center_y - distances.data[distance_index + 1] * stride) * y_scale;
                let x_max = (center_x + distances.data[distance_index + 2] * stride) * x_scale;
                let y_max = (center_y + distances.data[distance_index + 3] * stride) * y_scale;

                boxes.push(FaceDetectionBox {
                    x_min: x_min.clamp(0.0, geometry.image_width as f32),
                    y_min: y_min.clamp(0.0, geometry.image_height as f32),
                    x_max: x_max.clamp(0.0, geometry.image_width as f32),
                    y_max: y_max.clamp(0.0, geometry.image_height as f32),
                    confidence,
                });
            }
        }
    }

    Ok(())
}

fn write_output_files(
    image: &DynamicImage,
    preview: &RgbImage,
    input_path: Option<&Path>,
    faces: &[FaceDetectionBox],
    summaries: &[FaceDetectionOutputSummary],
    output_dir: &Path,
) -> anyhow::Result<FaceDetectionOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
    let files = FaceDetectionOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        detected_faces_json: output_dir.join("detected_faces.json"),
        detected_faces_image: output_dir.join("detected_faces.png"),
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

    let face_json = serde_json::to_string_pretty(faces).map_err(|source| {
        path_error(
            files.detected_faces_json.clone(),
            std::io::Error::other(source.to_string()),
        )
    })?;
    fs::write(&files.detected_faces_json, face_json)
        .map_err(|source| path_error(files.detected_faces_json.clone(), source))?;

    let mut marked_image = image.to_rgb8();
    for face in faces {
        draw_face_box(&mut marked_image, face);
    }
    marked_image.save(&files.detected_faces_image)?;

    Ok(files)
}

fn validate_detection_options(options: &FaceDetectionOptions) -> anyhow::Result<()> {
    if !options.model_path.is_file() {
        bail!("model file `{}` is missing", options.model_path.display());
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            MODEL_CODE,
            "score_threshold must be finite and within 0.0..=1.0"
        );
    }
    if !(0.0..=1.0).contains(&options.nms_threshold) || !options.nms_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            MODEL_CODE,
            "nms_threshold must be finite and within 0.0..=1.0"
        );
    }
    Ok(())
}

fn draw_face_box(image: &mut RgbImage, face: &FaceDetectionBox) {
    let x = face.x_min.round() as i32;
    let y = face.y_min.round() as i32;
    let width = (face.x_max - face.x_min).round().max(1.0) as u32;
    let height = (face.y_max - face.y_min).round().max(1.0) as u32;
    draw_hollow_rect_mut(
        image,
        Rect::at(x, y).of_size(width, height),
        Rgb([255, 0, 0]),
    );
}

fn require_output<'a>(
    outputs: &'a [ScrfdOutputTensor],
    output_name: &str,
) -> anyhow::Result<&'a ScrfdOutputTensor> {
    outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or_else(|| anyhow!("missing ONNX output `{}`", output_name.to_owned(),))
}

fn validate_scores(
    scores: &ScrfdOutputTensor,
    expected_anchor_count: usize,
) -> anyhow::Result<()> {
    if scores.shape.as_slice() == [1, expected_anchor_count as i64, 1]
        && scores.data.len() == expected_anchor_count
    {
        return Ok(());
    }

    bail!(
        "invalid tensor shape for `{}`: score output `{}` expected [1, {}, 1], got {:?}",
        MODEL_CODE,
        scores.name,
        expected_anchor_count,
        scores.shape
    )
}

fn validate_distances(
    distances: &ScrfdOutputTensor,
    expected_anchor_count: usize,
) -> anyhow::Result<()> {
    if distances.shape.as_slice() == [1, expected_anchor_count as i64, 4]
        && distances.data.len() == expected_anchor_count * 4
    {
        return Ok(());
    }

    bail!(
        "invalid tensor shape for `{}`: bbox output `{}` expected [1, {}, 4], got {:?}",
        MODEL_CODE,
        distances.name,
        expected_anchor_count,
        distances.shape
    )
}

fn non_maximum_suppression(
    mut boxes: Vec<FaceDetectionBox>,
    nms_threshold: f32,
) -> Vec<FaceDetectionBox> {
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

fn intersection_over_union(left: &FaceDetectionBox, right: &FaceDetectionBox) -> f32 {
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

fn area(face_box: &FaceDetectionBox) -> f32 {
    let width = (face_box.x_max - face_box.x_min).max(0.0);
    let height = (face_box.y_max - face_box.y_min).max(0.0);
    width * height
}

#[derive(Clone, Debug)]
struct PreparedFaceImage {
    preview: RgbImage,
    tensor_data: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
struct ScrfdOutputTensor {
    name: String,
    shape: Vec<i64>,
    data: Vec<f32>,
}

#[derive(Clone, Debug)]
struct ScrfdInferenceOutput {
    summaries: Vec<FaceDetectionOutputSummary>,
    tensors: Vec<ScrfdOutputTensor>,
}

#[derive(Clone, Copy, Debug)]
struct ScrfdGeometry {
    stride: usize,
    input_width: u32,
    input_height: u32,
    image_width: u32,
    image_height: u32,
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow!("filesystem error at `{}`: {source}", path.display())
}
