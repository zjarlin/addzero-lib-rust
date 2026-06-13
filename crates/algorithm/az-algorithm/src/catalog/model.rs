//! 算法组件数据模型。

use az_derive_aliases::{apply, plain_copy_eq, serde_code_enum, serde_eq};

/// 算法组件的稳定标识。
///
/// `code()` 返回 snake_case 字符串，可用于配置、API 传输和持久化。
#[apply(serde_code_enum)]
pub enum AlgorithmComponentKind {
    /// 人脸检测。
    FaceDetection,
    /// 人脸识别。
    FaceRecognition,
    /// 人员检测。
    PersonDetection,
    /// OCR 文字识别。
    OcrTextRecognition,
    /// 火焰检测。
    FlameDetection,
    /// 安全帽检测。
    SafetyHelmetDetection,
    /// 车辆检测。
    VehicleDetection,
    /// 二维码识别。
    QrCodeRecognition,
    /// 工人敲击计数。
    WorkerHitCounting,
}

impl AlgorithmComponentKind {
    /// 返回该组件的完整规格。
    ///
    /// 通过遍历所有已注册的 [`AlgorithmComponentSpec`] 查找匹配项。
    #[must_use]
    pub fn spec(self) -> Option<&'static AlgorithmComponentSpec> {
        inventory::iter::<AlgorithmComponentSpec>
            .into_iter()
            .find(|spec| spec.kind == self)
    }
}

/// 算法任务类型。
#[apply(serde_code_enum)]
pub enum AlgorithmTaskKind {
    /// 在图像中定位目标。
    Detection,
    /// 识别或匹配已定位目标的身份、类别或内容。
    Recognition,
    /// 统计事件出现次数。
    Counting,
}

/// 算法关注的目标对象。
#[apply(serde_code_enum)]
pub enum AlgorithmTargetKind {
    /// 人脸。
    Face,
    /// 人。
    Person,
    /// 文本。
    Text,
    /// 火焰。
    Flame,
    /// 安全帽。
    SafetyHelmet,
    /// 车辆。
    Vehicle,
    /// 二维码。
    QrCode,
    /// 工人敲击动作。
    WorkerHit,
}

/// 算法输入契约项。
#[apply(serde_code_enum)]
pub enum AlgorithmInputKind {
    /// 单张图片或单帧视频帧。
    Image,
    /// 人脸底库、人员档案或其他可匹配目标库。
    ReferenceSet,
    /// 可选的感兴趣区域，用于限制检测或识别范围。
    RegionOfInterest,
    /// 视频帧序列。
    VideoFrames,
    /// 人员轨迹。
    PersonTracks,
    /// 视觉动作置信度。
    ActionScores,
    /// 视觉目标观测。
    TargetObservations,
    /// 工具或手部接触点。
    ContactPoints,
}

/// 算法输出契约项。
#[apply(serde_code_enum)]
pub enum AlgorithmOutputKind {
    /// 目标边界框。
    BoundingBox,
    /// 置信度分数。
    Confidence,
    /// 分类标签。
    ClassLabel,
    /// 识别出的身份。
    Identity,
    /// 文本内容。
    Text,
    /// 二维码载荷。
    QrPayload,
    /// 事件计数。
    EventCount,
    /// 事件时间戳。
    EventTimestamp,
    /// 人员跟踪标识。
    PersonTrackId,
    /// 动作状态。
    ActionState,
    /// 有效目标标识。
    TargetId,
    /// 接触点。
    ContactPoint,
    /// 无效候选原因。
    InvalidReason,
}

/// 单个算法组件的静态规格。
///
/// 各组件通过 `src/catalog/components/*.rs` 中的 [`inventory::submit!`] 注册，
/// 运行时通过查询函数聚合。
#[apply(plain_copy_eq)]
pub struct AlgorithmComponentSpec {
    /// 组件稳定标识。
    pub kind: AlgorithmComponentKind,
    /// 面向界面的中文名称。
    pub label: &'static str,
    /// 任务类型。
    pub task: AlgorithmTaskKind,
    /// 目标对象。
    pub target: AlgorithmTargetKind,
    /// 输入契约。
    pub inputs: &'static [AlgorithmInputKind],
    /// 输出契约。
    pub outputs: &'static [AlgorithmOutputKind],
    /// 组件职责摘要。
    pub description: &'static str,
}

/// 可序列化的算法组件描述。
///
/// 该类型适合直接返回给 API、CLI 或前端；静态规格可通过
/// [`AlgorithmComponentSpec::to_descriptor`] 转换为该 DTO。
#[apply(serde_eq)]
pub struct AlgorithmComponentDescriptor {
    /// 组件稳定标识。
    pub kind: AlgorithmComponentKind,
    /// 组件 code。
    pub code: String,
    /// 面向界面的中文名称。
    pub label: String,
    /// 任务类型。
    pub task: AlgorithmTaskKind,
    /// 目标对象。
    pub target: AlgorithmTargetKind,
    /// 输入契约。
    pub inputs: Vec<AlgorithmInputKind>,
    /// 输出契约。
    pub outputs: Vec<AlgorithmOutputKind>,
    /// 组件职责摘要。
    pub description: String,
}

impl AlgorithmComponentSpec {
    /// 返回组件 code。
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.kind.code()
    }

    /// 转换为可序列化描述对象。
    #[must_use]
    pub fn to_descriptor(&self) -> AlgorithmComponentDescriptor {
        AlgorithmComponentDescriptor {
            kind: self.kind,
            code: self.code().to_owned(),
            label: self.label.to_owned(),
            task: self.task,
            target: self.target,
            inputs: self.inputs.to_vec(),
            outputs: self.outputs.to_vec(),
            description: self.description.to_owned(),
        }
    }
}
