//! 安全帽检测执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ab_glyph::{FontArc, PxScale};
use anyhow::{Context, anyhow, bail};
use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::components::safety_helmet_detection::model::{
    DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_NMS_THRESHOLD, DEFAULT_RESULT_DIR, DEFAULT_SCORE_THRESHOLD,
    SAFETY_HELMET_DETECTION_PPE_YOLO11S, SafetyHelmetDetectionBox,
    SafetyHelmetDetectionClass, SafetyHelmetDetectionOptions, SafetyHelmetDetectionOutputFiles,
    SafetyHelmetDetectionOutputSummary, SafetyHelmetDetectionRun,
};

const OUTPUT_SAMPLE_VALUES: usize = 8;
const YOLO_OUTPUT: &str = "output0";
const YOLO_INPUT_SIZE: f32 = 640.0;
const PPE_CLASS_COUNT: usize = 5;
const PPE_CHANNEL_COUNT: usize = 4 + PPE_CLASS_COUNT;

impl SafetyHelmetDetectionOptions {
    /// 使用当前 workspace 下的默认模型和默认输出目录。
    ///
    /// # Errors
    /// 当前工作目录无法定位或模型文件不存在时返回错误。
    pub fn default_workspace() -> anyhow::Result<Self> {
        let workspace_root = workspace_root()?;
        let model_path = workspace_root
            .join("crates/algorithm/algorithm")
            .join(DEFAULT_MODEL_RESOURCE_DIR)
            .join(SAFETY_HELMET_DETECTION_PPE_YOLO11S.local_file);
        if !model_path.is_file() {
            bail!("model file `{}` is missing", model_path.display());
        }

        Ok(Self {
            model_path,
            output_dir: workspace_root.join(DEFAULT_RESULT_DIR),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
        })
    }
}

/// 可复用的安全帽检测模型实例。
///
/// 实时视频场景应在启动时构造一次 runner，然后对每帧调用检测方法，避免每帧重复加载 ONNX 模型。
#[derive(Debug)]
pub struct SafetyHelmetDetectionRunner {
    options: SafetyHelmetDetectionOptions,
    session: Session,
}

impl SafetyHelmetDetectionRunner {
    /// 加载安全帽检测 ONNX 模型，创建可复用 runner。
    ///
    /// # Errors
    /// 模型文件不存在、阈值非法或 ONNX Runtime 加载失败时返回错误。
    pub fn new(options: SafetyHelmetDetectionOptions) -> anyhow::Result<Self> {
        validate_detection_options(&options)?;
        let mut builder = Session::builder()?;
        let session = builder.commit_from_file(&options.model_path)?;
        Ok(Self { options, session })
    }

    /// 返回 runner 当前使用的配置。
    #[must_use]
    pub const fn options(&self) -> &SafetyHelmetDetectionOptions {
        &self.options
    }

    /// 返回 runner 当前使用的模型路径。
    #[must_use]
    pub fn model_path(&self) -> &Path {
        &self.options.model_path
    }

    /// 对内存中的 RGB 帧执行真实安全帽检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_rgb_image_with_output_dir(
        &mut self,
        image: RgbImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<SafetyHelmetDetectionRun> {
        self.detect_dynamic_image_with_output_dir(DynamicImage::ImageRgb8(image), output_dir)
    }

    /// 对内存中的图片执行真实安全帽检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_dynamic_image_with_output_dir(
        &mut self,
        image: DynamicImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<SafetyHelmetDetectionRun> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir).map_err(|source| path_error(output_dir.to_path_buf(), source))?;
        let (preview, inference, detections) = self.detect_image(&image)?;
        let files = write_output_files(&image, &preview, None, &detections, &inference.summaries, output_dir)?;

        Ok(SafetyHelmetDetectionRun {
            input_path: files.source_input.clone(),
            model_path: self.options.model_path.clone(),
            detections,
            files,
            raw_outputs: inference.summaries,
        })
    }

    fn detect_image(
        &mut self,
        image: &DynamicImage,
    ) -> anyhow::Result<(
        RgbImage,
        SafetyHelmetInferenceOutput,
        Vec<SafetyHelmetDetectionBox>,
    )> {
        let prepared = prepare_yolo_image(image);
        let inference = run_yolo_session(&mut self.session, prepared.tensor_data)?;
        let detections = decode_yolo_safety_helmet_boxes(
            &inference.tensors,
            image.width(),
            image.height(),
            self.options.score_threshold,
            self.options.nms_threshold,
        )?;
        Ok((prepared.preview, inference, detections))
    }
}

