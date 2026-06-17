//! 图片 ONNX 推理辅助函数。

use std::fs;
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use anyhow::{anyhow, bail};

use crate::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputFiles, OnnxImageRun, OnnxInferenceSummary,
    OnnxImageOutputKind, OnnxModelMetadata, OnnxOutputSummary, OnnxTensorIoInfo,
    PreparedImageTensor, TensorElementKind, TensorInputSpec,
};

const MAX_OUTPUT_SAMPLE_VALUES: usize = 8;
const REVIEW_WIDTH: u32 = 960;
const REVIEW_HEIGHT: u32 = 620;
const REVIEW_PREVIEW_SIZE: u32 = 224;

/// 已加载的本地 ONNX Runtime 会话。
#[derive(Debug)]
pub struct LocalOnnxSession {
    model_path: PathBuf,
    session: Session,
}

impl LocalOnnxSession {
    /// 将本地 ONNX 模型文件加载进 ONNX Runtime。
    /// # Errors
    /// 当模型文件不存在或 ONNX Runtime 加载失败时返回错误。
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            bail!("ONNX model file not found: `{}`", path.display());
        }

        let mut builder = Session::builder()?;
        let session = builder.commit_from_file(path)?;
        Ok(Self {
            model_path: path.to_path_buf(),
            session,
        })
    }

    /// 返回图输入输出元数据。
    #[must_use]
    pub fn metadata(&self) -> OnnxModelMetadata {
        OnnxModelMetadata {
            model_path: self.model_path.clone(),
            inputs: self.session.inputs().iter().map(outlet_to_info).collect(),
            outputs: self.session.outputs().iter().map(outlet_to_info).collect(),
        }
    }

    /// 使用 f32 张量执行推理。
    ///
    /// # Errors
    /// 当张量形状无效或 ONNX Runtime 拒绝执行时返回错误。
    pub fn run_f32(
        &mut self,
        input_shape: &[usize],
        input_data: Vec<f32>,
    ) -> anyhow::Result<OnnxInferenceSummary> {
        validate_input_len("custom_onnx_model", input_shape, input_data.len())?;
        let input_name = first_input_name(&self.session);
        let output_names = output_names(&self.session);
        let input_array =
            ArrayD::from_shape_vec(IxDyn(input_shape), input_data).map_err(|error| {
                anyhow!("invalid tensor shape for `custom_onnx_model`: {error}")
            })?;
        let input = Tensor::from_array(input_array)?;
        let outputs = self.session.run(ort::inputs![input])?;
        let summaries = output_summaries(&output_names, outputs)?;

        Ok(OnnxInferenceSummary {
            model_path: self.model_path.clone(),
            input_name,
            input_shape: input_shape.to_vec(),
            outputs: summaries,
        })
    }

    /// 使用 u8 张量执行推理。
    ///
    /// # Errors
    /// 当张量形状无效或 ONNX Runtime 拒绝执行时返回错误。
    pub fn run_u8(
        &mut self,
        input_shape: &[usize],
        input_data: Vec<u8>,
    ) -> anyhow::Result<OnnxInferenceSummary> {
        validate_input_len("custom_onnx_model", input_shape, input_data.len())?;
        let input_name = first_input_name(&self.session);
        let output_names = output_names(&self.session);
        let input_array =
            ArrayD::from_shape_vec(IxDyn(input_shape), input_data).map_err(|error| {
                anyhow!("invalid tensor shape for `custom_onnx_model`: {error}")
            })?;
        let input = Tensor::from_array(input_array)?;
        let outputs = self.session.run(ort::inputs![input])?;
        let summaries = output_summaries(&output_names, outputs)?;

        Ok(OnnxInferenceSummary {
            model_path: self.model_path.clone(),
            input_name,
            input_shape: input_shape.to_vec(),
            outputs: summaries,
        })
    }

    /// 按模型规格的输入 layout 对真实图片执行推理。
    ///
    /// # Errors
    /// 当图片加载、预处理或 ONNX Runtime 推理失败时返回错误。
    pub fn run_image_file(
        &mut self,
        spec: &OnnxImageModelSpec,
        image_path: impl AsRef<Path>,
    ) -> anyhow::Result<(PreparedImageTensor, OnnxInferenceSummary)> {
        let prepared = prepare_image_tensor_for_spec(spec, image_path)?;
        let summary = self.run_prepared_image(spec, &prepared)?;
        Ok((prepared, summary))
    }

    /// 按模型规格的输入 layout 对内存图片执行推理。
    ///
    /// # Errors
    /// 当图片预处理或 ONNX Runtime 推理失败时返回错误。
    pub fn run_dynamic_image(
        &mut self,
        spec: &OnnxImageModelSpec,
        image: &DynamicImage,
    ) -> anyhow::Result<(PreparedImageTensor, OnnxInferenceSummary)> {
        let prepared = prepare_dynamic_image_tensor_for_spec(spec, image)?;
        let summary = self.run_prepared_image(spec, &prepared)?;
        Ok((prepared, summary))
    }

    fn run_prepared_image(
        &mut self,
        spec: &OnnxImageModelSpec,
        prepared: &PreparedImageTensor,
    ) -> anyhow::Result<OnnxInferenceSummary> {
        let summary = match prepared.element {
            TensorElementKind::Float32 => self.run_f32(
                &prepared.shape,
                prepared
                    .f32_data
                    .clone()
                    .ok_or_else(|| {
                        anyhow!(
                            "invalid tensor shape for `{}`: prepared image does not contain f32 tensor data",
                            spec.code
                        )
                    })?,
            )?,
            TensorElementKind::Uint8 => self.run_u8(
                &prepared.shape,
                prepared
                    .u8_data
                    .clone()
                    .ok_or_else(|| {
                        anyhow!(
                            "invalid tensor shape for `{}`: prepared image does not contain u8 tensor data",
                            spec.code
                        )
                    })?,
            )?,
        };
        Ok(summary)
    }
}

