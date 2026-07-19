//! 火焰与烟雾检测执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::components::flame_detection::model::{
    DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_NMS_THRESHOLD, DEFAULT_RESULT_DIR, DEFAULT_SCORE_THRESHOLD,
    FLAME_DETECTION_FIRE_SMOKE_YOLOV8N, FlameDetectionBox, FlameDetectionClass,
    FlameDetectionOptions, FlameDetectionOutputFiles, FlameDetectionOutputSummary,
    FlameDetectionRun, FlameVideoDetectionOptions, FlameVideoDetectionOutputFiles,
    FlameVideoDetectionRun, FlameVideoFrameDetection,
};
use anyhow::{anyhow, bail};

const OUTPUT_SAMPLE_VALUES: usize = 8;
const YOLO_OUTPUT: &str = "output0";
const YOLO_INPUT_SIZE: f32 = 320.0;

impl FlameDetectionOptions {
    /// 使用当前 workspace 下的默认模型和默认输出目录。
    ///
    /// # Errors
    /// 当前工作目录无法定位或模型文件不存在时返回错误。
    pub fn default_workspace() -> anyhow::Result<Self> {
        let workspace_root = workspace_root()?;
        let model_path = workspace_root
            .join("crates/algorithm/algorithm")
            .join(DEFAULT_MODEL_RESOURCE_DIR)
            .join(FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.local_file);
        if !model_path.is_file() {
            bail!("model file `{}` is missing", (model_path).display());
        }

        Ok(Self {
            model_path,
            output_dir: workspace_root.join(DEFAULT_RESULT_DIR),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
        })
    }
}

/// 可复用的火焰/烟雾检测模型实例。
///
/// 实时视频场景应在启动时构造一次 runner，然后对每帧调用检测方法，避免每帧重复加载 ONNX 模型。
#[derive(Debug)]
pub struct FlameDetectionRunner {
    options: FlameDetectionOptions,
    session: Session,
}

impl FlameDetectionRunner {
    /// 加载火焰检测 ONNX 模型，创建可复用 runner。
    ///
    /// # Errors
    /// 模型文件不存在、阈值非法或 ONNX Runtime 加载模型失败时返回错误。
    pub fn new(options: FlameDetectionOptions) -> anyhow::Result<Self> {
        validate_detection_options(&options)?;
        let mut builder = Session::builder()?;
        let session = builder.commit_from_file(&options.model_path)?;
        Ok(Self { options, session })
    }

    /// 返回 runner 当前使用的配置。
    #[must_use]
    pub const fn options(&self) -> &FlameDetectionOptions {
        &self.options
    }

    /// 对内存中的 RGB 帧执行真实火焰检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_rgb_image_with_output_dir(
        &mut self,
        image: RgbImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<FlameDetectionRun> {
        self.detect_dynamic_image_with_output_dir(DynamicImage::ImageRgb8(image), output_dir)
    }

    /// 对内存中的图片执行真实火焰检测，并把本帧文件写入指定输出目录。
    ///
    /// # Errors
    /// 图片预处理、模型推理或输出文件写入失败时返回错误。
    pub fn detect_dynamic_image_with_output_dir(
        &mut self,
        image: DynamicImage,
        output_dir: impl AsRef<Path>,
    ) -> anyhow::Result<FlameDetectionRun> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)
            .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
        let (preview, inference, detections) = self.detect_image(&image)?;
        let files = write_output_files(
            &image,
            &preview,
            None,
            &detections,
            &inference.summaries,
            output_dir,
        )?;

        Ok(FlameDetectionRun {
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
    ) -> anyhow::Result<(RgbImage, FlameInferenceOutput, Vec<FlameDetectionBox>)> {
        let prepared = prepare_yolo_image(image);
        let inference = run_yolo_session(&mut self.session, prepared.tensor_data)?;
        let detections = decode_yolo_flame_boxes(
            &inference.tensors,
            image.width(),
            image.height(),
            self.options.score_threshold,
            self.options.nms_threshold,
        )?;
        Ok((prepared.preview, inference, detections))
    }
}