/// 使用默认模型和默认输出目录执行安全帽检测。
///
/// # Errors
/// 图片读取、模型加载、推理、后处理或输出文件写入失败时返回错误。
pub fn run_safety_helmet_detection_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<SafetyHelmetDetectionRun> {
    let options = SafetyHelmetDetectionOptions::default_workspace()?;
    detect_safety_helmets_from_path_with_options(image_path, &options)
}

/// 使用默认模型和指定输出目录执行安全帽检测。
///
/// # Errors
/// 图片读取、模型加载、推理、后处理或输出文件写入失败时返回错误。
pub fn run_safety_helmet_detection_from_path_with_output(
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> anyhow::Result<SafetyHelmetDetectionRun> {
    detect_safety_helmets_from_path_with_options(
        image_path,
        &SafetyHelmetDetectionOptions {
            model_path: crate_root()
                .join(DEFAULT_MODEL_RESOURCE_DIR)
                .join(SAFETY_HELMET_DETECTION_PPE_YOLO11S.local_file),
            output_dir: output_dir.as_ref().to_path_buf(),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
        },
    )
}

/// 传入图片绝对路径和自定义配置执行安全帽检测。
///
/// # Errors
/// 图片读取、模型加载、推理、后处理或输出文件写入失败时返回错误。
pub fn detect_safety_helmets_from_path_with_options(
    image_path: impl AsRef<Path>,
    options: &SafetyHelmetDetectionOptions,
) -> anyhow::Result<SafetyHelmetDetectionRun> {
    validate_detection_options(options)?;
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| path_error(image_path.as_ref().to_path_buf(), source))?;
    let image = image::open(&image_path)?;
    run_detection(image, image_path, options)
}

fn run_detection(
    image: DynamicImage,
    input_path: PathBuf,
    options: &SafetyHelmetDetectionOptions,
) -> anyhow::Result<SafetyHelmetDetectionRun> {
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;

    let prepared = prepare_yolo_image(&image);
    let inference = run_yolo_model(&options.model_path, prepared.tensor_data)?;
    let detections = decode_yolo_safety_helmet_boxes(
        &inference.tensors,
        image.width(),
        image.height(),
        options.score_threshold,
        options.nms_threshold,
    )?;
    let files = write_output_files(
        &image,
        &prepared.preview,
        Some(&input_path),
        &detections,
        &inference.summaries,
        &options.output_dir,
    )?;

    Ok(SafetyHelmetDetectionRun {
        input_path,
        model_path: options.model_path.clone(),
        detections,
        files,
        raw_outputs: inference.summaries,
    })
}

fn run_yolo_model(
    model_path: &Path,
    tensor_data: Vec<f32>,
) -> anyhow::Result<SafetyHelmetInferenceOutput> {
    if !model_path.is_file() {
        bail!("model file `{}` is missing", model_path.display());
    }

    let mut builder = Session::builder()?;
    let mut session = builder.commit_from_file(model_path)?;
    run_yolo_session(&mut session, tensor_data)
}