/// 对真实图片执行本地 ONNX 推理并写出通用输出文件。
///
/// # Errors
/// 当模型、图片、推理或文件写入失败时返回错误。
#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印模型、输入、输出的绝对路径"
)]
pub fn run_real_image_model(
    algorithm_code: &'static str,
    spec: &OnnxImageModelSpec,
    resource_dir: impl AsRef<Path>,
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> anyhow::Result<OnnxImageRun> {
    let model_path = spec.require_local_path(resource_dir)?;
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| path_error(image_path.as_ref().to_path_buf(), source))?;
    let output_dir = output_dir.as_ref().to_path_buf();
    fs::create_dir_all(&output_dir)
        .map_err(|source| path_error(output_dir.clone(), source))?;

    dbg!(&model_path);
    dbg!(&image_path);
    dbg!(&output_dir);

    let mut session = LocalOnnxSession::from_file(&model_path)?;
    let (prepared, summary) = session.run_image_file(spec, &image_path)?;
    let files = write_inference_artifacts(
        algorithm_code,
        spec,
        &image_path,
        &prepared,
        &summary,
        &output_dir,
    )?;

    Ok(OnnxImageRun {
        input_path: image_path,
        model_path,
        files,
        raw_outputs: summary.outputs,
    })
}

/// 将真实图片准备为模型规格声明的 ONNX 输入张量。
///
/// # Errors
/// 当形状不支持或图片无法读取时返回错误。
pub fn prepare_image_tensor_for_spec(
    spec: &OnnxImageModelSpec,
    image_path: impl AsRef<Path>,
) -> anyhow::Result<PreparedImageTensor> {
    let image = image::open(image_path)?;
    prepare_dynamic_image_tensor_for_spec(spec, &image)
}

/// 将内存图片准备为模型规格声明的 ONNX 输入张量。
///
/// # Errors
/// 当形状不支持时返回错误。
pub fn prepare_dynamic_image_tensor_for_spec(
    spec: &OnnxImageModelSpec,
    image: &DynamicImage,
) -> anyhow::Result<PreparedImageTensor> {
    prepare_dynamic_image_tensor(spec.code, spec.input, image)
}

