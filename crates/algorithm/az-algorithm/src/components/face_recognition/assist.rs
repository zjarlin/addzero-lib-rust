//! 人脸识别相似度辅助函数。

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ab_glyph::{FontArc, PxScale};
use anyhow::{Context, anyhow, bail};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut};
use imageproc::geometric_transformations::{Interpolation, Projection, warp_into};
use imageproc::rect::Rect;

use crate::components::face_detection::model::FaceDetectionBox;
use crate::components::face_recognition::model::{
    ALGORITHM_CODE, DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RESULT_DIR,
    DEFAULT_SAME_IDENTITY_THRESHOLD, FACE_RECOGNITION_SFACE_2021DEC, FaceEmbeddingRun,
    FaceRecognitionOutputFiles, FaceRecognitionRun,
};
use az_onnx::onnx::image::assist::{
    LocalOnnxSession, write_inference_artifacts_from_image,
};
use az_onnx::onnx::image::model::{
    OnnxImageRun, OnnxInferenceSummary, OnnxOutputSummary, PreparedImageTensor,
};

const YUNET_MODEL_FILE_NAME: &str = "face_detection_yunet_2023mar.onnx";
const YUNET_MODEL_INPUT_SHAPE: &[usize] = &[1, 3, 640, 640];
const YUNET_MODEL_INPUT_WIDTH: u32 = 640;
const YUNET_MODEL_INPUT_HEIGHT: u32 = 640;
const YUNET_STRIDES: [usize; 3] = [8, 16, 32];
const YUNET_SCORE_THRESHOLD: f32 = 0.6;
const YUNET_NMS_THRESHOLD: f32 = 0.3;
const COMPARISON_WIDTH: u32 = 960;
const COMPARISON_IMAGE_HEIGHT: u32 = 480;
const COMPARISON_PANEL_HEIGHT: u32 = 220;
const COMPARISON_PADDING: u32 = 24;

struct InternalFaceEmbeddingRun {
    input_path: PathBuf,
    detected_face: AlignedFaceDetection,
    detected_face_count: usize,
    run: OnnxImageRun,
}

#[derive(Clone, Debug)]
struct AlignedFaceDetection {
    box_rect: FaceDetectionBox,
    landmarks: [(f32, f32); 5],
}

#[derive(Clone, Debug)]
struct YunetCandidate {
    face: AlignedFaceDetection,
    score: f32,
}

#[derive(Clone, Debug)]
struct YunetOutputTensor {
    name: String,
    shape: Vec<i64>,
    data: Vec<f32>,
}

/// 使用默认模型和默认输出目录比较两张人脸图片的 embedding 相似度。
///
/// # Errors
/// 图片读取、模型加载、推理、相似度计算或输出文件写入失败时返回错误。
pub fn compare_face_images(
    probe_image_path: impl AsRef<Path>,
    reference_image_path: impl AsRef<Path>,
) -> anyhow::Result<FaceRecognitionRun> {
    let workspace_root = workspace_root()?;
    compare_face_images_with_output(
        probe_image_path,
        reference_image_path,
        workspace_root.join(DEFAULT_RESULT_DIR),
    )
}