fn run_yolo_session(
    session: &mut Session,
    tensor_data: Vec<f32>,
) -> anyhow::Result<SafetyHelmetInferenceOutput> {
    let input_array = ArrayD::from_shape_vec(
        IxDyn(SAFETY_HELMET_DETECTION_PPE_YOLO11S.input.shape),
        tensor_data,
    )
    .map_err(|source| {
        anyhow!(
            "invalid tensor shape for `{}`: {}",
            SAFETY_HELMET_DETECTION_PPE_YOLO11S.code,
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
) -> anyhow::Result<SafetyHelmetInferenceOutput> {
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
        summaries.push(SafetyHelmetDetectionOutputSummary {
            name: output_name.clone(),
            tensor_type: ty.to_string(),
            shape: shape.iter().copied().collect(),
            element_count: data.len(),
            sample_f32: data.iter().take(OUTPUT_SAMPLE_VALUES).copied().collect(),
        });
        tensors.push(SafetyHelmetOutputTensor {
            name: output_name,
            shape: shape.iter().copied().collect(),
            data,
        });
    }

    Ok(SafetyHelmetInferenceOutput { summaries, tensors })
}

fn decode_yolo_safety_helmet_boxes(
    outputs: &[SafetyHelmetOutputTensor],
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
    nms_threshold: f32,
) -> anyhow::Result<Vec<SafetyHelmetDetectionBox>> {
    let output = require_output(outputs, YOLO_OUTPUT).or_else(|_| {
        outputs
            .first()
            .ok_or_else(|| anyhow!("missing ONNX output `{}`", YOLO_OUTPUT.to_owned()))
    })?;
    let layout = YoloOutputLayout::parse(output)?;
    let mut boxes = Vec::new();
    let width_scale = image_width as f32 / YOLO_INPUT_SIZE;
    let height_scale = image_height as f32 / YOLO_INPUT_SIZE;
    for candidate_index in 0..layout.candidate_count() {
        let Some((class_index, confidence)) = best_class_for_candidate(layout, output, candidate_index)
        else {
            continue;
        };
        if confidence < score_threshold {
            continue;
        }
        let Some(detection_class) = SafetyHelmetDetectionClass::from_class_index(class_index) else {
            continue;
        };

        let center_x = layout.value(output, 0, candidate_index) * width_scale;
        let center_y = layout.value(output, 1, candidate_index) * height_scale;
        let width = layout.value(output, 2, candidate_index) * width_scale;
        let height = layout.value(output, 3, candidate_index) * height_scale;
        boxes.push(SafetyHelmetDetectionBox {
            x_min: (center_x - width / 2.0).clamp(0.0, image_width as f32),
            y_min: (center_y - height / 2.0).clamp(0.0, image_height as f32),
            x_max: (center_x + width / 2.0).clamp(0.0, image_width as f32),
            y_max: (center_y + height / 2.0).clamp(0.0, image_height as f32),
            detection_class,
            class_index,
            confidence,
        });
    }

    Ok(non_maximum_suppression(boxes, nms_threshold))
}

fn best_class_for_candidate(
    layout: YoloOutputLayout,
    output: &SafetyHelmetOutputTensor,
    candidate_index: usize,
) -> Option<(usize, f32)> {
    (0..PPE_CLASS_COUNT)
        .map(|class_index| {
            (
                class_index,
                layout.value(output, 4 + class_index, candidate_index),
            )
        })
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
}

fn write_output_files(
    image: &DynamicImage,
    preview: &RgbImage,
    input_path: Option<&Path>,
    detections: &[SafetyHelmetDetectionBox],
    summaries: &[SafetyHelmetDetectionOutputSummary],
    output_dir: &Path,
) -> anyhow::Result<SafetyHelmetDetectionOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
    let files = SafetyHelmetDetectionOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        detected_safety_helmets_json: output_dir.join("detected_safety_helmets.json"),
        detected_safety_helmets_image: output_dir.join("detected_safety_helmets.png"),
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

    let detection_json = serde_json::to_string_pretty(detections).map_err(|source| {
        path_error(
            files.detected_safety_helmets_json.clone(),
            std::io::Error::other(source.to_string()),
        )
    })?;
    fs::write(&files.detected_safety_helmets_json, detection_json)
        .map_err(|source| path_error(files.detected_safety_helmets_json.clone(), source))?;

    let mut marked_image = image.to_rgb8();
    for detection in detections {
        draw_safety_helmet_box(&mut marked_image, detection);
    }
    marked_image.save(&files.detected_safety_helmets_image)?;

    Ok(files)
}

fn prepare_yolo_image(image: &DynamicImage) -> PreparedYoloImage {
    let preview = image.resize_exact(640, 640, FilterType::Triangle).to_rgb8();
    let tensor_data = rgb_to_nchw_f32_normalized(&preview);
    PreparedYoloImage {
        preview,
        tensor_data,
    }
}

fn rgb_to_nchw_f32_normalized(image: &RgbImage) -> Vec<f32> {
    let channel_len = image.width() as usize * image.height() as usize;
    let mut data = vec![0.0; channel_len * 3];
    for (index, pixel) in image.pixels().enumerate() {
        data[index] = f32::from(pixel[0]) / 255.0;
        data[channel_len + index] = f32::from(pixel[1]) / 255.0;
        data[channel_len * 2 + index] = f32::from(pixel[2]) / 255.0;
    }
    data
}

fn draw_safety_helmet_box(image: &mut RgbImage, detection: &SafetyHelmetDetectionBox) {
    let x = detection.x_min.round() as i32;
    let y = detection.y_min.round() as i32;
    let width = (detection.x_max - detection.x_min).round().max(1.0) as u32;
    let height = (detection.y_max - detection.y_min).round().max(1.0) as u32;
    let color = detection_color(detection.detection_class);
    draw_hollow_rect_mut(image, Rect::at(x, y).of_size(width, height), color);
    draw_hollow_rect_mut(
        image,
        Rect::at(x + 1, y + 1).of_size(
            width.saturating_sub(2).max(1),
            height.saturating_sub(2).max(1),
        ),
        color,
    );
    draw_boxed_text_label(
        image,
        x,
        y - 40,
        &[&format!(
            "{} {:.2}",
            detection_class_chinese_label(detection.detection_class),
            detection.confidence
        )],
        color,
    );
}

fn draw_boxed_text_label(image: &mut RgbImage, x: i32, y: i32, lines: &[&str], color: Rgb<u8>) {
    if let Some(font) = chinese_annotation_font() {
        draw_chinese_boxed_text_label(image, x, y, lines, color, font);
    } else {
        draw_pixel_boxed_text_label(image, x, y, lines, color);
    }
}

fn draw_chinese_boxed_text_label(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    lines: &[&str],
    color: Rgb<u8>,
    font: &FontArc,
) {
    const PADDING: i32 = 6;
    const LINE_GAP: i32 = 4;

    let scale = PxScale::from(24.0);
    let measured_lines = lines
        .iter()
        .map(|line| {
            let (width, height) = text_size(scale, font, line);
            (*line, width as i32, height as i32)
        })
        .collect::<Vec<_>>();
    let label_width = measured_lines
        .iter()
        .map(|(_, width, _)| *width)
        .max()
        .unwrap_or(0)
        + PADDING * 2;
    let label_height = measured_lines
        .iter()
        .map(|(_, _, height)| *height)
        .sum::<i32>()
        + LINE_GAP * (measured_lines.len().saturating_sub(1) as i32)
        + PADDING * 2;
    if label_width <= PADDING * 2 || label_height <= PADDING * 2 {
        return;
    }

    let x = x.min(image.width() as i32 - label_width).max(0);
    let y = y.min(image.height() as i32 - label_height).max(0);
    draw_filled_rect_mut(
        image,
        Rect::at(x, y).of_size(label_width as u32, label_height as u32),
        Rgb([0, 0, 0]),
    );
    draw_hollow_rect_mut(
        image,
        Rect::at(x, y).of_size(label_width as u32, label_height as u32),
        color,
    );

    let mut cursor_y = y + PADDING;
    for (line, _, height) in measured_lines {
        draw_text_mut(image, color, x + PADDING, cursor_y, scale, font, line);
        cursor_y += height + LINE_GAP;
    }
}

fn chinese_annotation_font() -> Option<&'static FontArc> {
    static FONT: OnceLock<Option<FontArc>> = OnceLock::new();
    FONT.get_or_init(load_chinese_annotation_font).as_ref()
}