/// 传入图片绝对路径执行火焰/烟雾检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_flames_from_path(image_path: impl AsRef<Path>) -> anyhow::Result<FlameDetectionRun> {
    let options = FlameDetectionOptions::default_workspace()?;
    detect_flames_from_path_with_options(image_path, &options)
}

/// 传入图片绝对路径和自定义配置执行火焰/烟雾检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_flames_from_path_with_options(
    image_path: impl AsRef<Path>,
    options: &FlameDetectionOptions,
) -> anyhow::Result<FlameDetectionRun> {
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| path_error(image_path.as_ref().to_path_buf(), source))?;
    let image = image::open(&image_path)?;
    run_detection(image, image_path, options)
}

/// 兼容旧命名：使用默认模型和默认输出目录执行火焰检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_flame_detection_from_path(
    image_path: impl AsRef<Path>,
) -> anyhow::Result<FlameDetectionRun> {
    detect_flames_from_path(image_path)
}

/// 兼容旧命名：使用默认模型和指定输出目录执行火焰检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_flame_detection_from_path_with_output(
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> anyhow::Result<FlameDetectionRun> {
    detect_flames_from_path_with_options(
        image_path,
        &FlameDetectionOptions {
            model_path: crate_root()
                .join(DEFAULT_MODEL_RESOURCE_DIR)
                .join(FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.local_file),
            output_dir: output_dir.as_ref().to_path_buf(),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
        },
    )
}

/// 传入视频绝对路径，抽帧执行真实火焰/烟雾检测，并输出带检测框的标注视频。
///
/// 该接口面向离线文件或网关侧短片段处理。实时 RTSP 场景应复用 [`FlameDetectionRunner`]
/// 对解码后的帧流进行常驻推理，避免先把所有帧落盘。
///
/// # Errors
/// 视频文件、ffmpeg、ONNX 推理、图片处理或输出文件写入失败时返回错误。
pub fn detect_flames_in_video_from_path(
    video_path: impl AsRef<Path>,
    options: &FlameVideoDetectionOptions,
) -> anyhow::Result<FlameVideoDetectionRun> {
    validate_video_options(options)?;
    let video_path = std::fs::canonicalize(video_path.as_ref())
        .map_err(|source| path_error(video_path.as_ref().to_path_buf(), source))?;
    recreate_dir(&options.output_dir)?;

    let files = FlameVideoDetectionOutputFiles {
        source_input_video: options.output_dir.join("source_input.mp4"),
        extracted_frame_dir: options.output_dir.join("extracted_frames"),
        annotated_frame_dir: options.output_dir.join("annotated_frames"),
        frame_detections_json: options.output_dir.join("frame_detections.json"),
        annotated_video: options.output_dir.join("annotated_flames.mp4"),
    };
    fs::create_dir_all(&files.extracted_frame_dir)
        .map_err(|source| path_error(files.extracted_frame_dir.clone(), source))?;
    fs::create_dir_all(&files.annotated_frame_dir)
        .map_err(|source| path_error(files.annotated_frame_dir.clone(), source))?;
    fs::copy(&video_path, &files.source_input_video)
        .map_err(|source| path_error(video_path.clone(), source))?;

    extract_video_frames(&video_path, &files.extracted_frame_dir, options)?;
    let frame_paths = collected_frame_paths(&files.extracted_frame_dir, options.max_frames)?;
    let mut runner = FlameDetectionRunner::new(FlameDetectionOptions {
        model_path: options.model_path.clone(),
        output_dir: options.output_dir.join("per_frame_detection"),
        score_threshold: options.score_threshold,
        nms_threshold: options.nms_threshold,
    })?;
    let mut frames = Vec::new();
    for (frame_index, frame_path) in frame_paths.iter().enumerate() {
        let frame_output_dir = options
            .output_dir
            .join("per_frame_detection")
            .join(format!("frame_{frame_index:05}"));
        let image = image::open(frame_path)?;
        let image_run = runner.detect_dynamic_image_with_output_dir(image, frame_output_dir)?;
        let annotated_frame_path = files
            .annotated_frame_dir
            .join(format!("frame_{frame_index:05}.png"));
        fs::copy(
            &image_run.files.detected_flames_image,
            &annotated_frame_path,
        )
        .map_err(|source| path_error(image_run.files.detected_flames_image.clone(), source))?;
        frames.push(FlameVideoFrameDetection {
            frame_index,
            timestamp_ms: frame_timestamp_ms(frame_index, options.sample_fps),
            frame_path: frame_path.clone(),
            annotated_frame_path,
            detections: image_run.detections,
        });
    }

    let frame_json = serde_json::to_string_pretty(&frames)?;
    fs::write(&files.frame_detections_json, frame_json)
        .map_err(|source| path_error(files.frame_detections_json.clone(), source))?;
    encode_annotated_video(&files.annotated_frame_dir, &files.annotated_video, options)?;

    Ok(FlameVideoDetectionRun {
        input_video_path: video_path,
        model_path: options.model_path.clone(),
        files,
        frames,
    })
}