/// 使用默认模型和指定输出目录比较两张人脸图片的 embedding 相似度。
///
/// # Errors
/// 图片读取、模型加载、推理、相似度计算或输出文件写入失败时返回错误。
pub fn compare_face_images_with_output(
    probe_image_path: impl AsRef<Path>,
    reference_image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> anyhow::Result<FaceRecognitionRun> {
    compare_face_images_with_threshold(
        probe_image_path,
        reference_image_path,
        output_dir,
        DEFAULT_SAME_IDENTITY_THRESHOLD,
    )
}

/// 使用指定阈值比较两张人脸图片的 embedding 相似度。
///
/// # Errors
/// 图片读取、模型加载、推理、相似度计算或输出文件写入失败时返回错误。
pub fn compare_face_images_with_threshold(
    probe_image_path: impl AsRef<Path>,
    reference_image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    same_identity_threshold: f32,
) -> anyhow::Result<FaceRecognitionRun> {
    if !same_identity_threshold.is_finite() {
        bail!("same identity threshold must be finite");
    }

    let output_dir = output_dir.as_ref().to_path_buf();
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir `{}`", output_dir.display()))?;

    let model_path = FACE_RECOGNITION_SFACE_2021DEC
        .require_local_path(crate_root().join(DEFAULT_MODEL_RESOURCE_DIR))?;
    let mut session = LocalOnnxSession::from_file(&model_path)?;
    let probe_run = extract_face_embedding(
        "probe",
        probe_image_path,
        &mut session,
        output_dir.join("probe"),
    )?;
    let reference_run = extract_face_embedding(
        "reference",
        reference_image_path,
        &mut session,
        output_dir.join("reference"),
    )?;

    let probe_embedding = embedding_from_outputs("probe", &probe_run.run.raw_outputs)?;
    let reference_embedding =
        embedding_from_outputs("reference", &reference_run.run.raw_outputs)?;
    let cosine_similarity = cosine_similarity(&probe_embedding, &reference_embedding)?;
    let same_identity = cosine_similarity >= same_identity_threshold;

    let files = FaceRecognitionOutputFiles {
        probe: probe_run.run.files.clone(),
        reference: reference_run.run.files.clone(),
        similarity_json: output_dir.join("similarity.json"),
        comparison_image: output_dir.join("comparison.png"),
    };
    let result = FaceRecognitionRun {
        algorithm_code: ALGORITHM_CODE.to_owned(),
        probe: FaceEmbeddingRun {
            input_path: probe_run.input_path,
            detected_face: probe_run.detected_face.box_rect,
            detected_face_count: probe_run.detected_face_count,
            files: probe_run.run.files,
            embedding_dimension: probe_embedding.len(),
            raw_outputs: probe_run.run.raw_outputs,
        },
        reference: FaceEmbeddingRun {
            input_path: reference_run.input_path,
            detected_face: reference_run.detected_face.box_rect,
            detected_face_count: reference_run.detected_face_count,
            files: reference_run.run.files,
            embedding_dimension: reference_embedding.len(),
            raw_outputs: reference_run.run.raw_outputs,
        },
        model_path,
        cosine_similarity,
        same_identity_threshold,
        same_identity,
        files,
    };

    write_comparison_image(&result)?;
    let json = serde_json::to_string_pretty(&result)?;
    fs::write(&result.files.similarity_json, json).with_context(|| {
        format!(
            "failed to write face similarity JSON `{}`",
            result.files.similarity_json.display()
        )
    })?;
    Ok(result)
}

fn extract_face_embedding(
    label: &str,
    image_path: impl AsRef<Path>,
    session: &mut LocalOnnxSession,
    output_dir: PathBuf,
) -> anyhow::Result<InternalFaceEmbeddingRun> {
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir `{}`", output_dir.display()))?;
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .with_context(|| format!("failed to resolve `{}`", image_path.as_ref().display()))?;
    let image = image::open(&image_path)
        .with_context(|| format!("failed to read `{}`", image_path.display()))?;
    let detection = detect_aligned_faces_from_image(&image, output_dir.join("face_detection"))?;
    let detected_face = select_primary_face(label, &detection.faces)?;
    let face_crop = align_face_crop(&image, &detected_face)?;
    let prepared = prepare_sface_tensor(&face_crop);
    let summary = session.run_f32(
        &prepared.shape,
        prepared
            .f32_tensor_data()
            .ok_or_else(|| anyhow!("{label} prepared tensor is missing f32 data"))?
            .to_vec(),
    )?;
    let files = write_inference_artifacts_from_image(
        ALGORITHM_CODE,
        &FACE_RECOGNITION_SFACE_2021DEC,
        &DynamicImage::ImageRgb8(face_crop),
        &prepared,
        &summary,
        &output_dir,
    )?;
    let run = OnnxImageRun {
        input_path: image_path.clone(),
        model_path: summary.model_path.clone(),
        files,
        raw_outputs: summary.outputs,
    };

    Ok(InternalFaceEmbeddingRun {
        input_path: image_path,
        detected_face,
        detected_face_count: detection.faces.len(),
        run,
    })
}

fn detect_aligned_faces_from_image(
    image: &DynamicImage,
    output_dir: PathBuf,
) -> anyhow::Result<AlignedFaceDetectionRun> {
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir `{}`", output_dir.display()))?;
    let prepared = prepare_yunet_tensor(image);
    let model_path = crate_root()
        .join("resources/face_detection/models")
        .join(YUNET_MODEL_FILE_NAME);
    if !model_path.is_file() {
        bail!("model file `{}` is missing", model_path.display());
    }
    let mut session = LocalOnnxSession::from_file(&model_path)?;
    let summary = session.run_f32(YUNET_MODEL_INPUT_SHAPE, prepared.tensor_data)?;
    let tensors = yunet_output_tensors(summary)?;
    let faces = decode_yunet_faces(
        &tensors,
        YUNET_MODEL_INPUT_WIDTH,
        YUNET_MODEL_INPUT_HEIGHT,
        image.width(),
        image.height(),
        YUNET_SCORE_THRESHOLD,
        YUNET_NMS_THRESHOLD,
    )?;
    write_yunet_detection_artifacts(image, &prepared.preview, &faces, &output_dir)?;
    Ok(AlignedFaceDetectionRun { faces })
}

struct AlignedFaceDetectionRun {
    faces: Vec<AlignedFaceDetection>,
}

struct PreparedYunetImage {
    preview: RgbImage,
    tensor_data: Vec<f32>,
}

fn prepare_yunet_tensor(image: &DynamicImage) -> PreparedYunetImage {
    let preview = image
        .resize_exact(
            YUNET_MODEL_INPUT_WIDTH,
            YUNET_MODEL_INPUT_HEIGHT,
            FilterType::Triangle,
        )
        .to_rgb8();
    PreparedYunetImage {
        tensor_data: rgb_to_raw_nchw_f32(&preview),
        preview,
    }
}

fn rgb_to_raw_nchw_f32(image: &RgbImage) -> Vec<f32> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![0.0; channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = f32::from(pixel[2]);
        data[channel_len + index] = f32::from(pixel[1]);
        data[channel_len * 2 + index] = f32::from(pixel[0]);
    }
    data
}

