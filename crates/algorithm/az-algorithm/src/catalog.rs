//! 算法组件目录与输入输出契约。

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
    #[must_use]
    pub const fn spec(self) -> &'static AlgorithmComponentSpec {
        match self {
            Self::FaceDetection => &FACE_DETECTION,
            Self::FaceRecognition => &FACE_RECOGNITION,
            Self::PersonDetection => &PERSON_DETECTION,
            Self::OcrTextRecognition => &OCR_TEXT_RECOGNITION,
            Self::FlameDetection => &FLAME_DETECTION,
            Self::SafetyHelmetDetection => &SAFETY_HELMET_DETECTION,
            Self::VehicleDetection => &VEHICLE_DETECTION,
            Self::QrCodeRecognition => &QR_CODE_RECOGNITION,
            Self::WorkerHitCounting => &WORKER_HIT_COUNTING,
        }
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

const IMAGE_INPUTS: &[AlgorithmInputKind] = &[AlgorithmInputKind::Image];
const IMAGE_AND_REFERENCE_INPUTS: &[AlgorithmInputKind] =
    &[AlgorithmInputKind::Image, AlgorithmInputKind::ReferenceSet];
const IMAGE_AND_ROI_INPUTS: &[AlgorithmInputKind] = &[
    AlgorithmInputKind::Image,
    AlgorithmInputKind::RegionOfInterest,
];

const DETECTION_OUTPUTS: &[AlgorithmOutputKind] = &[
    AlgorithmOutputKind::BoundingBox,
    AlgorithmOutputKind::Confidence,
    AlgorithmOutputKind::ClassLabel,
];
const FACE_RECOGNITION_OUTPUTS: &[AlgorithmOutputKind] = &[
    AlgorithmOutputKind::BoundingBox,
    AlgorithmOutputKind::Confidence,
    AlgorithmOutputKind::Identity,
];
const TEXT_RECOGNITION_OUTPUTS: &[AlgorithmOutputKind] =
    &[AlgorithmOutputKind::BoundingBox, AlgorithmOutputKind::Text];
const QR_RECOGNITION_OUTPUTS: &[AlgorithmOutputKind] = &[
    AlgorithmOutputKind::BoundingBox,
    AlgorithmOutputKind::QrPayload,
    AlgorithmOutputKind::Confidence,
];
const VISUAL_WORKER_HIT_INPUTS: &[AlgorithmInputKind] = &[
    AlgorithmInputKind::VideoFrames,
    AlgorithmInputKind::PersonTracks,
    AlgorithmInputKind::ActionScores,
    AlgorithmInputKind::TargetObservations,
    AlgorithmInputKind::ContactPoints,
];
const EVENT_COUNT_OUTPUTS: &[AlgorithmOutputKind] = &[
    AlgorithmOutputKind::PersonTrackId,
    AlgorithmOutputKind::ActionState,
    AlgorithmOutputKind::EventCount,
    AlgorithmOutputKind::EventTimestamp,
    AlgorithmOutputKind::TargetId,
    AlgorithmOutputKind::ContactPoint,
    AlgorithmOutputKind::InvalidReason,
    AlgorithmOutputKind::Confidence,
];

/// 人脸检测算法组件规格。
pub const FACE_DETECTION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::FaceDetection,
    label: "人脸检测",
    task: AlgorithmTaskKind::Detection,
    target: AlgorithmTargetKind::Face,
    inputs: IMAGE_INPUTS,
    outputs: DETECTION_OUTPUTS,
    description: "在图片或视频帧中定位人脸区域并输出置信度。",
};

/// 人脸识别算法组件规格。
pub const FACE_RECOGNITION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::FaceRecognition,
    label: "人脸识别",
    task: AlgorithmTaskKind::Recognition,
    target: AlgorithmTargetKind::Face,
    inputs: IMAGE_AND_REFERENCE_INPUTS,
    outputs: FACE_RECOGNITION_OUTPUTS,
    description: "将检测到的人脸与人脸底库匹配并输出身份结果。",
};

/// 人员检测算法组件规格。
pub const PERSON_DETECTION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::PersonDetection,
    label: "人员检测",
    task: AlgorithmTaskKind::Detection,
    target: AlgorithmTargetKind::Person,
    inputs: IMAGE_INPUTS,
    outputs: DETECTION_OUTPUTS,
    description: "在图片或视频帧中定位人员目标。",
};

/// OCR 文字识别算法组件规格。
pub const OCR_TEXT_RECOGNITION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::OcrTextRecognition,
    label: "OCR文字识别",
    task: AlgorithmTaskKind::Recognition,
    target: AlgorithmTargetKind::Text,
    inputs: IMAGE_AND_ROI_INPUTS,
    outputs: TEXT_RECOGNITION_OUTPUTS,
    description: "识别图片中的文字区域并输出文本内容。",
};