fn run_detection(
    image: DynamicImage,
    input_path: PathBuf,
    options: &FlameDetectionOptions,
) -> anyhow::Result<FlameDetectionRun> {
    validate_detection_options(options)?;
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;

    let prepared = prepare_yolo_image(&image);
    let inference = run_yolo_model(&options.model_path, prepared.tensor_data)?;
    let detections = decode_yolo_flame_boxes(
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

    Ok(FlameDetectionRun {
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
) -> anyhow::Result<FlameInferenceOutput> {
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
) -> anyhow::Result<FlameInferenceOutput> {
    let input_array = ArrayD::from_shape_vec(
        IxDyn(FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.input.shape),
        tensor_data,
    )
    .map_err(|source| {
        anyhow!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
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
) -> anyhow::Result<FlameInferenceOutput> {
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
        summaries.push(FlameDetectionOutputSummary {
            name: output_name.clone(),
            tensor_type: ty.to_string(),
            shape: shape.iter().copied().collect(),
            element_count: data.len(),
            sample_f32: data.iter().take(OUTPUT_SAMPLE_VALUES).copied().collect(),
        });
        tensors.push(FlameOutputTensor {
            name: output_name,
            shape: shape.iter().copied().collect(),
            data,
        });
    }

    Ok(FlameInferenceOutput { summaries, tensors })
}

fn decode_yolo_flame_boxes(
    outputs: &[FlameOutputTensor],
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
    nms_threshold: f32,
) -> anyhow::Result<Vec<FlameDetectionBox>> {
    let output = require_output(outputs, YOLO_OUTPUT).or_else(|_| {
        outputs
            .first()
            .ok_or_else(|| anyhow!("missing ONNX output `{}`", YOLO_OUTPUT.to_owned(),))
    })?;
    let layout = YoloOutputLayout::parse(output)?;
    let mut boxes = Vec::new();
    let width_scale = image_width as f32 / YOLO_INPUT_SIZE;
    let height_scale = image_height as f32 / YOLO_INPUT_SIZE;
    for candidate_index in 0..layout.candidate_count() {
        let fire_score = layout.value(output, 4, candidate_index);
        let smoke_score = layout.value(output, 5, candidate_index);
        let (class_index, confidence) = if fire_score >= smoke_score {
            (0, fire_score)
        } else {
            (1, smoke_score)
        };
        if confidence < score_threshold {
            continue;
        }
        let Some(detection_class) = FlameDetectionClass::from_class_index(class_index) else {
            continue;
        };

        let center_x = layout.value(output, 0, candidate_index) * width_scale;
        let center_y = layout.value(output, 1, candidate_index) * height_scale;
        let width = layout.value(output, 2, candidate_index) * width_scale;
        let height = layout.value(output, 3, candidate_index) * height_scale;
        boxes.push(FlameDetectionBox {
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

fn write_output_files(
    image: &DynamicImage,
    preview: &RgbImage,
    input_path: Option<&Path>,
    detections: &[FlameDetectionBox],
    summaries: &[FlameDetectionOutputSummary],
    output_dir: &Path,
) -> anyhow::Result<FlameDetectionOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
    let files = FlameDetectionOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        detected_flames_json: output_dir.join("detected_flames.json"),
        detected_flames_image: output_dir.join("detected_flames.png"),
    };

    if let Some(input_path) = input_path {
        fs::copy(input_path, &files.source_input)
            .map_err(|source| path_error(input_path.to_path_buf(), source))?;
    } else {
        image.save(&files.source_input)?;
    }

    let mut marked_preview = preview.clone();
    draw_scaled_flame_boxes(
        &mut marked_preview,
        detections,
        image.width(),
        image.height(),
    );
    marked_preview.save(&files.model_input_preview)?;

    let raw_json = serde_json::to_string_pretty(summaries)?;
    fs::write(&files.raw_outputs_json, raw_json)
        .map_err(|source| path_error(files.raw_outputs_json.clone(), source))?;

    let detections_json = serde_json::to_string_pretty(detections)?;
    fs::write(&files.detected_flames_json, detections_json)
        .map_err(|source| path_error(files.detected_flames_json.clone(), source))?;

    let mut marked_image = image.to_rgb8();
    for detection in detections {
        draw_flame_box(&mut marked_image, detection);
    }
    marked_image.save(&files.detected_flames_image)?;

    Ok(files)
}

fn prepare_yolo_image(image: &DynamicImage) -> PreparedYoloImage {
    let preview = image.resize_exact(320, 320, FilterType::Triangle).to_rgb8();
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

fn draw_scaled_flame_boxes(
    image: &mut RgbImage,
    detections: &[FlameDetectionBox],
    source_width: u32,
    source_height: u32,
) {
    let scale_x = image.width() as f32 / source_width as f32;
    let scale_y = image.height() as f32 / source_height as f32;
    for detection in detections {
        draw_flame_box(
            image,
            &FlameDetectionBox {
                x_min: detection.x_min * scale_x,
                y_min: detection.y_min * scale_y,
                x_max: detection.x_max * scale_x,
                y_max: detection.y_max * scale_y,
                detection_class: detection.detection_class,
                class_index: detection.class_index,
                confidence: detection.confidence,
            },
        );
    }
}

fn draw_flame_box(image: &mut RgbImage, detection: &FlameDetectionBox) {
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
        y - 34,
        &[&format!(
            "{} {:.2}",
            detection.detection_class.label(),
            detection.confidence
        )],
        color,
    );
}

fn draw_boxed_text_label(image: &mut RgbImage, x: i32, y: i32, lines: &[&str], color: Rgb<u8>) {
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
        '.' => Some(["000", "000", "000", "000", "010"]),
        ' ' => Some(["000", "000", "000", "000", "000"]),
        _ => None,
    }
}

fn detection_color(detection_class: FlameDetectionClass) -> Rgb<u8> {
    match detection_class {
        FlameDetectionClass::Fire => Rgb([255, 45, 35]),
        FlameDetectionClass::Smoke => Rgb([96, 165, 250]),
    }
}

fn non_maximum_suppression(
    mut boxes: Vec<FlameDetectionBox>,
    nms_threshold: f32,
) -> Vec<FlameDetectionBox> {
    boxes.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));

    let mut kept: Vec<FlameDetectionBox> = Vec::new();
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

fn intersection_over_union(left: &FlameDetectionBox, right: &FlameDetectionBox) -> f32 {
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

fn area(detection: &FlameDetectionBox) -> f32 {
    let width = (detection.x_max - detection.x_min).max(0.0);
    let height = (detection.y_max - detection.y_min).max(0.0);
    width * height
}

fn require_output<'a>(
    outputs: &'a [FlameOutputTensor],
    output_name: &str,
) -> anyhow::Result<&'a FlameOutputTensor> {
    outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or_else(|| anyhow!("missing ONNX output `{}`", output_name.to_owned(),))
}

fn validate_detection_options(options: &FlameDetectionOptions) -> anyhow::Result<()> {
    if !options.model_path.is_file() {
        bail!("model file `{}` is missing", options.model_path.display());
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
            "score_threshold must be finite and within 0.0..=1.0"
        );
    }
    if !(0.0..=1.0).contains(&options.nms_threshold) || !options.nms_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
            "nms_threshold must be finite and within 0.0..=1.0"
        );
    }
    Ok(())
}

