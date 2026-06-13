use crate::catalog::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

inventory::submit! {
    AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::SafetyHelmetDetection,
        label: "安全帽检测",
        task: AlgorithmTaskKind::Detection,
        target: AlgorithmTargetKind::SafetyHelmet,
        inputs: &[AlgorithmInputKind::Image],
        outputs: &[
            AlgorithmOutputKind::BoundingBox,
            AlgorithmOutputKind::Confidence,
            AlgorithmOutputKind::ClassLabel,
        ],
        description: "检测人员头部安全帽佩戴相关目标。",
    }
}