fn prepare_dynamic_image_tensor(
    model_code: &'static str,
    input: TensorInputSpec,
    image: &DynamicImage,
) -> anyhow::Result<PreparedImageTensor> {
    let [batch, first, second, third] = input.shape else {
        bail!(
            "invalid tensor shape for `{}`: expected 4D image input shape, got {:?}",
            model_code,
            input.shape
        );
    };
    if *batch != 1 {
        bail!("invalid tensor shape for `{model_code}`: image tests support batch=1 only, got {batch}");
    }

    let (layout, height, width) = if *first == 3 {
        (ImageTensorLayout::Nchw, *second, *third)
    } else if *third == 3 {
        (ImageTensorLayout::Nhwc, *first, *second)
    } else {
        bail!(
            "invalid tensor shape for `{}`: expected RGB channel dimension, got {:?}",
            model_code,
            input.shape
        );
    };

    let preview = image
        .resize_exact(width as u32, height as u32, FilterType::Triangle)
        .to_rgb8();
    let (f32_data, u8_data) = match input.element {
        TensorElementKind::Float32 => (Some(rgb_to_f32_tensor(&preview, layout)), None),
        TensorElementKind::Uint8 => (None, Some(rgb_to_u8_tensor(&preview, layout))),
    };

    Ok(PreparedImageTensor {
        shape: input.shape.to_vec(),
        element: input.element,
        width: width as u32,
        height: height as u32,
        preview,
        f32_data,
        u8_data,
    })
}

/// 将内存图片推理结果写成通用 ONNX 输出文件。
///
/// # Errors
/// 当文件写入失败时返回错误。
pub fn write_inference_artifacts_from_image(
    algorithm_code: &str,
    spec: &OnnxImageModelSpec,
    source_image: &DynamicImage,
    prepared: &PreparedImageTensor,
    summary: &OnnxInferenceSummary,
    output_dir: &Path,
) -> anyhow::Result<OnnxImageOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| path_error(output_dir.to_path_buf(), source))?;
    let files = OnnxImageOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        raw_output_review: output_dir.join("raw_output_review.png"),
    };

    source_image.save(&files.source_input)?;
    prepared.preview.save(&files.model_input_preview)?;
    let json = serde_json::to_string_pretty(summary)?;
    fs::write(&files.raw_outputs_json, json)
        .map_err(|source| path_error(files.raw_outputs_json.clone(), source))?;
    write_raw_output_review_image(
        &prepared.preview,
        summary,
        &files.raw_output_review,
        algorithm_code,
        spec.output_kind,
    )?;
    assert_real_outputs_exist(algorithm_code, summary);

    Ok(files)
}

#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印输入、输出文件的绝对路径"
)]
fn write_inference_artifacts(
    algorithm_code: &str,
    spec: &OnnxImageModelSpec,
    source_image: &Path,
    prepared: &PreparedImageTensor,
    summary: &OnnxInferenceSummary,
    output_dir: &Path,
) -> anyhow::Result<OnnxImageOutputFiles> {
    let files = OnnxImageOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        raw_output_review: output_dir.join("raw_output_review.png"),
    };

    dbg!(&files.source_input);
    dbg!(&files.model_input_preview);
    dbg!(&files.raw_outputs_json);
    dbg!(&files.raw_output_review);

    prepared.preview.save(&files.model_input_preview)?;
    fs::copy(source_image, &files.source_input)
        .map_err(|source| path_error(source_image.to_path_buf(), source))?;
    let json = serde_json::to_string_pretty(summary)?;
    fs::write(&files.raw_outputs_json, json)
        .map_err(|source| path_error(files.raw_outputs_json.clone(), source))?;
    write_raw_output_review_image(
        &prepared.preview,
        summary,
        &files.raw_output_review,
        algorithm_code,
        spec.output_kind,
    )?;

    assert_real_outputs_exist(algorithm_code, summary);

    Ok(files)
}

