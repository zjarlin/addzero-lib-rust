//! OCR 文字识别执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use image::imageops::FilterType;
use image::{DynamicImage, RgbImage};

use az_onnx::onnx::image::assist::LocalOnnxSession;
use az_onnx::onnx::image::model::{OnnxOutputSummary, PreparedImageTensor};

use crate::paddle::model::{
    DEFAULT_DETECTION_RESULT_DIR, DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RECOGNITION_RESULT_DIR,
    OCR_PADDLE_CHINESE_DICT_FILE, OCR_PADDLE_CHINESE_RECOGNITION, OCR_PADDLE_V3_DETECTION,
    OcrTextBoundingBox, OcrTextLine, OcrTextRecognitionOutputFiles, OcrTextToken,
    RECOGNITION_ALGORITHM_CODE,
};

const DETECTION_THRESHOLD: f32 = 0.3;
const MIN_TEXT_REGION_AREA: u32 = 8;
const TEXT_REGION_PADDING: u32 = 4;

#[derive(Clone, Debug, PartialEq)]
struct OcrModelRun {
    input_path: PathBuf,
    model_path: PathBuf,
    raw_outputs: Vec<OnnxOutputSummary>,
}

/// OCR 检测与识别两阶段真实推理结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OcrTextRecognitionRun {
    /// CTC 解码后的 OCR 文本。
    pub recognized_text: String,
    /// CTC 解码保留的 token 明细。
    pub tokens: Vec<OcrTextToken>,
    /// 按检测框裁剪后的文本行结果。
    pub lines: Vec<OcrTextLine>,
    /// OCR 文本后处理输出文件。
    pub files: OcrTextRecognitionOutputFiles,
}

/// 使用默认模型和默认输出目录执行 OCR 两阶段真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_ocr_text_recognition_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<OcrTextRecognitionRun> {
    let workspace_root = workspace_root()?;
    run_ocr_text_recognition_from_path_with_output(
        image_path,
        workspace_root.join(DEFAULT_DETECTION_RESULT_DIR),
        workspace_root.join(DEFAULT_RECOGNITION_RESULT_DIR),
    )
}

