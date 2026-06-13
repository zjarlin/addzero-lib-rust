//! 车辆检测执行辅助函数。

use std::fs;
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use ndarray::{ArrayD, IxDyn};
use ort::session::Session;
use ort::value::{Tensor, TensorElementType, ValueType};

use crate::error::{VehicleDetectionError, VehicleDetectionResult};
use crate::logic_vehicle_detection::model::{
    DEFAULT_MODEL_RESOURCE_DIR, DEFAULT_RESULT_DIR, DEFAULT_SCORE_THRESHOLD, VehicleClass,
    VehicleDetectionBox, VehicleDetectionOptions, VehicleDetectionOutputFiles,
    VehicleDetectionOutputSummary, VehicleDetectionRun, VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1,
};

const OUTPUT_SAMPLE_VALUES: usize = 8;
const DETECTION_BOXES_OUTPUT: &str = "detection_boxes:0";
const DETECTION_CLASSES_OUTPUT: &str = "detection_classes:0";
const DETECTION_SCORES_OUTPUT: &str = "detection_scores:0";
const NUM_DETECTIONS_OUTPUT: &str = "num_detections:0";

impl VehicleDetectionOptions {
    /// 使用当前 workspace 下的默认模型和默认输出目录。
    ///
    /// # Errors
    /// 当前工作目录无法定位或模型文件不存在时返回错误。
    pub fn default_workspace() -> VehicleDetectionResult<Self> {
        let workspace_root = workspace_root()?;
        let model_path = workspace_root
            .join("crates/algorithm/az-vehicle-detection")
            .join(DEFAULT_MODEL_RESOURCE_DIR)
            .join(VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.local_file);
        if !model_path.is_file() {
            return Err(VehicleDetectionError::ModelFileMissing { path: model_path });
        }

        Ok(Self {
            model_path,
            output_dir: workspace_root.join(DEFAULT_RESULT_DIR),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
        })
    }
}

/// 传入图片绝对路径执行车辆检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_vehicles_from_path(
    image_path: impl AsRef<Path>,
) -> VehicleDetectionResult<VehicleDetectionRun> {
    let options = VehicleDetectionOptions::default_workspace()?;
    detect_vehicles_from_path_with_options(image_path, &options)
}

/// 传入图片绝对路径和自定义配置执行车辆检测。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn detect_vehicles_from_path_with_options(
    image_path: impl AsRef<Path>,
    options: &VehicleDetectionOptions,
) -> VehicleDetectionResult<VehicleDetectionRun> {
    validate_detection_options(options)?;
    let image_path = std::fs::canonicalize(image_path.as_ref())
        .map_err(|source| VehicleDetectionError::io(image_path.as_ref().to_path_buf(), source))?;
    let image = image::open(&image_path)?;
    run_detection(image, image_path, options)
}

/// 使用默认模型和默认输出目录执行车辆检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_vehicle_detection_from_path(
    image_path: impl AsRef<Path>,
) -> VehicleDetectionResult<VehicleDetectionRun> {
    detect_vehicles_from_path(image_path)
}

/// 使用默认模型和指定输出目录执行车辆检测真实推理。
///
/// # Errors
/// 图片读取、模型加载、推理或输出文件写入失败时返回错误。
pub fn run_vehicle_detection_from_path_with_output(
    image_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> VehicleDetectionResult<VehicleDetectionRun> {
    detect_vehicles_from_path_with_options(
        image_path,
        &VehicleDetectionOptions {
            model_path: crate_root()
                .join(DEFAULT_MODEL_RESOURCE_DIR)
                .join(VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.local_file),
            output_dir: output_dir.as_ref().to_path_buf(),
            score_threshold: DEFAULT_SCORE_THRESHOLD,
        },
    )
}

fn run_detection(
    image: DynamicImage,
    input_path: PathBuf,
    options: &VehicleDetectionOptions,
) -> VehicleDetectionResult<VehicleDetectionRun> {
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| VehicleDetectionError::io(options.output_dir.clone(), source))?;

    let prepared = prepare_coco_ssd_image(&image);
    let inference = run_coco_ssd_model(&options.model_path, prepared.tensor_data)?;
    let vehicles = decode_coco_ssd_vehicle_boxes(
        &inference.tensors,
        image.width(),
        image.height(),
        options.score_threshold,
    )?;
    let files = write_output_files(
        &image,
        &prepared.preview,
        &input_path,
        &vehicles,
        &inference.summaries,
        &options.output_dir,
    )?;

    Ok(VehicleDetectionRun {
        input_path,
        model_path: options.model_path.clone(),
        vehicles,
        files,
        raw_outputs: inference.summaries,
    })
}

