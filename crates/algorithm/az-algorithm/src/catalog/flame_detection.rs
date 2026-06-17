use crate::catalog::model::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

pub const SPEC: AlgorithmComponentSpec = AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::FlameDetection,
        label: "火焰检测",
        task: AlgorithmTaskKind::Detection,
        target: AlgorithmTargetKind::Flame,
        inputs: &[AlgorithmInputKind::Image],
        outputs: &[
            AlgorithmOutputKind::BoundingBox,
            AlgorithmOutputKind::Confidence,
            AlgorithmOutputKind::ClassLabel,
        ],
        description: "检测图片或视频帧中的火焰目标。",
    };