fn load_chinese_annotation_font() -> Option<FontArc> {
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

fn draw_pixel_boxed_text_label(image: &mut RgbImage, x: i32, y: i32, lines: &[&str], color: Rgb<u8>) {
    const GLYPH_WIDTH: i32 = 3;
    const GLYPH_HEIGHT: i32 = 5;
    const SCALE: i32 = 3;
    const GLYPH_GAP: i32 = 2;
    const PADDING: i32 = 5;
    const LINE_GAP: i32 = 5;

    let visible_chars = lines
        .iter()
        .map(|line| line.chars().count() as i32)
        .max()
        .unwrap_or(0);
    if visible_chars == 0 {
        return;
    }

    let width = visible_chars * (GLYPH_WIDTH * SCALE + GLYPH_GAP) - GLYPH_GAP + PADDING * 2;
    let line_height = GLYPH_HEIGHT * SCALE;
    let height = lines.len() as i32 * line_height
        + lines.len().saturating_sub(1) as i32 * LINE_GAP
        + PADDING * 2;
    let x = x.min(image.width() as i32 - width).max(0);
    let y = y.min(image.height() as i32 - height).max(0);
    draw_filled_rect_mut(
        image,
        Rect::at(x, y).of_size(width as u32, height as u32),
        Rgb([0, 0, 0]),
    );
    draw_hollow_rect_mut(
        image,
        Rect::at(x, y).of_size(width as u32, height as u32),
        color,
    );
    draw_text_label(image, x + PADDING, y + PADDING, lines, Rgb([255, 255, 255]));
}

fn draw_text_label(image: &mut RgbImage, x: i32, y: i32, lines: &[&str], color: Rgb<u8>) {
    const GLYPH_WIDTH: i32 = 3;
    const GLYPH_HEIGHT: i32 = 5;
    const SCALE: i32 = 3;
    const GLYPH_GAP: i32 = 2;
    const LINE_GAP: i32 = 5;

    let mut top = y;
    for line in lines {
        let mut cursor_x = x;
        for ch in line.chars() {
            if let Some(rows) = glyph_rows(ch.to_ascii_uppercase()) {
                draw_glyph(image, cursor_x, top, rows, SCALE, color);
            }
            cursor_x += GLYPH_WIDTH * SCALE + GLYPH_GAP;
        }
        top += GLYPH_HEIGHT * SCALE + LINE_GAP;
    }
}

fn draw_glyph(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    rows: [&'static str; 5],
    scale: i32,
    color: Rgb<u8>,
) {
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, pixel) in row.bytes().enumerate() {
            if pixel == b'1' {
                draw_filled_rect_mut(
                    image,
                    Rect::at(
                        x + column_index as i32 * scale,
                        y + row_index as i32 * scale,
                    )
                    .of_size(scale as u32, scale as u32),
                    color,
                );
            }
        }
    }
}