fn run_coco_ssd_model(
    model_path: &Path,
    tensor_data: Vec<u8>,
) -> VehicleDetectionResult<CocoSsdInferenceOutput> {
    if !model_path.is_file() {
        return Err(VehicleDetectionError::ModelFileMissing {
            path: model_path.to_path_buf(),
        });
    }

    let mut builder = Session::builder()?;
    let mut session = builder.commit_from_file(model_path)?;
    run_coco_ssd_session(&mut session, tensor_data)
}

fn run_coco_ssd_session(
    session: &mut Session,
    tensor_data: Vec<u8>,
) -> VehicleDetectionResult<CocoSsdInferenceOutput> {
    let input_array = ArrayD::from_shape_vec(
        IxDyn(VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.input.shape),
        tensor_data,
    )
    .map_err(|source| VehicleDetectionError::InvalidTensorShape {
        model_code: VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.code,
        reason: source.to_string(),
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

fn collect_coco_ssd_outputs(
    output_names: &[String],
    outputs: ort::session::SessionOutputs<'_>,
) -> VehicleDetectionResult<CocoSsdInferenceOutput> {
    let mut summaries = Vec::new();
    let mut tensors = Vec::new();

    for (index, (_name, value)) in outputs.iter().enumerate() {
        let output_name = output_names
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("output_{index}"));
        let ValueType::Tensor { ty, .. } = value.dtype() else {
            return Err(VehicleDetectionError::UnsupportedOnnxOutput {
                output_name,
                tensor_type: value.dtype().to_string(),
            });
        };
        if !matches!(ty, TensorElementType::Float32) {
            return Err(VehicleDetectionError::UnsupportedOnnxOutput {
                output_name,
                tensor_type: ty.to_string(),
            });
        }

        let (shape, data) = value.try_extract_tensor::<f32>()?;
        let data = data.to_vec();
        summaries.push(VehicleDetectionOutputSummary {
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

fn decode_coco_ssd_vehicle_boxes(
    outputs: &[CocoSsdOutputTensor],
    image_width: u32,
    image_height: u32,
    score_threshold: f32,
) -> VehicleDetectionResult<Vec<VehicleDetectionBox>> {
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
    let mut vehicles = Vec::new();
    for index in 0..detection_count {
        let class_id = classes.data[index];
        let Some(vehicle_class) = vehicle_class_from_coco_id(class_id) else {
            continue;
        };

        let confidence = scores.data[index];
        if confidence < score_threshold {
            continue;
        }

        let box_index = index * 4;
        let y_min = boxes.data[box_index].clamp(0.0, 1.0) * image_height as f32;
        let x_min = boxes.data[box_index + 1].clamp(0.0, 1.0) * image_width as f32;
        let y_max = boxes.data[box_index + 2].clamp(0.0, 1.0) * image_height as f32;
        let x_max = boxes.data[box_index + 3].clamp(0.0, 1.0) * image_width as f32;
        vehicles.push(VehicleDetectionBox {
            x_min,
            y_min,
            x_max,
            y_max,
            vehicle_class,
            class_id: vehicle_class.coco_class_id(),
            confidence,
        });
    }

    vehicles.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    Ok(vehicles)
}

fn vehicle_class_from_coco_id(class_id: f32) -> Option<VehicleClass> {
    if !class_id.is_finite() {
        return None;
    }

    match class_id.round() as i32 {
        2 => Some(VehicleClass::Bicycle),
        3 => Some(VehicleClass::Car),
        4 => Some(VehicleClass::Motorcycle),
        6 => Some(VehicleClass::Bus),
        8 => Some(VehicleClass::Truck),
        _ => None,
    }
}

fn write_output_files(
    image: &DynamicImage,
    preview: &RgbImage,
    input_path: &Path,
    vehicles: &[VehicleDetectionBox],
    summaries: &[VehicleDetectionOutputSummary],
    output_dir: &Path,
) -> VehicleDetectionResult<VehicleDetectionOutputFiles> {
    fs::create_dir_all(output_dir)
        .map_err(|source| VehicleDetectionError::io(output_dir.to_path_buf(), source))?;
    let files = VehicleDetectionOutputFiles {
        source_input: output_dir.join("source_input.jpg"),
        model_input_preview: output_dir.join("model_input_preview.png"),
        raw_outputs_json: output_dir.join("raw_outputs.json"),
        detected_vehicles_json: output_dir.join("detected_vehicles.json"),
        detected_vehicles_image: output_dir.join("detected_vehicles.png"),
    };

    fs::copy(input_path, &files.source_input)
        .map_err(|source| VehicleDetectionError::io(input_path.to_path_buf(), source))?;

    let mut marked_preview = preview.clone();
    draw_scaled_vehicle_boxes(
        &mut marked_preview,
        vehicles,
        image.width(),
        image.height(),
    );
    marked_preview.save(&files.model_input_preview)?;

    let raw_json = serde_json::to_string_pretty(summaries)?;
    fs::write(&files.raw_outputs_json, raw_json)
        .map_err(|source| VehicleDetectionError::io(files.raw_outputs_json.clone(), source))?;

    let vehicle_json = serde_json::to_string_pretty(vehicles)?;
    fs::write(&files.detected_vehicles_json, vehicle_json)
        .map_err(|source| VehicleDetectionError::io(files.detected_vehicles_json.clone(), source))?;

    let mut marked_image = image.to_rgb8();
    for vehicle in vehicles {
        draw_vehicle_box(&mut marked_image, vehicle);
    }
    marked_image.save(&files.detected_vehicles_image)?;

    Ok(files)
}

fn prepare_coco_ssd_image(image: &DynamicImage) -> PreparedCocoSsdImage {
    let preview = image.resize_exact(1200, 1200, FilterType::Triangle).to_rgb8();
    let tensor_data = preview.pixels().flat_map(|pixel| pixel.0).collect();
    PreparedCocoSsdImage {
        preview,
        tensor_data,
    }
}

fn draw_scaled_vehicle_boxes(
    image: &mut RgbImage,
    vehicles: &[VehicleDetectionBox],
    source_width: u32,
    source_height: u32,
) {
    let scale_x = image.width() as f32 / source_width as f32;
    let scale_y = image.height() as f32 / source_height as f32;
    for vehicle in vehicles {
        draw_vehicle_box(
            image,
            &VehicleDetectionBox {
                x_min: vehicle.x_min * scale_x,
                y_min: vehicle.y_min * scale_y,
                x_max: vehicle.x_max * scale_x,
                y_max: vehicle.y_max * scale_y,
                vehicle_class: vehicle.vehicle_class,
                class_id: vehicle.class_id,
                confidence: vehicle.confidence,
            },
        );
    }
}

fn draw_vehicle_box(image: &mut RgbImage, vehicle: &VehicleDetectionBox) {
    let x = vehicle.x_min.round() as i32;
    let y = vehicle.y_min.round() as i32;
    let width = (vehicle.x_max - vehicle.x_min).round().max(1.0) as u32;
    let height = (vehicle.y_max - vehicle.y_min).round().max(1.0) as u32;
    draw_hollow_rect_mut(
        image,
        Rect::at(x, y).of_size(width, height),
        Rgb([255, 70, 40]),
    );
}

fn require_output<'a>(
    outputs: &'a [CocoSsdOutputTensor],
    output_name: &str,
) -> VehicleDetectionResult<&'a CocoSsdOutputTensor> {
    outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or_else(|| VehicleDetectionError::MissingOnnxOutput {
            output_name: output_name.to_owned(),
        })
}

fn validate_boxes(boxes: &CocoSsdOutputTensor) -> VehicleDetectionResult<()> {
    if boxes.shape.as_slice() == [1, 100, 4] && boxes.data.len() == 400 {
        return Ok(());
    }

    Err(VehicleDetectionError::InvalidTensorShape {
        model_code: VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.code,
        reason: format!(
            "output `{}` expected [1, 100, 4], got {:?}",
            boxes.name, boxes.shape
        ),
    })
}

fn validate_vector(
    output: &CocoSsdOutputTensor,
    expected_name: &str,
) -> VehicleDetectionResult<()> {
    if output.shape.as_slice() == [1, 100] && output.data.len() == 100 {
        return Ok(());
    }

    Err(VehicleDetectionError::InvalidTensorShape {
        model_code: VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.code,
        reason: format!(
            "output `{expected_name}` expected [1, 100], got {:?}",
            output.shape
        ),
    })
}

fn validate_num_detections(output: &CocoSsdOutputTensor) -> VehicleDetectionResult<()> {
    if output.shape.as_slice() == [1] && output.data.len() == 1 {
        return Ok(());
    }

    Err(VehicleDetectionError::InvalidTensorShape {
        model_code: VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.code,
        reason: format!(
            "output `{}` expected [1], got {:?}",
            output.name, output.shape
        ),
    })
}

fn validate_detection_options(options: &VehicleDetectionOptions) -> VehicleDetectionResult<()> {
    if !options.model_path.is_file() {
        return Err(VehicleDetectionError::ModelFileMissing {
            path: options.model_path.clone(),
        });
    }
    if !(0.0..=1.0).contains(&options.score_threshold) || !options.score_threshold.is_finite() {
        return Err(VehicleDetectionError::InvalidTensorShape {
            model_code: VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1.code,
            reason: "score_threshold must be finite and within 0.0..=1.0".to_owned(),
        });
    }
    Ok(())
}

fn workspace_root() -> VehicleDetectionResult<PathBuf> {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .map_err(|source| VehicleDetectionError::io(PathBuf::from(env!("CARGO_MANIFEST_DIR")), source))
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
struct CocoSsdInferenceOutput {
    summaries: Vec<VehicleDetectionOutputSummary>,
    tensors: Vec<CocoSsdOutputTensor>,
}