fn assert_real_outputs_exist(algorithm_code: &str, summary: &OnnxInferenceSummary) {
    assert!(
        !summary.outputs.is_empty(),
        "{algorithm_code} 必须产生 ONNX 输出，不能只加载模型"
    );
    assert!(
        summary
            .outputs
            .iter()
            .any(|output| output.element_count > 0),
        "{algorithm_code} 至少一个输出张量必须包含真实元素"
    );
}

fn write_raw_output_review_image(
    preview: &RgbImage,
    summary: &OnnxInferenceSummary,
    output_path: &Path,
    algorithm_code: &str,
    output_kind: OnnxImageOutputKind,
) -> anyhow::Result<()> {
    let mut canvas = RgbImage::from_pixel(REVIEW_WIDTH, REVIEW_HEIGHT, Rgb([248, 250, 252]));
    draw_text_label(
        &mut canvas,
        24,
        22,
        &[
            "ONNX RAW OUTPUT REVIEW",
            &format!("ALGO {algorithm_code}"),
            &format!("INPUT {}", join_shape(&summary.input_shape)),
        ],
        Rgb([15, 23, 42]),
    );

    let preview = image::imageops::resize(
        preview,
        REVIEW_PREVIEW_SIZE,
        REVIEW_PREVIEW_SIZE,
        FilterType::Triangle,
    );
    image::imageops::replace(&mut canvas, &preview, 24, 112);
    draw_hollow_rect_mut(
        &mut canvas,
        Rect::at(24, 112).of_size(REVIEW_PREVIEW_SIZE, REVIEW_PREVIEW_SIZE),
        Rgb([15, 23, 42]),
    );
    draw_preview_annotation(&mut canvas, 24, 112, REVIEW_PREVIEW_SIZE, summary, output_kind);

    let mut y = 112_i32;
    for output in &summary.outputs {
        draw_output_summary(&mut canvas, 288, y, output, output_kind);
        y += 128;
        if y > REVIEW_HEIGHT as i32 - 96 {
            break;
        }
    }

    canvas.save(output_path)?;
    Ok(())
}

fn draw_preview_annotation(
    canvas: &mut RgbImage,
    x: i32,
    y: i32,
    size: u32,
    summary: &OnnxInferenceSummary,
    output_kind: OnnxImageOutputKind,
) {
    match output_kind {
        OnnxImageOutputKind::Embedding => {
            draw_text_label(
                canvas,
                x,
                y + size as i32 + 16,
                &["MODEL INPUT PREVIEW", "OUTPUT IS EMBEDDING"],
                Rgb([71, 85, 105]),
            );
            return;
        }
        OnnxImageOutputKind::RawTensor => {
            draw_text_label(
                canvas,
                x,
                y + size as i32 + 16,
                &["MODEL INPUT PREVIEW", "RAW TENSOR ONLY"],
                Rgb([71, 85, 105]),
            );
            return;
        }
        OnnxImageOutputKind::ImageClassification => {}
    }

    let Some((class_index, confidence)) = top_review_sample(summary) else {
        return;
    };

    let color = Rgb([220, 38, 38]);
    let inset = 6_i32;
    for stroke in 0..3_i32 {
        let offset = inset + stroke;
        let side = size.saturating_sub((offset * 2) as u32);
        draw_hollow_rect_mut(
            canvas,
            Rect::at(x + offset, y + offset).of_size(side, side),
            color,
        );
    }
    draw_boxed_text_label(
        canvas,
        x + inset + 4,
        y + inset + 4,
        &[
            "IMAGE LEVEL",
            &format!("TOP {class_index} P {confidence:.3}"),
        ],
        color,
    );
    draw_text_label(
        canvas,
        x,
        y + size as i32 + 16,
        &["IMAGE LEVEL CLASSIFICATION", "BOX MEANS WHOLE IMAGE"],
        Rgb([71, 85, 105]),
    );
}

fn top_review_sample(summary: &OnnxInferenceSummary) -> Option<(usize, f32)> {
    let output = summary
        .outputs
        .iter()
        .find(|output| !output.sample_f32.is_empty())?;
    let normalized = normalized_review_values(
        &output.sample_f32,
        OnnxImageOutputKind::ImageClassification,
    );
    normalized
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
}

