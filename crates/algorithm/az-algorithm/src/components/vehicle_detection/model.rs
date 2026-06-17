//! 车辆检测模型规格。

use std::path::PathBuf;

use az_onnx::onnx::image::model::{
    OnnxImageModelSpec, OnnxImageOutputKind, TensorElementKind, TensorInputSpec,
};

/// 车辆检测稳定算法 code。
pub const ALGORITHM_CODE: &str = "vehicle_detection";

/// 默认输出目录。
pub const DEFAULT_RESULT_DIR: &str = "target/az-algorithm-results/vehicle_detection";

/// 默认模型资源目录，基于本 crate 根目录解析。
pub const DEFAULT_MODEL_RESOURCE_DIR: &str = "resources/vehicle_detection/models";

/// COCO bicycle 类别 ID。
pub const COCO_BICYCLE_CLASS_ID: f32 = 2.0;
/// COCO car 类别 ID。
pub const COCO_CAR_CLASS_ID: f32 = 3.0;
/// COCO motorcycle 类别 ID。
pub const COCO_MOTORCYCLE_CLASS_ID: f32 = 4.0;
/// COCO bus 类别 ID。
pub const COCO_BUS_CLASS_ID: f32 = 6.0;
/// COCO truck 类别 ID。
pub const COCO_TRUCK_CLASS_ID: f32 = 8.0;

/// 默认车辆置信度阈值。
pub const DEFAULT_SCORE_THRESHOLD: f32 = 0.5;

const SSD_MOBILENET_INPUT: &[usize] = &[1, 1200, 1200, 3];

/// 复用于车辆检测的 COCO SSD MobileNet v1 模型。
pub const VEHICLE_DETECTION_COCO_SSD_MOBILENET_V1: OnnxImageModelSpec = OnnxImageModelSpec {
    code: "vehicle_detection_coco_ssd_mobilenet_v1",
    label: "COCO SSD MobileNet v1 vehicle detection",
    source_repo: "onnxmodelzoo/ssd_mobilenet_v1_10",
    source_file: "ssd_mobilenet_v1_10.onnx",
    local_file: "coco_ssd_mobilenet_v1_10.onnx",
    license: "apache-2.0",
    revision: "338a91b8e06061536f22129b4bf5227a3d496e8c",
    input: TensorInputSpec {
        shape: SSD_MOBILENET_INPUT,
        element: TensorElementKind::Uint8,
    },
    output_kind: OnnxImageOutputKind::RawTensor,
    notes: "COCO class filtering should select car, bus, truck, motorcycle, and bicycle detections.",
};

/// 车辆检测执行配置。
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleDetectionOptions {
    /// COCO SSD MobileNet ONNX 模型绝对路径。
    pub model_path: PathBuf,
    /// 输出目录绝对路径。
    pub output_dir: PathBuf,
    /// 车辆类别置信度阈值。
    pub score_threshold: f32,
}

/// COCO 中当前算法接受的车辆类。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VehicleClass {
    /// 自行车。
    Bicycle,
    /// 汽车。
    Car,
    /// 摩托车。
    Motorcycle,
    /// 公交车。
    Bus,
    /// 卡车。
    Truck,
}

impl VehicleClass {
    /// 返回 COCO 类别 ID。
    #[must_use]
    pub const fn coco_class_id(self) -> f32 {
        match self {
            Self::Bicycle => COCO_BICYCLE_CLASS_ID,
            Self::Car => COCO_CAR_CLASS_ID,
            Self::Motorcycle => COCO_MOTORCYCLE_CLASS_ID,
            Self::Bus => COCO_BUS_CLASS_ID,
            Self::Truck => COCO_TRUCK_CLASS_ID,
        }
    }
}

/// 单个车辆检测框。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VehicleDetectionBox {
    /// 左上角 x 坐标，单位是原图像素。
    pub x_min: f32,
    /// 左上角 y 坐标，单位是原图像素。
    pub y_min: f32,
    /// 右下角 x 坐标，单位是原图像素。
    pub x_max: f32,
    /// 右下角 y 坐标，单位是原图像素。
    pub y_max: f32,
    /// COCO 车辆类别。
    pub vehicle_class: VehicleClass,
    /// COCO 类别 ID。
    pub class_id: f32,
    /// 检测置信度。
    pub confidence: f32,
}

/// 单个 ONNX 输出摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VehicleDetectionOutputSummary {
    /// 输出张量名称。
    pub name: String,
    /// 输出张量元素类型。
    pub tensor_type: String,
    /// 运行时输出形状。
    pub shape: Vec<i64>,
    /// 张量标量元素数量。
    pub element_count: usize,
    /// 前几个 f32 样本值。
    pub sample_f32: Vec<f32>,
}

/// 车辆检测输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct VehicleDetectionOutputFiles {
    /// 原始输入图副本。
    pub source_input: PathBuf,
    /// 模型实际看到的 resize 输入预览图，包含车辆检测框。
    pub model_input_preview: PathBuf,
    /// ONNX 原始输出摘要 JSON。
    pub raw_outputs_json: PathBuf,
    /// COCO SSD 后处理得到的车辆框 JSON。
    pub detected_vehicles_json: PathBuf,
    /// 画出车辆框的输出图。
    pub detected_vehicles_image: PathBuf,
}

/// 车辆检测结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VehicleDetectionRun {
    /// 输入图片路径。
    pub input_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 检测到的车辆框。
    pub vehicles: Vec<VehicleDetectionBox>,
    /// 输出文件路径。
    pub files: VehicleDetectionOutputFiles,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<VehicleDetectionOutputSummary>,
}