fn glyph_rows(ch: char) -> Option<[&'static str; 5]> {
    match ch {
        '0' => Some(["111", "101", "101", "101", "111"]),
        '1' => Some(["010", "110", "010", "010", "111"]),
        '2' => Some(["111", "001", "111", "100", "111"]),
        '3' => Some(["111", "001", "111", "001", "111"]),
        '4' => Some(["101", "101", "111", "001", "001"]),
        '5' => Some(["111", "100", "111", "001", "111"]),
        '6' => Some(["111", "100", "111", "101", "111"]),
        '7' => Some(["111", "001", "010", "010", "010"]),
        '8' => Some(["111", "101", "111", "101", "111"]),
        '9' => Some(["111", "101", "111", "001", "111"]),
        'A' => Some(["010", "101", "111", "101", "101"]),
        'B' => Some(["110", "101", "110", "101", "110"]),
        'C' => Some(["111", "100", "100", "100", "111"]),
        'D' => Some(["110", "101", "101", "101", "110"]),
        'E' => Some(["111", "100", "110", "100", "111"]),
        'F' => Some(["111", "100", "110", "100", "100"]),
        'G' => Some(["111", "100", "101", "101", "111"]),
        'H' => Some(["101", "101", "111", "101", "101"]),
        'I' => Some(["111", "010", "010", "010", "111"]),
        'J' => Some(["001", "001", "001", "101", "111"]),
        'K' => Some(["101", "101", "110", "101", "101"]),
        'L' => Some(["100", "100", "100", "100", "111"]),
        'M' => Some(["101", "111", "111", "101", "101"]),
        'N' => Some(["101", "111", "111", "111", "101"]),
        'O' => Some(["111", "101", "101", "101", "111"]),
        'P' => Some(["110", "101", "110", "100", "100"]),
        'Q' => Some(["111", "101", "101", "111", "001"]),
        'R' => Some(["110", "101", "110", "101", "101"]),
        'S' => Some(["111", "100", "111", "001", "111"]),
        'T' => Some(["111", "010", "010", "010", "010"]),
        'U' => Some(["101", "101", "101", "101", "111"]),
        'V' => Some(["101", "101", "101", "101", "010"]),
        'W' => Some(["101", "101", "111", "111", "101"]),
        'X' => Some(["101", "101", "010", "101", "101"]),
        'Y' => Some(["101", "101", "010", "010", "010"]),
        'Z' => Some(["111", "001", "010", "100", "111"]),
        ':' => Some(["000", "010", "000", "010", "000"]),
        '.' => Some(["000", "000", "000", "000", "010"]),
        '-' => Some(["000", "000", "111", "000", "000"]),
        '_' => Some(["000", "000", "000", "000", "111"]),
        ' ' => Some(["000", "000", "000", "000", "000"]),
        _ => None,
    }
}