fn draw_output_summary(
    canvas: &mut RgbImage,
    x: i32,
    y: i32,
    output: &OnnxOutputSummary,
    output_kind: OnnxImageOutputKind,
) {
    let output_kind_label = match output_kind {
        OnnxImageOutputKind::RawTensor => "RAW TENSOR",
        OnnxImageOutputKind::ImageClassification => "CLASSIFICATION",
        OnnxImageOutputKind::Embedding => "EMBEDDING VECTOR",
    };
    draw_text_label(
        canvas,
        x,
        y,
        &[
            &format!("OUTPUT {}", output.name),
            output_kind_label,
            &format!(
                "TYPE {} SHAPE {} ELEMENTS {}",
                output.tensor_type,
                join_shape(&output.shape),
                output.element_count
            ),
        ],
        Rgb([15, 23, 42]),
    );

    let values = normalized_review_values(&output.sample_f32, output_kind);
    if values.is_empty() {
        draw_text_label(
            canvas,
            x,
            y + 62,
            &["NO SAMPLE VALUES"],
            Rgb([148, 163, 184]),
        );
        return;
    }

    let bar_x = x;
    let bar_y = y + 86;
    let max_bar_width = 560_u32;
    let bar_height = 14_u32;
    for (index, value) in values.iter().enumerate() {
        let current_y = bar_y + index as i32 * 22;
        draw_hollow_rect_mut(
            canvas,
            Rect::at(bar_x, current_y).of_size(max_bar_width, bar_height),
            Rgb([203, 213, 225]),
        );
        let width = (max_bar_width as f32 * value)
            .round()
            .clamp(1.0, max_bar_width as f32) as u32;
        draw_filled_rect_mut(
            canvas,
            Rect::at(bar_x, current_y).of_size(width, bar_height),
            review_bar_color(index),
        );
        draw_text_label(
            canvas,
            bar_x + max_bar_width as i32 + 12,
            current_y - 2,
            &[&format!("{} {:.4}", index, output.sample_f32[index])],
            Rgb([51, 65, 85]),
        );
    }
}

fn normalized_review_values(values: &[f32], output_kind: OnnxImageOutputKind) -> Vec<f32> {
    if values.is_empty() {
        return Vec::new();
    }
    if output_kind == OnnxImageOutputKind::ImageClassification
        && values.iter().all(|value| value.is_finite())
    {
        let max = values
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, |current, value| current.max(value));
        let exp_values = values
            .iter()
            .map(|value| (*value - max).exp())
            .collect::<Vec<_>>();
        let exp_sum = exp_values.iter().sum::<f32>();
        if exp_sum.is_finite() && exp_sum > 0.0 {
            return exp_values
                .into_iter()
                .map(|value| (value / exp_sum).clamp(0.0, 1.0))
                .collect();
        }
    }
    values
        .iter()
        .map(|value| value.abs().clamp(0.0, 1.0))
        .collect()
}

fn review_bar_color(index: usize) -> Rgb<u8> {
    match index % 4 {
        0 => Rgb([220, 38, 38]),
        1 => Rgb([37, 99, 235]),
        2 => Rgb([22, 163, 74]),
        _ => Rgb([217, 119, 6]),
    }
}

