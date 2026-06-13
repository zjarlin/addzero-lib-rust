//! 图片 ONNX 推理辅助函数。

use std::fs;
use std::path::{Path, PathBuf};

use image::DynamicImage;
use image::imageops::FilterType;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::error::{OnnxImageError, OnnxImageResult};
use crate::logic_onnx_image::model::{
    OnnxImageModelSpec, OnnxImageOutputFiles, OnnxImageRun, OnnxInferenceSummary,
    OnnxModelMetadata, OnnxOutputSummary, OnnxTensorIoInfo, PreparedImageTensor,
    TensorElementKind, TensorInputSpec,
};

const MAX_OUTPUT_SAMPLE_VALUES: usize = 8;

/// 已加载的本地 ONNX Runtime 会话。
#[derive(Debug)]
pub struct LocalOnnxSession {
    model_path: PathBuf,
    session: Session,
}

impl LocalOnnxSession {
    /// 将本地 ONNX 模型文件加载进 ONNX Runtime。
    ///
    /// # Errors
    /// 当模型文件不存在或 ONNX Runtime 加载失败时返回错误。
    pub fn from_file(path: impl AsRef<Path>) -> OnnxImageResult<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(OnnxImageError::io(
                path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "ONNX model file not found"),
            ));
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
    ) -> OnnxImageResult<OnnxInferenceSummary> {
        validate_input_len("custom_onnx_model", input_shape, input_data.len())?;
        let input_name = first_input_name(&self.session);
        let output_names = output_names(&self.session);
        let input_array =
            ArrayD::from_shape_vec(IxDyn(input_shape), input_data).map_err(|error| {
                OnnxImageError::InvalidTensorShape {
                    model_code: "custom_onnx_model",
                    reason: error.to_string(),
                }
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
    ) -> OnnxImageResult<OnnxInferenceSummary> {
        validate_input_len("custom_onnx_model", input_shape, input_data.len())?;
        let input_name = first_input_name(&self.session);
        let output_names = output_names(&self.session);
        let input_array =
            ArrayD::from_shape_vec(IxDyn(input_shape), input_data).map_err(|error| {
                OnnxImageError::InvalidTensorShape {
                    model_code: "custom_onnx_model",
                    reason: error.to_string(),
                }
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
    ) -> OnnxImageResult<(PreparedImageTensor, OnnxInferenceSummary)> {
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
    ) -> OnnxImageResult<(PreparedImageTensor, OnnxInferenceSummary)> {
        let prepared = prepare_dynamic_image_tensor_for_spec(spec, image)?;
        let summary = self.run_prepared_image(spec, &prepared)?;
        Ok((prepared, summary))
    }

    fn run_prepared_image(
        &mut self,
        spec: &OnnxImageModelSpec,
        prepared: &PreparedImageTensor,
    ) -> OnnxImageResult<OnnxInferenceSummary> {
        let summary = match prepared.element {
            TensorElementKind::Float32 => self.run_f32(
                &prepared.shape,
                prepared.f32_data.clone().ok_or_else(|| {
                    OnnxImageError::InvalidTensorShape {
                        model_code: spec.code,
                        reason: "prepared image does not contain f32 tensor data".to_owned(),
                    }
                })?,
            )?,
            TensorElementKind::Uint8 => self.run_u8(
                &prepared.shape,
                prepared.u8_data.clone().ok_or_else(|| {
                    OnnxImageError::InvalidTensorShape {
                        model_code: spec.code,
                        reason: "prepared image does not contain u8 tensor data".to_owned(),
                    }
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
) -> OnnxImageResult<OnnxImageRun> {
    let model_path = spec.require_local_path(resource_dir)?;
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| OnnxImageError::io(image_path.as_ref().to_path_buf(), source))?;
    let output_dir = output_dir.as_ref().to_path_buf();
    fs::create_dir_all(&output_dir).map_err(|source| OnnxImageError::io(output_dir.clone(), source))?;

    dbg!(&model_path);
    dbg!(&image_path);
    dbg!(&output_dir);

    let mut session = LocalOnnxSession::from_file(&model_path)?;
    let (prepared, summary) = session.run_image_file(spec, &image_path)?;
    let files = write_inference_artifacts(algorithm_code, &image_path, &prepared, &summary, &output_dir)?;

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
) -> OnnxImageResult<PreparedImageTensor> {
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
) -> OnnxImageResult<PreparedImageTensor> {
    prepare_dynamic_image_tensor(spec.code, spec.input, image)
}

fn prepare_dynamic_image_tensor(
    model_code: &'static str,
    input: TensorInputSpec,
    image: &DynamicImage,
) -> OnnxImageResult<PreparedImageTensor> {
    let [batch, first, second, third] = input.shape else {
        return Err(OnnxImageError::InvalidTensorShape {
            model_code,
            reason: format!("expected 4D image input shape, got {:?}", input.shape),
        });
    };
    if *batch != 1 {
        return Err(OnnxImageError::InvalidTensorShape {
            model_code,
            reason: format!("image tests support batch=1 only, got {batch}"),
        });
    }

    let (layout, height, width) = if *first == 3 {
        (ImageTensorLayout::Nchw, *second, *third)
    } else if *third == 3 {
        (ImageTensorLayout::Nhwc, *first, *second)
    } else {
        return Err(OnnxImageError::InvalidTensorShape {
            model_code,
            reason: format!("expected RGB channel dimension, got {:?}", input.shape),
        });
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
    source_image: &DynamicImage,
    prepared: &PreparedImageTensor,
    summary: &OnnxInferenceSummary,
    output_dir: &Path,
) -> OnnxImageResult<OnnxImageOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| OnnxImageError::io(output_dir.to_path_buf(), source))?;
    let files = OnnxImageOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
    };

    source_image.save(&files.source_input)?;
    prepared.preview.save(&files.model_input_preview)?;
    let json = serde_json::to_string_pretty(summary)?;
    fs::write(&files.raw_outputs_json, json)
        .map_err(|source| OnnxImageError::io(files.raw_outputs_json.clone(), source))?;
    assert_real_outputs_exist(algorithm_code, summary);

    Ok(files)
}

#[expect(
    clippy::dbg_macro,
    reason = "用户要求测试直接打印输入、输出文件的绝对路径"
)]
fn write_inference_artifacts(
    algorithm_code: &str,
    source_image: &Path,
    prepared: &PreparedImageTensor,
    summary: &OnnxInferenceSummary,
    output_dir: &Path,
) -> OnnxImageResult<OnnxImageOutputFiles> {
    let files = OnnxImageOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
    };

    dbg!(&files.source_input);
    dbg!(&files.model_input_preview);
    dbg!(&files.raw_outputs_json);

    prepared.preview.save(&files.model_input_preview)?;
    fs::copy(source_image, &files.source_input)
        .map_err(|source| OnnxImageError::io(source_image.to_path_buf(), source))?;
    let json = serde_json::to_string_pretty(summary)?;
    fs::write(&files.raw_outputs_json, json)
        .map_err(|source| OnnxImageError::io(files.raw_outputs_json.clone(), source))?;

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
) -> OnnxImageResult<()> {
    let element_count =
        element_count(input_shape).ok_or_else(|| OnnxImageError::InvalidTensorShape {
            model_code,
            reason: "shape multiplication overflowed".to_owned(),
        })?;
    if element_count == input_len {
        Ok(())
    } else {
        Err(OnnxImageError::InvalidTensorShape {
            model_code,
            reason: format!("shape requires {element_count} values but input contains {input_len}"),
        })
    }
}

fn element_count(shape: &[usize]) -> Option<usize> {
    shape
        .iter()
        .try_fold(1_usize, |current, dimension| current.checked_mul(*dimension))
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
) -> OnnxImageResult<Vec<OnnxOutputSummary>> {
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
) -> OnnxImageResult<OnnxOutputSummary> {
    let ValueType::Tensor { ty, .. } = value.dtype() else {
        return Err(OnnxImageError::UnsupportedOnnxOutput {
            output_name,
            tensor_type: value.dtype().to_string(),
        });
    };

    match ty {
        TensorElementType::Float32 => summarize_primitive_output::<f32>(output_name, ty, value),
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
        other => Err(OnnxImageError::UnsupportedOnnxOutput {
            output_name,
            tensor_type: other.to_string(),
        }),
    }
}

fn summarize_primitive_output<T>(
    output_name: String,
    tensor_type: &TensorElementType,
    value: &ort::value::DynValue,
) -> OnnxImageResult<OnnxOutputSummary>
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
    })
}

fn summarize_bool_output(
    output_name: String,
    tensor_type: &TensorElementType,
    value: &ort::value::DynValue,
) -> OnnxImageResult<OnnxOutputSummary> {
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