/// 火焰检测算法组件规格。
pub const FLAME_DETECTION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::FlameDetection,
    label: "火焰检测",
    task: AlgorithmTaskKind::Detection,
    target: AlgorithmTargetKind::Flame,
    inputs: IMAGE_INPUTS,
    outputs: DETECTION_OUTPUTS,
    description: "检测图片或视频帧中的火焰目标。",
};

/// 安全帽检测算法组件规格。
pub const SAFETY_HELMET_DETECTION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::SafetyHelmetDetection,
    label: "安全帽检测",
    task: AlgorithmTaskKind::Detection,
    target: AlgorithmTargetKind::SafetyHelmet,
    inputs: IMAGE_INPUTS,
    outputs: DETECTION_OUTPUTS,
    description: "检测人员头部安全帽佩戴相关目标。",
};

/// 车辆检测算法组件规格。
pub const VEHICLE_DETECTION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::VehicleDetection,
    label: "车辆检测",
    task: AlgorithmTaskKind::Detection,
    target: AlgorithmTargetKind::Vehicle,
    inputs: IMAGE_INPUTS,
    outputs: DETECTION_OUTPUTS,
    description: "在图片或视频帧中定位车辆目标。",
};

/// 二维码识别算法组件规格。
pub const QR_CODE_RECOGNITION: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::QrCodeRecognition,
    label: "二维码识别",
    task: AlgorithmTaskKind::Recognition,
    target: AlgorithmTargetKind::QrCode,
    inputs: IMAGE_AND_ROI_INPUTS,
    outputs: QR_RECOGNITION_OUTPUTS,
    description: "识别图片中的二维码区域并输出解码载荷。",
};

/// 工人敲击计数算法组件规格。
pub const WORKER_HIT_COUNTING: AlgorithmComponentSpec = AlgorithmComponentSpec {
    kind: AlgorithmComponentKind::WorkerHitCounting,
    label: "工人敲击计数",
    task: AlgorithmTaskKind::Counting,
    target: AlgorithmTargetKind::WorkerHit,
    inputs: VISUAL_WORKER_HIT_INPUTS,
    outputs: EVENT_COUNT_OUTPUTS,
    description: "基于人员轨迹、接触点、目标类型和目标响应，按每个人分别统计有效敲击和无效候选。",
};

/// 默认算法组件清单。
pub const ALGORITHM_COMPONENTS: &[AlgorithmComponentSpec] = &[
    FACE_DETECTION,
    FACE_RECOGNITION,
    PERSON_DETECTION,
    OCR_TEXT_RECOGNITION,
    FLAME_DETECTION,
    SAFETY_HELMET_DETECTION,
    VEHICLE_DETECTION,
    QR_CODE_RECOGNITION,
    WORKER_HIT_COUNTING,
];

/// 返回全部算法组件规格。
#[must_use]
pub const fn algorithm_components() -> &'static [AlgorithmComponentSpec] {
    ALGORITHM_COMPONENTS
}

/// 根据稳定 code 查找算法组件。
#[must_use]
pub fn algorithm_component_by_code(code: &str) -> Option<&'static AlgorithmComponentSpec> {
    AlgorithmComponentKind::from_code(code).map(AlgorithmComponentKind::spec)
}

/// 根据中文名称查找算法组件。
#[must_use]
pub fn algorithm_component_by_label(label: &str) -> Option<&'static AlgorithmComponentSpec> {
    ALGORITHM_COMPONENTS
        .iter()
        .find(|component| component.label == label)
}

/// 按任务类型过滤算法组件。
pub fn algorithm_components_by_task(
    task: AlgorithmTaskKind,
) -> impl Iterator<Item = &'static AlgorithmComponentSpec> {
    ALGORITHM_COMPONENTS
        .iter()
        .filter(move |component| component.task == task)
}

/// 按目标对象过滤算法组件。
pub fn algorithm_components_by_target(
    target: AlgorithmTargetKind,
) -> impl Iterator<Item = &'static AlgorithmComponentSpec> {
    ALGORITHM_COMPONENTS
        .iter()
        .filter(move |component| component.target == target)
}

/// 返回全部组件的可序列化描述。
#[must_use]
pub fn algorithm_component_descriptors() -> Vec<AlgorithmComponentDescriptor> {
    ALGORITHM_COMPONENTS
        .iter()
        .map(AlgorithmComponentSpec::to_descriptor)
        .collect()
}
