use crate::catalog::model::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

pub const SPEC: AlgorithmComponentSpec = AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::FaceDetection,
        label: "人脸检测",
        task: AlgorithmTaskKind::Detection,
        target: AlgorithmTargetKind::Face,
        inputs: &[AlgorithmInputKind::Image],
        outputs: &[
            AlgorithmOutputKind::BoundingBox,
            AlgorithmOutputKind::Confidence,
            AlgorithmOutputKind::ClassLabel,
        ],
        description: "在图片或视频帧中定位人脸区域并输出置信度。",
    };
