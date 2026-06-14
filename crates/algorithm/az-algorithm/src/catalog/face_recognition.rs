use crate::catalog::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

pub const SPEC: AlgorithmComponentSpec = AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::FaceRecognition,
        label: "人脸识别",
        task: AlgorithmTaskKind::Recognition,
        target: AlgorithmTargetKind::Face,
        inputs: &[AlgorithmInputKind::Image, AlgorithmInputKind::ReferenceSet],
        outputs: &[
            AlgorithmOutputKind::BoundingBox,
            AlgorithmOutputKind::Confidence,
            AlgorithmOutputKind::Identity,
        ],
        description: "将检测到的人脸与人脸底库匹配并输出身份结果。",
    };