fn validate_video_options(options: &FlameVideoDetectionOptions) -> anyhow::Result<()> {
    if !options.model_path.is_file() {
        bail!("model file `{}` is missing", options.model_path.display());
    }
    if options.sample_fps == 0 {
        bail!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
            "sample_fps must be greater than 0"
        );
    }
    if options.output_fps == 0 {
        bail!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
            "output_fps must be greater than 0"
        );
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
            "score_threshold must be finite and within 0.0..=1.0"
        );
    }
    if !(0.0..=1.0).contains(&options.nms_threshold) || !options.nms_threshold.is_finite() {
        bail!(
            "invalid tensor shape for `{}`: {}",
            FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
            "nms_threshold must be finite and within 0.0..=1.0"
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
    options: &FlameVideoDetectionOptions,
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
    options: &FlameVideoDetectionOptions,
) -> anyhow::Result<()> {
    run_ffmpeg(
        &options.ffmpeg_path,
        &[
            "-y".to_owned(),
            "-framerate".to_owned(),
            options.output_fps.to_string(),
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

#[derive(Clone, Debug)]
struct PreparedYoloImage {
    preview: RgbImage,
    tensor_data: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
struct FlameOutputTensor {
    name: String,
    shape: Vec<i64>,
    data: Vec<f32>,
}

#[derive(Clone, Debug)]
struct FlameInferenceOutput {
    summaries: Vec<FlameDetectionOutputSummary>,
    tensors: Vec<FlameOutputTensor>,
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
    fn parse(output: &FlameOutputTensor) -> anyhow::Result<Self> {
        let [1, second, third] = output.shape.as_slice() else {
            bail!(
                "invalid tensor shape for `{}`: output `{}` expected rank-3 YOLO tensor, got {:?}",
                FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
                output.name,
                output.shape
            );
        };
        let second = usize::try_from(*second).unwrap_or_default();
        let third = usize::try_from(*third).unwrap_or_default();
        if output.data.len() != second.saturating_mul(third) {
            bail!(
                "invalid tensor shape for `{}`: output `{}` shape {:?} does not match {} scalar values",
                FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
                output.name,
                output.shape,
                output.data.len()
            );
        }
        if second >= 6 && third > second {
            Ok(Self::ChannelsFirst {
                channel_count: second,
                candidate_count: third,
            })
        } else if third >= 6 && second > third {
            Ok(Self::CandidatesFirst {
                candidate_count: second,
                channel_count: third,
            })
        } else {
            bail!(
                "invalid tensor shape for `{}`: output `{}` expected YOLO channels containing x,y,w,h,fire,smoke; got {:?}",
                FLAME_DETECTION_FIRE_SMOKE_YOLOV8N.code,
                output.name,
                output.shape
            )
        }
    }

    fn value(self, output: &FlameOutputTensor, channel: usize, candidate_index: usize) -> f32 {
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