fn yunet_output_tensors(summary: OnnxInferenceSummary) -> anyhow::Result<Vec<YunetOutputTensor>> {
    summary
        .outputs
        .into_iter()
        .map(|output| {
            let Some(data) = output.full_f32 else {
                bail!(
                    "YuNet output `{}` did not retain complete f32 tensor",
                    output.name
                );
            };
            Ok(YunetOutputTensor {
                name: output.name,
                shape: output.shape,
                data,
            })
        })
        .collect()
}

fn decode_yunet_faces(
    outputs: &[YunetOutputTensor],
    input_width: u32,
    input_height: u32,
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
    nms_threshold: f32,
) -> anyhow::Result<Vec<AlignedFaceDetection>> {
    let mut candidates = Vec::new();
    for stride in YUNET_STRIDES {
        let class_scores = require_yunet_output(outputs, &format!("cls_{stride}"))?;
        let object_scores = require_yunet_output(outputs, &format!("obj_{stride}"))?;
        let boxes = require_yunet_output(outputs, &format!("bbox_{stride}"))?;
        let keypoints = require_yunet_output(outputs, &format!("kps_{stride}"))?;
        decode_yunet_stride(
            class_scores,
            object_scores,
            boxes,
            keypoints,
            YunetGeometry {
                stride,
                input_width,
                input_height,
                image_width,
                image_height,
            },
            score_threshold,
            &mut candidates,
        )?;
    }
    Ok(yunet_non_maximum_suppression(candidates, nms_threshold)
        .into_iter()
        .map(|candidate| candidate.face)
        .collect())
}

struct YunetGeometry {
    stride: usize,
    input_width: u32,
    input_height: u32,
    image_width: u32,
    image_height: u32,
}