fn detection_class_chinese_label(detection_class: SafetyHelmetDetectionClass) -> &'static str {
    match detection_class {
        SafetyHelmetDetectionClass::Hardhat => "已戴安全帽",
        SafetyHelmetDetectionClass::NoHardhat => "未戴安全帽",
        SafetyHelmetDetectionClass::Vest => "已穿反光背心",
        SafetyHelmetDetectionClass::NoVest => "未穿反光背心",
        SafetyHelmetDetectionClass::Person => "人员",
    }
}

fn detection_color(detection_class: SafetyHelmetDetectionClass) -> Rgb<u8> {
    match detection_class {
        SafetyHelmetDetectionClass::Hardhat => Rgb([34, 197, 94]),
        SafetyHelmetDetectionClass::NoHardhat => Rgb([239, 68, 68]),
        SafetyHelmetDetectionClass::Vest => Rgb([250, 204, 21]),
        SafetyHelmetDetectionClass::NoVest => Rgb([249, 115, 22]),
        SafetyHelmetDetectionClass::Person => Rgb([59, 130, 246]),
    }
}

fn non_maximum_suppression(
    mut boxes: Vec<SafetyHelmetDetectionBox>,
    nms_threshold: f32,
) -> Vec<SafetyHelmetDetectionBox> {
    boxes.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));

    let mut kept: Vec<SafetyHelmetDetectionBox> = Vec::new();
    for candidate in boxes {
        let overlaps_existing = kept.iter().any(|selected| {
            candidate.detection_class == selected.detection_class
                && intersection_over_union(&candidate, selected) > nms_threshold
        });
        if !overlaps_existing {
            kept.push(candidate);
        }
    }

    kept
}

fn intersection_over_union(
    left: &SafetyHelmetDetectionBox,
    right: &SafetyHelmetDetectionBox,
) -> f32 {
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

fn area(detection: &SafetyHelmetDetectionBox) -> f32 {
    let width = (detection.x_max - detection.x_min).max(0.0);
    let height = (detection.y_max - detection.y_min).max(0.0);
    width * height
}

fn require_output<'a>(
    outputs: &'a [SafetyHelmetOutputTensor],
    output_name: &str,
) -> anyhow::Result<&'a SafetyHelmetOutputTensor> {
    outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or_else(|| anyhow!("missing ONNX output `{}`", output_name.to_owned()))
}

