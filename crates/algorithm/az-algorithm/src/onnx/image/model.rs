//! 图片 ONNX 推理公开模型。

use std::path::{Path, PathBuf};

use anyhow::bail;

/// ONNX 图片输入张量元素类型。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TensorElementKind {
    /// 32 位浮点张量。
    Float32,
    /// 无符号 8 位整型张量。
    Uint8,
}

/// ONNX 图片模型输出的业务语义。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum OnnxImageOutputKind {
    /// 未后处理的普通张量输出。
    RawTensor,
    /// 整图分类分数输出。
    ImageClassification,
    /// 特征向量输出，例如人脸 embedding。
    Embedding,
}

/// ONNX 图片推理使用的张量形状。
#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
pub struct TensorInputSpec {
    /// 输入张量维度。
    pub shape: &'static [usize],
    /// 输入张量元素类型。
    pub element: TensorElementKind,
}

/// 单个本地 ONNX 图片模型的静态描述。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OnnxImageModelSpec {
    /// 稳定模型 code。
    pub code: &'static str,
    /// 可读模型名称。
    pub label: &'static str,
    /// 来源仓库，例如 Hugging Face model id。
    pub source_repo: &'static str,
    /// 来源仓库内的文件路径。
    pub source_file: &'static str,
    /// 本地资源文件名。
    pub local_file: &'static str,
    /// 上游模型仓库声明的许可。
    pub license: &'static str,
    /// 集成时验证过的来源 revision。
    pub revision: &'static str,
    /// 本地图片推理使用的输入张量形状。
    pub input: TensorInputSpec,
    /// 输出张量的业务语义，用于避免把 raw tensor 误画成检测或分类结果。
    pub output_kind: OnnxImageOutputKind,
    /// 模型范围、复用方式或限制说明。
    pub notes: &'static str,
}

impl OnnxImageModelSpec {
    /// 解析给定资源目录下的预期本地文件路径。
    #[must_use]
    pub fn local_path(&self, resource_dir: impl AsRef<Path>) -> PathBuf {
        resource_dir.as_ref().join(self.local_file)
    }

    /// 解析并验证给定资源目录下的预期本地文件路径。
    ///
    /// # Errors
    /// 当文件不存在时返回错误。
    pub fn require_local_path(&self, resource_dir: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let path = self.local_path(resource_dir);
        if path.is_file() {
            Ok(path)
        } else {
            bail!("model file `{}` is missing", path.display())
        }
    }
}

/// 单个 ONNX 输入或输出张量的可序列化视图。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct OnnxTensorIoInfo {
    /// 输入或输出名称。
    pub name: String,
    /// ONNX Runtime 渲染出的元素类型。
    pub tensor_type: String,
    /// 图中声明的形状，动态维度用负数表示。
    pub shape: Vec<i64>,
}

/// 加载时收集的可序列化 ONNX 模型元数据。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct OnnxModelMetadata {
    /// 模型文件路径。
    pub model_path: PathBuf,
    /// 图输入张量。
    pub inputs: Vec<OnnxTensorIoInfo>,
    /// 图输出张量。
    pub outputs: Vec<OnnxTensorIoInfo>,
}

/// 单次推理的输出摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OnnxOutputSummary {
    /// 输出名称。
    pub name: String,
    /// 输出元素类型。
    pub tensor_type: String,
    /// 运行时输出形状。
    pub shape: Vec<i64>,
    /// 张量标量元素数量。
    pub element_count: usize,
    /// 前几个可转换为 f32 的标量值。
    pub sample_f32: Vec<f32>,
    /// 完整 f32 输出，仅供运行时后处理使用，不写入 JSON 摘要。
    #[serde(skip)]
    pub full_f32: Option<Vec<f32>>,
}

/// smoke 或真实图片推理返回的摘要。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OnnxInferenceSummary {
    /// 模型文件路径。
    pub model_path: PathBuf,
    /// 本次运行使用的输入张量名称。
    pub input_name: String,
    /// 本次运行使用的输入形状。
    pub input_shape: Vec<usize>,
    /// 输出摘要列表。
    pub outputs: Vec<OnnxOutputSummary>,
}

/// 为 ONNX 模型准备好的 resize 后图片张量。
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedImageTensor {
    /// 模型输入形状。
    pub shape: Vec<usize>,
    /// 模型期望的元素类型。
    pub element: TensorElementKind,
    /// resize 后宽度。
    pub width: u32,
    /// resize 后高度。
    pub height: u32,
    /// 张量 layout 转换前的 RGB resize 预览图。
    pub preview: image::RgbImage,
    pub(crate) f32_data: Option<Vec<f32>>,
    pub(crate) u8_data: Option<Vec<u8>>,
}

impl PreparedImageTensor {
    /// 从已完成预处理的 f32 图像张量构造输入。
    #[must_use]
    pub fn from_f32_tensor(
        shape: Vec<usize>,
        width: u32,
        height: u32,
        preview: image::RgbImage,
        f32_data: Vec<f32>,
    ) -> Self {
        Self {
            shape,
            element: TensorElementKind::Float32,
            width,
            height,
            preview,
            f32_data: Some(f32_data),
            u8_data: None,
        }
    }

    /// 返回准备好的 f32 输入张量数据。
    #[must_use]
    pub fn f32_tensor_data(&self) -> Option<&[f32]> {
        self.f32_data.as_deref()
    }

    /// 返回准备好的 u8 输入张量数据。
    #[must_use]
    pub fn u8_tensor_data(&self) -> Option<&[u8]> {
        self.u8_data.as_deref()
    }
}

/// ONNX 图片推理输出文件路径。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct OnnxImageOutputFiles {
    /// 原始输入图副本。
    pub source_input: PathBuf,
    /// 模型实际看到的 resize 输入预览图。
    pub model_input_preview: PathBuf,
    /// ONNX 原始输出摘要 JSON。
    pub raw_outputs_json: PathBuf,
    /// ONNX 原始输出的可视化审阅图。
    pub raw_output_review: PathBuf,
}

/// ONNX 图片推理运行结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OnnxImageRun {
    /// 输入图片路径。
    pub input_path: PathBuf,
    /// 模型路径。
    pub model_path: PathBuf,
    /// 输出文件路径。
    pub files: OnnxImageOutputFiles,
    /// ONNX 输出摘要。
    pub raw_outputs: Vec<OnnxOutputSummary>,
}