/// 使用默认模型和指定输出目录执行 OCR 两阶段真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_ocr_text_recognition_from_path_with_output(
    image_path: impl AsRef<Path>,
    detection_output_dir: impl AsRef<Path>,
    recognition_output_dir: impl AsRef<Path>,
) -> anyhow::Result<OcrTextRecognitionRun> {
    let image_path = image_path.as_ref();
    let resource_dir = crate_root().join(DEFAULT_MODEL_RESOURCE_DIR);
    // Kept for API compatibility; OCR no longer exposes detector debug artifacts.
    recreate_dir(detection_output_dir.as_ref())?;
    let detection = run_ocr_detection_model(image_path, &resource_dir)?;
    let recognition_output_dir = recognition_output_dir.as_ref().to_path_buf();
    recreate_dir(&recognition_output_dir)?;
    let source_image = image::open(image_path)
        .with_context(|| format!("failed to open OCR source image `{}`", image_path.display()))?;
    let recognition = run_ocr_recognition_model(
        &source_image,
        Some(image_path),
        &resource_dir,
    )?;
    let full_image_tokens =
        decode_paddle_ocr_ctc_tokens(&recognition.raw_outputs, &resource_dir)?;
    let full_image_text = tokens_to_text(&full_image_tokens);
    let lines = recognize_detected_text_lines(
        image_path,
        &detection,
        &resource_dir,
    )?;
    let recognized_text = if lines.is_empty() {
        full_image_text
    } else {
        lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let tokens = if lines.is_empty() {
        full_image_tokens
    } else {
        lines
            .iter()
            .flat_map(|line| line.tokens.iter().cloned())
            .collect()
    };
    let files = write_recognized_text_outputs(
        &recognition,
        &recognized_text,
        &tokens,
        &lines,
        &recognition_output_dir,
    )?;
    Ok(OcrTextRecognitionRun {
        recognized_text,
        tokens,
        lines,
        files,
    })
}

fn recognize_detected_text_lines(
    image_path: &Path,
    detection: &OcrModelRun,
    resource_dir: &Path,
) -> anyhow::Result<Vec<OcrTextLine>> {
    let image = image::open(image_path)
        .with_context(|| format!("failed to open OCR source image `{}`", image_path.display()))?;
    let boxes = decode_text_line_boxes(&detection.raw_outputs, image.width(), image.height())?;

    boxes
        .into_iter()
        .enumerate()
        .map(|(index, bounding_box)| {
            let crop = crop_text_region(&image, &bounding_box)?;
            let run = run_ocr_recognition_model(&crop, None, resource_dir)?;
            let tokens = decode_paddle_ocr_ctc_tokens(&run.raw_outputs, resource_dir)?;
            Ok(OcrTextLine {
                index,
                text: tokens_to_text(&tokens),
                bounding_box,
                tokens,
            })
        })
        .collect()
}

fn run_ocr_detection_model(
    image_path: impl AsRef<Path>,
    resource_dir: &Path,
) -> anyhow::Result<OcrModelRun> {
    let model_path = OCR_PADDLE_V3_DETECTION.require_local_path(resource_dir)?;
    let image_path = std::fs::canonicalize(image_path.as_ref()).with_context(|| {
        format!(
            "failed to resolve OCR detection image `{}`",
            image_path.as_ref().display()
        )
    })?;
    let image = image::open(&image_path).with_context(|| {
        format!(
            "failed to open OCR detection image `{}`",
            image_path.display()
        )
    })?;
    let mut session = LocalOnnxSession::from_file(&model_path)?;
    let prepared = prepare_paddle_ocr_detection_tensor(&image)?;
    let summary = session.run_f32(
        &prepared.shape,
        prepared
            .f32_tensor_data()
            .context("prepared OCR detection tensor is missing f32 data")?
            .to_vec(),
    )?;
    Ok(OcrModelRun {
        input_path: image_path,
        model_path,
        raw_outputs: summary.outputs,
    })
}

fn run_ocr_recognition_model(
    image: &DynamicImage,
    input_path: Option<&Path>,
    resource_dir: &Path,
) -> anyhow::Result<OcrModelRun> {
    let model_path = OCR_PADDLE_CHINESE_RECOGNITION.require_local_path(resource_dir)?;
    let mut session = LocalOnnxSession::from_file(&model_path)?;
    let prepared = prepare_paddle_ocr_recognition_tensor(image)?;
    let summary = session.run_f32(
        &prepared.shape,
        prepared
            .f32_tensor_data()
            .context("prepared OCR recognition tensor is missing f32 data")?
            .to_vec(),
    )?;
    Ok(OcrModelRun {
        input_path: if let Some(input_path) = input_path {
            std::fs::canonicalize(input_path).with_context(|| {
                format!(
                    "failed to resolve OCR recognition input path `{}`",
                    input_path.display()
                )
            })?
        } else {
            PathBuf::new()
        },
        model_path,
        raw_outputs: summary.outputs,
    })
}

fn prepare_paddle_ocr_detection_tensor(image: &DynamicImage) -> anyhow::Result<PreparedImageTensor> {
    let [1, 3, height, width] = OCR_PADDLE_V3_DETECTION.input.shape else {
        bail!(
            "invalid OCR detection input shape {:?}",
            OCR_PADDLE_V3_DETECTION.input.shape
        );
    };
    let resized = image
        .resize_exact(*width as u32, *height as u32, FilterType::Triangle)
        .to_rgb8();
    let tensor = rgb_to_paddle_ocr_nchw_f32(&resized);
    Ok(PreparedImageTensor::from_f32_tensor(
        OCR_PADDLE_V3_DETECTION.input.shape.to_vec(),
        *width as u32,
        *height as u32,
        resized,
        tensor,
    ))
}

fn prepare_paddle_ocr_recognition_tensor(
    image: &DynamicImage,
) -> anyhow::Result<PreparedImageTensor> {
    let [1, 3, height, width] = OCR_PADDLE_CHINESE_RECOGNITION.input.shape else {
        bail!(
            "invalid OCR recognition input shape {:?}",
            OCR_PADDLE_CHINESE_RECOGNITION.input.shape
        );
    };
    let resized = resize_ocr_line(image, *width as u32, *height as u32);
    let tensor = rgb_to_paddle_ocr_nchw_f32(&resized);
    Ok(PreparedImageTensor::from_f32_tensor(
        OCR_PADDLE_CHINESE_RECOGNITION.input.shape.to_vec(),
        *width as u32,
        *height as u32,
        resized,
        tensor,
    ))
}

fn resize_ocr_line(image: &DynamicImage, target_width: u32, target_height: u32) -> RgbImage {
    let rgb = image.to_rgb8();
    let source_width = rgb.width().max(1);
    let source_height = rgb.height().max(1);
    let resized_width = ((source_width as f32 * target_height as f32 / source_height as f32)
        .ceil() as u32)
        .clamp(1, target_width);
    let resized = image::imageops::resize(&rgb, resized_width, target_height, FilterType::Triangle);
    let mut canvas = RgbImage::from_pixel(target_width, target_height, image::Rgb([255, 255, 255]));
    image::imageops::replace(&mut canvas, &resized, 0, 0);
    canvas
}

fn rgb_to_paddle_ocr_nchw_f32(image: &RgbImage) -> Vec<f32> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![0.0; channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = normalize_paddle_ocr_pixel(pixel[0]);
        data[channel_len + index] = normalize_paddle_ocr_pixel(pixel[1]);
        data[channel_len * 2 + index] = normalize_paddle_ocr_pixel(pixel[2]);
    }
    data
}

