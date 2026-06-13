use crate::catalog::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

inventory::submit! {
    AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::PersonDetection,
        label: "人员检测",
        task: AlgorithmTaskKind::Detection,
        target: AlgorithmTargetKind::Person,
        inputs: &[AlgorithmInputKind::Image],
        outputs: &[
            AlgorithmOutputKind::BoundingBox,
            AlgorithmOutputKind::Confidence,
            AlgorithmOutputKind::ClassLabel,
        ],
        description: "在图片或视频帧中定位人员目标。",
    }
}