fn decode_yunet_stride(
    class_scores: &YunetOutputTensor,
    object_scores: &YunetOutputTensor,
    boxes: &YunetOutputTensor,
    keypoints: &YunetOutputTensor,
    geometry: YunetGeometry,
    score_threshold: f32,
    candidates: &mut Vec<YunetCandidate>,
) -> anyhow::Result<()> {
    let feature_width = geometry.input_width as usize / geometry.stride;
    let feature_height = geometry.input_height as usize / geometry.stride;
    let expected_count = feature_width * feature_height;
    validate_yunet_tensor(class_scores, expected_count, 1)?;
    validate_yunet_tensor(object_scores, expected_count, 1)?;
    validate_yunet_tensor(boxes, expected_count, 4)?;
    validate_yunet_tensor(keypoints, expected_count, 10)?;

    let x_scale = geometry.image_width as f32 / geometry.input_width as f32;
    let y_scale = geometry.image_height as f32 / geometry.input_height as f32;
    let stride = geometry.stride as f32;
    for row in 0..feature_height {
        for column in 0..feature_width {
            let index = row * feature_width + column;
            let class_score = class_scores.data[index].clamp(0.0, 1.0);
            let object_score = object_scores.data[index].clamp(0.0, 1.0);
            let score = (class_score * object_score).sqrt();
            if score < score_threshold {
                continue;
            }
            let box_index = index * 4;
            let center_x = (column as f32 + boxes.data[box_index]) * stride;
            let center_y = (row as f32 + boxes.data[box_index + 1]) * stride;
            let width = boxes.data[box_index + 2].exp() * stride;
            let height = boxes.data[box_index + 3].exp() * stride;
            let x_min = (center_x - width * 0.5) * x_scale;
            let y_min = (center_y - height * 0.5) * y_scale;
            let x_max = (center_x + width * 0.5) * x_scale;
            let y_max = (center_y + height * 0.5) * y_scale;

            let mut landmarks = [(0.0_f32, 0.0_f32); 5];
            let keypoint_index = index * 10;
            for point in 0..5 {
                landmarks[point] = (
                    (keypoints.data[keypoint_index + point * 2] + column as f32) * stride * x_scale,
                    (keypoints.data[keypoint_index + point * 2 + 1] + row as f32) * stride * y_scale,
                );
            }
            candidates.push(YunetCandidate {
                face: AlignedFaceDetection {
                    box_rect: FaceDetectionBox {
                        x_min: x_min.clamp(0.0, geometry.image_width as f32),
                        y_min: y_min.clamp(0.0, geometry.image_height as f32),
                        x_max: x_max.clamp(0.0, geometry.image_width as f32),
                        y_max: y_max.clamp(0.0, geometry.image_height as f32),
                        confidence: score,
                    },
                    landmarks,
                },
                score,
            });
        }
    }
    Ok(())
}

fn validate_yunet_tensor(
    tensor: &YunetOutputTensor,
    expected_count: usize,
    expected_channels: usize,
) -> anyhow::Result<()> {
    if tensor.shape.as_slice() == [1, expected_count as i64, expected_channels as i64]
        && tensor.data.len() == expected_count * expected_channels
    {
        Ok(())
    } else {
        bail!(
            "invalid YuNet output `{}`: expected [1, {}, {}], got {:?}",
            tensor.name,
            expected_count,
            expected_channels,
            tensor.shape
        )
    }
}

fn require_yunet_output<'a>(
    outputs: &'a [YunetOutputTensor],
    output_name: &str,
) -> anyhow::Result<&'a YunetOutputTensor> {
    outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or_else(|| anyhow!("missing YuNet output `{}`", output_name))
}