fn normalize_paddle_ocr_pixel(value: u8) -> f32 {
    (f32::from(value) / 255.0 - 0.5) / 0.5
}

fn decode_text_line_boxes(
    outputs: &[OnnxOutputSummary],
    source_width: u32,
    source_height: u32,
) -> anyhow::Result<Vec<OcrTextBoundingBox>> {
    let output = outputs
        .iter()
        .find(|output| output.name == "fetch_name_0")
        .or_else(|| outputs.first())
        .context("OCR detection output is missing")?;
    let heatmap = output.full_f32.as_deref().with_context(|| {
        format!(
            "OCR detection output `{}` did not retain complete f32 heatmap",
            output.name
        )
    })?;
    let (heatmap_width, heatmap_height) = detection_heatmap_shape(output)?;
    if heatmap.len() != heatmap_width * heatmap_height {
        bail!(
            "OCR detection output `{}` shape {:?} requires {} values but has {}",
            output.name,
            output.shape,
            heatmap_width * heatmap_height,
            heatmap.len()
        );
    }

    let mut row_ranges = active_row_ranges(heatmap, heatmap_width, heatmap_height);
    if row_ranges.is_empty() {
        return Ok(Vec::new());
    }

    let scale_x = source_width as f32 / heatmap_width as f32;
    let scale_y = source_height as f32 / heatmap_height as f32;
    let mut boxes = Vec::new();
    for (start_y, end_y) in row_ranges.drain(..) {
        let Some((start_x, end_x, score)) =
            active_column_range(heatmap, heatmap_width, start_y, end_y)
        else {
            continue;
        };
        let x_min = scaled_lower(start_x, scale_x, TEXT_REGION_PADDING);
        let y_min = scaled_lower(start_y, scale_y, TEXT_REGION_PADDING);
        let x_max = scaled_upper(end_x, scale_x, TEXT_REGION_PADDING, source_width);
        let y_max = scaled_upper(end_y, scale_y, TEXT_REGION_PADDING, source_height);
        if x_max > x_min && y_max > y_min {
            boxes.push(OcrTextBoundingBox {
                x_min,
                y_min,
                x_max,
                y_max,
                score,
            });
        }
    }
    Ok(boxes)
}