fn join_shape<T: std::fmt::Display>(shape: &[T]) -> String {
    if shape.is_empty() {
        return "[]".to_owned();
    }
    let values = shape
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
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
        '[' => Some(["110", "100", "100", "100", "110"]),
        ']' => Some(["011", "001", "001", "001", "011"]),
        ',' => Some(["000", "000", "000", "010", "100"]),
        ' ' => Some(["000", "000", "000", "000", "000"]),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum ImageTensorLayout {
    Nchw,
    Nhwc,
}

fn rgb_to_f32_tensor(image: &image::RgbImage, layout: ImageTensorLayout) -> Vec<f32> {
    match layout {
        ImageTensorLayout::Nchw => {
            let channel_len = image.width() as usize * image.height() as usize;
            let mut data = vec![0.0; channel_len * 3];
            for (index, pixel) in image.pixels().enumerate() {
                data[index] = f32::from(pixel[0]);
                data[channel_len + index] = f32::from(pixel[1]);
                data[channel_len * 2 + index] = f32::from(pixel[2]);
            }
            data
        }
        ImageTensorLayout::Nhwc => image
            .pixels()
            .flat_map(|pixel| pixel.0.map(f32::from))
            .collect(),
    }
}

fn rgb_to_u8_tensor(image: &image::RgbImage, layout: ImageTensorLayout) -> Vec<u8> {
    match layout {
        ImageTensorLayout::Nchw => {
            let channel_len = image.width() as usize * image.height() as usize;
            let mut data = vec![0; channel_len * 3];
            for (index, pixel) in image.pixels().enumerate() {
                data[index] = pixel[0];
                data[channel_len + index] = pixel[1];
                data[channel_len * 2 + index] = pixel[2];
            }
            data
        }
        ImageTensorLayout::Nhwc => image
            .pixels()
            .flat_map(|pixel| pixel.0.into_iter())
            .collect(),
    }
}

fn validate_input_len(
    model_code: &'static str,
    input_shape: &[usize],
    input_len: usize,
) -> anyhow::Result<()> {
    let element_count =
        element_count(input_shape).ok_or_else(|| {
            anyhow!("invalid tensor shape for `{model_code}`: shape multiplication overflowed")
        })?;
    if element_count == input_len {
        Ok(())
    } else {
        bail!(
            "invalid tensor shape for `{model_code}`: shape requires {element_count} values but input contains {input_len}"
        )
    }
}

fn element_count(shape: &[usize]) -> Option<usize> {
    shape.iter().try_fold(1_usize, |current, dimension| {
        current.checked_mul(*dimension)
    })
}

fn first_input_name(session: &Session) -> String {
    session
        .inputs()
        .first()
        .map(|input| input.name().to_owned())
        .unwrap_or_default()
}

fn output_names(session: &Session) -> Vec<String> {
    session
        .outputs()
        .iter()
        .map(|output| output.name().to_owned())
        .collect()
}

fn outlet_to_info(outlet: &ort::value::Outlet) -> OnnxTensorIoInfo {
    let (tensor_type, shape) = match outlet.dtype() {
        ValueType::Tensor { ty, shape, .. } => {
            (ty.to_string(), shape.iter().copied().collect::<Vec<_>>())
        }
        dtype => (dtype.to_string(), Vec::new()),
    };

    OnnxTensorIoInfo {
        name: outlet.name().to_owned(),
        tensor_type,
        shape,
    }
}

fn output_summaries(
    output_names: &[String],
    outputs: ort::session::SessionOutputs<'_>,
) -> anyhow::Result<Vec<OnnxOutputSummary>> {
    outputs
        .iter()
        .enumerate()
        .map(|(index, (_name, value))| {
            let output_name = output_names
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("output_{index}"));
            summarize_output(output_name, &value)
        })
        .collect()
}

fn summarize_output(
    output_name: String,
    value: &ort::value::DynValue,
) -> anyhow::Result<OnnxOutputSummary> {
    let ValueType::Tensor { ty, .. } = value.dtype() else {
        bail!(
            "unsupported ONNX output tensor type `{}` from output `{output_name}`",
            value.dtype()
        );
    };

    match ty {
        TensorElementType::Float32 => summarize_f32_output(output_name, ty, value),
        TensorElementType::Float64 => summarize_primitive_output::<f64>(output_name, ty, value),
        TensorElementType::Int64 => summarize_primitive_output::<i64>(output_name, ty, value),
        TensorElementType::Int32 => summarize_primitive_output::<i32>(output_name, ty, value),
        TensorElementType::Int16 => summarize_primitive_output::<i16>(output_name, ty, value),
        TensorElementType::Int8 => summarize_primitive_output::<i8>(output_name, ty, value),
        TensorElementType::Uint64 => summarize_primitive_output::<u64>(output_name, ty, value),
        TensorElementType::Uint32 => summarize_primitive_output::<u32>(output_name, ty, value),
        TensorElementType::Uint16 => summarize_primitive_output::<u16>(output_name, ty, value),
        TensorElementType::Uint8 => summarize_primitive_output::<u8>(output_name, ty, value),
        TensorElementType::Bool => summarize_bool_output(output_name, ty, value),
        other => bail!(
            "unsupported ONNX output tensor type `{other}` from output `{output_name}`"
        ),
    }
}