fn validate_detection_options(options: &SafetyHelmetDetectionOptions) -> anyhow::Result<()> {
    if !options.model_path.is_file() {
        bail!("model file `{}` is missing", options.model_path.display());
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        bail!(
            "invalid score_threshold for `{}`: expected 0..=1, got {}",
            SAFETY_HELMET_DETECTION_PPE_YOLO11S.code,
            options.score_threshold
        );
    }
    if !(0.0..=1.0).contains(&options.nms_threshold) || !options.nms_threshold.is_finite() {
        bail!(
            "invalid nms_threshold for `{}`: expected 0..=1, got {}",
            SAFETY_HELMET_DETECTION_PPE_YOLO11S.code,
            options.nms_threshold
        );
    }
    Ok(())
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .with_context(|| format!("failed to resolve workspace root from `{}`", env!("CARGO_MANIFEST_DIR")))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Clone, Debug)]
struct PreparedYoloImage {
    preview: RgbImage,
    tensor_data: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
struct SafetyHelmetOutputTensor {
    name: String,
    shape: Vec<i64>,
    data: Vec<f32>,
}

#[derive(Clone, Debug)]
struct SafetyHelmetInferenceOutput {
    summaries: Vec<SafetyHelmetDetectionOutputSummary>,
    tensors: Vec<SafetyHelmetOutputTensor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum YoloOutputLayout {
    ChannelsFirst {
        channel_count: usize,
        candidate_count: usize,
    },
    CandidatesFirst {
        candidate_count: usize,
        channel_count: usize,
    },
}

impl YoloOutputLayout {
    fn parse(output: &SafetyHelmetOutputTensor) -> anyhow::Result<Self> {
        let [1, second, third] = output.shape.as_slice() else {
            bail!(
                "invalid tensor shape for `{}`: output `{}` expected rank-3 YOLO tensor, got {:?}",
                SAFETY_HELMET_DETECTION_PPE_YOLO11S.code,
                output.name,
                output.shape
            );
        };
        let second = usize::try_from(*second).unwrap_or_default();
        let third = usize::try_from(*third).unwrap_or_default();
        if output.data.len() != second.saturating_mul(third) {
            bail!(
                "invalid tensor shape for `{}`: output `{}` shape {:?} does not match {} scalar values",
                SAFETY_HELMET_DETECTION_PPE_YOLO11S.code,
                output.name,
                output.shape,
                output.data.len()
            );
        }
        if second >= PPE_CHANNEL_COUNT && third > second {
            Ok(Self::ChannelsFirst {
                channel_count: second,
                candidate_count: third,
            })
        } else if third >= PPE_CHANNEL_COUNT && second > third {
            Ok(Self::CandidatesFirst {
                candidate_count: second,
                channel_count: third,
            })
        } else {
            bail!(
                "invalid tensor shape for `{}`: output `{}` expected YOLO channels containing x,y,w,h and {} PPE classes; got {:?}",
                SAFETY_HELMET_DETECTION_PPE_YOLO11S.code,
                output.name,
                PPE_CLASS_COUNT,
                output.shape
            )
        }
    }

    fn value(self, output: &SafetyHelmetOutputTensor, channel: usize, candidate_index: usize) -> f32 {
        match self {
            Self::ChannelsFirst {
                channel_count: _,
                candidate_count,
            } => output.data[channel * candidate_count + candidate_index],
            Self::CandidatesFirst {
                candidate_count: _,
                channel_count,
            } => output.data[candidate_index * channel_count + channel],
        }
    }

    const fn candidate_count(self) -> usize {
        match self {
            Self::ChannelsFirst {
                channel_count: _,
                candidate_count,
            } => candidate_count,
            Self::CandidatesFirst {
                candidate_count,
                channel_count: _,
            } => candidate_count,
        }
    }
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow!("filesystem error at `{}`: {source}", path.display())
}