fn detection_heatmap_shape(output: &OnnxOutputSummary) -> anyhow::Result<(usize, usize)> {
    match output.shape.as_slice() {
        [1, 1, height, width] => Ok((
            positive_dimension(*width, "heatmap width")?,
            positive_dimension(*height, "heatmap height")?,
        )),
        [1, height, width] | [height, width] => Ok((
            positive_dimension(*width, "heatmap width")?,
            positive_dimension(*height, "heatmap height")?,
        )),
        shape => bail!(
            "OCR detection output `{}` expected shape [1, 1, height, width], got {:?}",
            output.name,
            shape
        ),
    }
}

fn active_row_ranges(heatmap: &[f32], width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = None;
    for y in 0..height {
        let active_count = (0..width)
            .filter(|x| heatmap[y * width + x] >= DETECTION_THRESHOLD)
            .count() as u32;
        let active = active_count >= MIN_TEXT_REGION_AREA;
        match (start, active) {
            (None, true) => start = Some(y),
            (Some(start_y), false) => {
                if y.saturating_sub(start_y) as u32 >= MIN_TEXT_REGION_AREA {
                    ranges.push((start_y, y));
                }
                start = None;
            }
            _ => {}
        }
    }
    if let Some(start_y) = start
        && height.saturating_sub(start_y) as u32 >= MIN_TEXT_REGION_AREA
    {
        ranges.push((start_y, height));
    }
    ranges
}

fn active_column_range(
    heatmap: &[f32],
    width: usize,
    start_y: usize,
    end_y: usize,
) -> Option<(usize, usize, f32)> {
    let mut min_x = width;
    let mut max_x = 0_usize;
    let mut score_sum = 0.0_f32;
    let mut score_count = 0_usize;
    for y in start_y..end_y {
        for x in 0..width {
            let score = heatmap[y * width + x];
            if score >= DETECTION_THRESHOLD {
                min_x = min_x.min(x);
                max_x = max_x.max(x + 1);
                score_sum += score;
                score_count += 1;
            }
        }
    }
    (score_count > 0).then(|| (min_x, max_x, score_sum / score_count as f32))
}

fn scaled_lower(value: usize, scale: f32, padding: u32) -> u32 {
    ((value as f32 * scale).floor() as u32).saturating_sub(padding)
}

fn scaled_upper(value: usize, scale: f32, padding: u32, limit: u32) -> u32 {
    (((value as f32 * scale).ceil() as u32).saturating_add(padding)).min(limit)
}

fn crop_text_region(
    image: &DynamicImage,
    bounding_box: &OcrTextBoundingBox,
) -> anyhow::Result<DynamicImage> {
    let width = bounding_box.x_max.saturating_sub(bounding_box.x_min);
    let height = bounding_box.y_max.saturating_sub(bounding_box.y_min);
    if width == 0 || height == 0 {
        bail!("OCR text crop must have positive dimensions: {bounding_box:?}");
    }
    Ok(image.crop_imm(bounding_box.x_min, bounding_box.y_min, width, height))
}

fn decode_paddle_ocr_ctc_tokens(
    outputs: &[OnnxOutputSummary],
    resource_dir: &Path,
) -> anyhow::Result<Vec<OcrTextToken>> {
    let output = outputs
        .iter()
        .find(|output| output.name == "fetch_name_0")
        .or_else(|| outputs.first())
        .context("OCR recognition output is missing")?;
    let class_labels = read_paddle_ocr_labels(resource_dir)?;
    let tensor = output.full_f32.as_deref().with_context(|| {
        format!(
            "OCR recognition output `{}` did not retain complete f32 logits",
            output.name
        )
    })?;
    let (time_steps, class_count) = recognition_time_class_shape(output)?;
    if tensor.len() != time_steps * class_count {
        bail!(
            "OCR recognition output `{}` shape {:?} requires {} values but has {}",
            output.name,
            output.shape,
            time_steps * class_count,
            tensor.len()
        );
    }

    let mut tokens = Vec::new();
    let mut previous_class = None;
    for time_step in 0..time_steps {
        let offset = time_step * class_count;
        let (class_index, score) = argmax(&tensor[offset..offset + class_count])?;
        if class_index != 0 && previous_class != Some(class_index) {
            if let Some(token) = class_labels.get(class_index) {
                tokens.push(OcrTextToken {
                    time_step,
                    class_index,
                    token: token.clone(),
                    score,
                });
            }
        }
        previous_class = Some(class_index);
    }
    Ok(tokens)
}