fn yunet_non_maximum_suppression(
    mut candidates: Vec<YunetCandidate>,
    nms_threshold: f32,
) -> Vec<YunetCandidate> {
    candidates.sort_by(|left, right| right.score.total_cmp(&left.score));
    let mut kept = Vec::new();
    for candidate in candidates {
        let overlaps_existing = kept.iter().any(|selected: &YunetCandidate| {
            intersection_over_union(&candidate.face.box_rect, &selected.face.box_rect) > nms_threshold
        });
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
    let union = face_area(left) + face_area(right) - intersection;
    if union <= 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn face_area(face: &FaceDetectionBox) -> f32 {
    let width = (face.x_max - face.x_min).max(0.0);
    let height = (face.y_max - face.y_min).max(0.0);
    width * height
}

fn select_primary_face(
    label: &str,
    faces: &[AlignedFaceDetection],
) -> anyhow::Result<AlignedFaceDetection> {
    faces
        .iter()
        .max_by(|left, right| {
            face_rank(*left)
                .partial_cmp(&face_rank(*right))
                .unwrap_or(Ordering::Equal)
        })
        .cloned()
        .ok_or_else(|| anyhow!("{label} image did not contain a detected face"))
}

fn face_rank(face: &AlignedFaceDetection) -> f32 {
    face_area(&face.box_rect) * face.box_rect.confidence.max(0.0)
}

fn align_face_crop(image: &DynamicImage, face: &AlignedFaceDetection) -> anyhow::Result<RgbImage> {
    const TEMPLATE: [(f32, f32); 5] = [
        (38.2946, 51.6963),
        (73.5318, 51.5014),
        (56.0252, 71.7366),
        (41.5493, 92.3655),
        (70.7299, 92.2041),
    ];
    let projection = estimate_similarity_projection(face.landmarks, TEMPLATE)
        .ok_or_else(|| anyhow!("failed to estimate face alignment transform"))?;
    let mut aligned = RgbImage::from_pixel(112, 112, Rgb([0, 0, 0]));
    let source = image.to_rgb8();
    warp_into(
        &source,
        &projection,
        Interpolation::Bilinear,
        Rgb([0, 0, 0]),
        &mut aligned,
    );
    Ok(aligned)
}

fn estimate_similarity_projection(
    from: [(f32, f32); 5],
    to: [(f32, f32); 5],
) -> Option<Projection> {
    let n = from.len() as f32;
    let from_mean = mean_point(&from);
    let to_mean = mean_point(&to);
    let mut a = 0.0_f32;
    let mut b = 0.0_f32;
    let mut denom = 0.0_f32;
    for ((from_x, from_y), (to_x, to_y)) in from.into_iter().zip(to) {
        let x = from_x - from_mean.0;
        let y = from_y - from_mean.1;
        let u = to_x - to_mean.0;
        let v = to_y - to_mean.1;
        a += x * u + y * v;
        b += x * v - y * u;
        denom += x * x + y * y;
    }
    if denom <= f32::EPSILON {
        return None;
    }
    a /= denom;
    b /= denom;
    let tx = to_mean.0 - a * from_mean.0 + b * from_mean.1;
    let ty = to_mean.1 - b * from_mean.0 - a * from_mean.1;
    let _ = n;
    Projection::from_matrix([a, -b, tx, b, a, ty, 0.0, 0.0, 1.0])
}

fn mean_point(points: &[(f32, f32); 5]) -> (f32, f32) {
    let (sum_x, sum_y) = points
        .iter()
        .fold((0.0_f32, 0.0_f32), |(sum_x, sum_y), (x, y)| {
            (sum_x + x, sum_y + y)
        });
    (sum_x / points.len() as f32, sum_y / points.len() as f32)
}

fn prepare_sface_tensor(face_crop: &RgbImage) -> PreparedImageTensor {
    let preview = image::imageops::resize(face_crop, 112, 112, FilterType::Triangle);
    PreparedImageTensor::from_f32_tensor(
        vec![1, 3, 112, 112],
        112,
        112,
        preview.clone(),
        rgb_to_sface_nchw_f32(&preview),
    )
}

fn rgb_to_sface_nchw_f32(image: &RgbImage) -> Vec<f32> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![0.0; channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = f32::from(pixel[2]);
        data[channel_len + index] = f32::from(pixel[1]);
        data[channel_len * 2 + index] = f32::from(pixel[0]);
    }
    data
}

fn embedding_from_outputs(
    label: &str,
    outputs: &[OnnxOutputSummary],
) -> anyhow::Result<Vec<f32>> {
    let output = outputs
        .iter()
        .find(|output| output.name == "fc1")
        .or_else(|| outputs.first())
        .ok_or_else(|| anyhow!("{label} face embedding output is missing"))?;
    let Some(embedding) = &output.full_f32 else {
        bail!(
            "{label} output `{}` did not retain a complete f32 embedding",
            output.name
        );
    };
    if embedding.len() != output.element_count {
        bail!(
            "{label} output `{}` retained {} values but ONNX output has {} elements",
            output.name,
            embedding.len(),
            output.element_count
        );
    }
    Ok(embedding.clone())
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> anyhow::Result<f32> {
    if left.len() != right.len() {
        bail!(
            "face embeddings have different dimensions: {} vs {}",
            left.len(),
            right.len()
        );
    }
    if left.is_empty() {
        bail!("face embeddings must not be empty");
    }

    let (dot, left_norm_squared, right_norm_squared) = left.iter().zip(right).fold(
        (0.0_f32, 0.0_f32, 0.0_f32),
        |(dot, left_norm, right_norm), (left_value, right_value)| {
            (
                dot + left_value * right_value,
                left_norm + left_value * left_value,
                right_norm + right_value * right_value,
            )
        },
    );
    if left_norm_squared <= 0.0 || right_norm_squared <= 0.0 {
        bail!("face embeddings must have non-zero norm");
    }

    let similarity = dot / (left_norm_squared.sqrt() * right_norm_squared.sqrt());
    if similarity.is_finite() {
        Ok(similarity)
    } else {
        bail!("face cosine similarity is not finite")
    }
}

fn write_comparison_image(result: &FaceRecognitionRun) -> anyhow::Result<()> {
    let mut canvas =
        RgbImage::from_pixel(COMPARISON_WIDTH, COMPARISON_IMAGE_HEIGHT, Rgb([248, 250, 252]));
    draw_text_lines(
        &mut canvas,
        24,
        18,
        &["FACE_RECOGNITION", "two-image identity comparison"],
        20.0,
        Rgb([15, 23, 42]),
    );

    let panel_width = (COMPARISON_WIDTH - COMPARISON_PADDING * 3) / 2;
    let panel_y = 72_i32;
    draw_image_panel(
        &mut canvas,
        &result.probe.input_path,
        &result.probe.detected_face,
        result.probe.detected_face_count,
        COMPARISON_PADDING as i32,
        panel_y,
        panel_width,
        COMPARISON_PANEL_HEIGHT,
        "probe",
    )?;
    draw_image_panel(
        &mut canvas,
        &result.reference.input_path,
        &result.reference.detected_face,
        result.reference.detected_face_count,
        (COMPARISON_PADDING * 2 + panel_width) as i32,
        panel_y,
        panel_width,
        COMPARISON_PANEL_HEIGHT,
        "reference",
    )?;

    let result_color = if result.same_identity {
        Rgb([22, 163, 74])
    } else {
        Rgb([220, 38, 38])
    };
    draw_hollow_rect_mut(
        &mut canvas,
        Rect::at(22, 70).of_size(COMPARISON_WIDTH - 44, COMPARISON_PANEL_HEIGHT + 4),
        result_color,
    );

    let annotation = [
        "{".to_owned(),
        format!("  \"algorithm_code\": \"{}\",", result.algorithm_code),
        format!("  \"same_identity\": {},", result.same_identity),
        format!("  \"cosine_similarity\": {:.6},", result.cosine_similarity),
        format!(
            "  \"same_identity_threshold\": {:.6}",
            result.same_identity_threshold
        ),
        "}".to_owned(),
    ];
    let annotation_lines = annotation.iter().map(String::as_str).collect::<Vec<_>>();
    draw_filled_rect_mut(
        &mut canvas,
        Rect::at(24, 306).of_size(COMPARISON_WIDTH - 48, COMPARISON_IMAGE_HEIGHT - 330),
        Rgb([15, 23, 42]),
    );
    draw_text_lines(
        &mut canvas,
        36,
        314,
        &annotation_lines,
        13.0,
        Rgb([255, 255, 255]),
    );

    canvas
        .save(&result.files.comparison_image)
        .with_context(|| {
            format!(
                "failed to write face comparison image `{}`",
                result.files.comparison_image.display()
            )
        })?;
    Ok(())
}

fn write_yunet_detection_artifacts(
    image: &DynamicImage,
    preview: &RgbImage,
    faces: &[AlignedFaceDetection],
    output_dir: &Path,
) -> anyhow::Result<()> {
    let source_input = output_dir.join("source_input.jpg");
    let model_input_preview = output_dir.join("model_input_preview.png");
    let detected_faces_json = output_dir.join("detected_faces.json");
    let detected_faces_image = output_dir.join("detected_faces.png");
    image.save(&source_input)?;
    preview.save(&model_input_preview)?;
    let face_boxes = faces
        .iter()
        .map(|face| &face.box_rect)
        .collect::<Vec<_>>();
    fs::write(
        detected_faces_json,
        serde_json::to_string_pretty(&face_boxes)?,
    )?;
    let mut marked = image.to_rgb8();
    for face in faces {
        draw_face_box_with_landmarks(&mut marked, face);
    }
    marked.save(detected_faces_image)?;
    Ok(())
}

fn draw_face_box_with_landmarks(image: &mut RgbImage, face: &AlignedFaceDetection) {
    draw_face_box(image, &face.box_rect);
    for (x, y) in face.landmarks {
        draw_filled_rect_mut(
            image,
            Rect::at(x.round() as i32 - 2, y.round() as i32 - 2).of_size(5, 5),
            Rgb([34, 197, 94]),
        );
    }
}

fn draw_face_box(image: &mut RgbImage, face: &FaceDetectionBox) {
    let x = face.x_min.round() as i32;
    let y = face.y_min.round() as i32;
    let width = (face.x_max - face.x_min).round().max(1.0) as u32;
    let height = (face.y_max - face.y_min).round().max(1.0) as u32;
    draw_hollow_rect_mut(image, Rect::at(x, y).of_size(width, height), Rgb([255, 0, 0]));
}

fn draw_image_panel(
    canvas: &mut RgbImage,
    input_path: &Path,
    face: &FaceDetectionBox,
    face_count: usize,
    panel_x: i32,
    panel_y: i32,
    panel_width: u32,
    panel_height: u32,
    label: &str,
) -> anyhow::Result<()> {
    let image = image::open(input_path)
        .with_context(|| format!("failed to read `{}`", input_path.display()))?;
    let (source_width, source_height) = image.dimensions();
    let (resized, scaled_width, scaled_height) = resize_to_fit(&image, panel_width, panel_height);
    let image_x = panel_x + ((panel_width - scaled_width) / 2) as i32;
    let image_y = panel_y + ((panel_height - scaled_height) / 2) as i32;

    draw_filled_rect_mut(
        canvas,
        Rect::at(panel_x, panel_y).of_size(panel_width, panel_height),
        Rgb([226, 232, 240]),
    );
    image::imageops::replace(canvas, &resized, i64::from(image_x), i64::from(image_y));
    draw_hollow_rect_mut(
        canvas,
        Rect::at(panel_x, panel_y).of_size(panel_width, panel_height),
        Rgb([71, 85, 105]),
    );

    let scale_x = scaled_width as f32 / source_width.max(1) as f32;
    let scale_y = scaled_height as f32 / source_height.max(1) as f32;
    let box_x = image_x + (face.x_min * scale_x).round() as i32;
    let box_y = image_y + (face.y_min * scale_y).round() as i32;
    let box_width = ((face.x_max - face.x_min) * scale_x).round().max(1.0) as u32;
    let box_height = ((face.y_max - face.y_min) * scale_y).round().max(1.0) as u32;
    for stroke in 0..3_i32 {
        draw_hollow_rect_mut(
            canvas,
            Rect::at(box_x - stroke, box_y - stroke)
                .of_size(box_width + stroke as u32 * 2, box_height + stroke as u32 * 2),
            Rgb([34, 197, 94]),
        );
    }

    draw_filled_rect_mut(
        canvas,
        Rect::at(panel_x, panel_y).of_size(panel_width, 28),
        Rgb([15, 23, 42]),
    );
    draw_text_lines(
        canvas,
        panel_x + 10,
        panel_y + 6,
        &[&format!(
            "{label} faces={face_count} selected_score={:.3}",
            face.confidence
        )],
        14.0,
        Rgb([255, 255, 255]),
    );
    Ok(())
}

fn resize_to_fit(image: &DynamicImage, max_width: u32, max_height: u32) -> (RgbImage, u32, u32) {
    let (source_width, source_height) = image.dimensions();
    let scale = (max_width as f32 / source_width.max(1) as f32)
        .min(max_height as f32 / source_height.max(1) as f32);
    let width = (source_width as f32 * scale).round().clamp(1.0, max_width as f32) as u32;
    let height = (source_height as f32 * scale)
        .round()
        .clamp(1.0, max_height as f32) as u32;
    (
        image
            .resize_exact(width, height, FilterType::Triangle)
            .to_rgb8(),
        width,
        height,
    )
}

fn draw_text_lines(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    lines: &[&str],
    size: f32,
    color: Rgb<u8>,
) {
    let Some(font) = annotation_font() else {
        return;
    };
    let scale = PxScale::from(size);
    let mut cursor_y = y;
    for line in lines {
        draw_text_mut(image, color, x, cursor_y, scale, font, line);
        cursor_y += (size + 3.0).round() as i32;
    }
}

fn annotation_font() -> Option<&'static FontArc> {
    static FONT: OnceLock<Option<FontArc>> = OnceLock::new();
    FONT.get_or_init(load_annotation_font).as_ref()
}

fn load_annotation_font() -> Option<FontArc> {
    [
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
    ]
    .into_iter()
    .find_map(|path| {
        fs::read(path)
            .ok()
            .and_then(|data| FontArc::try_from_vec(data).ok())
    })
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .with_context(|| format!("failed to resolve workspace root from `{}`", env!("CARGO_MANIFEST_DIR")))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