fn summarize_f32_output(
    output_name: String,
    tensor_type: &TensorElementType,
    value: &ort::value::DynValue,
) -> anyhow::Result<OnnxOutputSummary> {
    let (shape, data) = value.try_extract_tensor::<f32>()?;
    Ok(OnnxOutputSummary {
        name: output_name,
        tensor_type: tensor_type.to_string(),
        shape: shape.iter().copied().collect(),
        element_count: data.len(),
        sample_f32: data.iter().take(MAX_OUTPUT_SAMPLE_VALUES).copied().collect(),
        full_f32: Some(data.to_vec()),
    })
}

fn summarize_primitive_output<T>(
    output_name: String,
    tensor_type: &TensorElementType,
    value: &ort::value::DynValue,
) -> anyhow::Result<OnnxOutputSummary>
where
    T: ort::value::PrimitiveTensorElementType + Copy + IntoSampleF32,
{
    let (shape, data) = value.try_extract_tensor::<T>()?;
    Ok(OnnxOutputSummary {
        name: output_name,
        tensor_type: tensor_type.to_string(),
        shape: shape.iter().copied().collect(),
        element_count: data.len(),
        sample_f32: data
            .iter()
            .take(MAX_OUTPUT_SAMPLE_VALUES)
            .map(|value| value.into_sample_f32())
            .collect(),
        full_f32: None,
    })
}

fn summarize_bool_output(
    output_name: String,
    tensor_type: &TensorElementType,
    value: &ort::value::DynValue,
) -> anyhow::Result<OnnxOutputSummary> {
    let (shape, data) = value.try_extract_tensor::<bool>()?;
    Ok(OnnxOutputSummary {
        name: output_name,
        tensor_type: tensor_type.to_string(),
        shape: shape.iter().copied().collect(),
        element_count: data.len(),
        sample_f32: data
            .iter()
            .take(MAX_OUTPUT_SAMPLE_VALUES)
            .map(|value| if *value { 1.0 } else { 0.0 })
            .collect(),
        full_f32: None,
    })
}

trait IntoSampleF32 {
    fn into_sample_f32(self) -> f32;
}

impl IntoSampleF32 for f32 {
    fn into_sample_f32(self) -> f32 {
        self
    }
}

impl IntoSampleF32 for f64 {
    fn into_sample_f32(self) -> f32 {
        self as f32
    }
}

impl IntoSampleF32 for i64 {
    fn into_sample_f32(self) -> f32 {
        self as f32
    }
}

impl IntoSampleF32 for i32 {
    fn into_sample_f32(self) -> f32 {
        self as f32
    }
}

impl IntoSampleF32 for i16 {
    fn into_sample_f32(self) -> f32 {
        f32::from(self)
    }
}

impl IntoSampleF32 for i8 {
    fn into_sample_f32(self) -> f32 {
        f32::from(self)
    }
}

impl IntoSampleF32 for u64 {
    fn into_sample_f32(self) -> f32 {
        self as f32
    }
}

impl IntoSampleF32 for u32 {
    fn into_sample_f32(self) -> f32 {
        self as f32
    }
}

impl IntoSampleF32 for u16 {
    fn into_sample_f32(self) -> f32 {
        f32::from(self)
    }
}

impl IntoSampleF32 for u8 {
    fn into_sample_f32(self) -> f32 {
        f32::from(self)
    }
}

fn path_error(path: PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow!("filesystem error at `{}`: {source}", path.display())
}