fn read_paddle_ocr_labels(resource_dir: &Path) -> anyhow::Result<Vec<String>> {
    let dict_path = resource_dir.join(OCR_PADDLE_CHINESE_DICT_FILE);
    let content = fs::read_to_string(&dict_path)
        .with_context(|| format!("failed to read OCR dictionary `{}`", dict_path.display()))?;
    let mut labels = Vec::with_capacity(content.lines().count() + 2);
    labels.push(String::new());
    labels.extend(content.lines().map(ToOwned::to_owned));
    labels.push(" ".to_owned());
    Ok(labels)
}

fn recognition_time_class_shape(output: &OnnxOutputSummary) -> anyhow::Result<(usize, usize)> {
    match output.shape.as_slice() {
        [1, time_steps, class_count] => Ok((
            positive_dimension(*time_steps, "time")?,
            positive_dimension(*class_count, "class")?,
        )),
        [time_steps, class_count] => Ok((
            positive_dimension(*time_steps, "time")?,
            positive_dimension(*class_count, "class")?,
        )),
        shape => bail!(
            "OCR recognition output `{}` expected shape [1, time, class] or [time, class], got {:?}",
            output.name,
            shape
        ),
    }
}

fn positive_dimension(value: i64, label: &str) -> anyhow::Result<usize> {
    if value <= 0 {
        bail!("OCR recognition {label} dimension must be positive, got {value}");
    }
    Ok(value as usize)
}

fn argmax(values: &[f32]) -> anyhow::Result<(usize, f32)> {
    values
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| value.is_finite())
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .context("OCR recognition timestep has no finite class score")
}

fn write_recognized_text_outputs(
    recognition: &OcrModelRun,
    recognized_text: &str,
    tokens: &[OcrTextToken],
    lines: &[OcrTextLine],
    recognition_output_dir: &Path,
) -> anyhow::Result<OcrTextRecognitionOutputFiles> {
    fs::create_dir_all(recognition_output_dir).with_context(|| {
        format!(
            "failed to create OCR recognition output dir `{}`",
            recognition_output_dir.display()
        )
    })?;
    let files = OcrTextRecognitionOutputFiles {
        recognized_text: recognition_output_dir.join("recognized_text.txt"),
        recognized_text_json: recognition_output_dir.join("recognized_text.json"),
    };
    fs::write(&files.recognized_text, recognized_text).with_context(|| {
        format!(
            "failed to write OCR recognized text `{}`",
            files.recognized_text.display()
        )
    })?;
    let json = serde_json::json!({
        "algorithm_code": RECOGNITION_ALGORITHM_CODE,
        "input_path": recognition.input_path,
        "model_path": recognition.model_path,
        "recognized_text": recognized_text,
        "tokens": tokens,
        "lines": lines,
    });
    fs::write(
        &files.recognized_text_json,
        serde_json::to_string_pretty(&json)?,
    )
    .with_context(|| {
        format!(
            "failed to write OCR recognized text JSON `{}`",
            files.recognized_text_json.display()
        )
    })?;
    Ok(files)
}

fn tokens_to_text(tokens: &[OcrTextToken]) -> String {
    tokens
        .iter()
        .map(|token| token.token.as_str())
        .collect::<String>()
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .with_context(|| format!("failed to resolve workspace root from `{}`", env!("CARGO_MANIFEST_DIR")))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn recreate_dir(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove OCR output dir `{}`", path.display()))?;
    }
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create OCR output dir `{}`", path.display()))?;
    Ok(())
}
