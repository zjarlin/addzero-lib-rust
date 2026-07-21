use crate::catalog::model::{
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
            AlgorithmOutputKind::Identity,
            AlgorithmOutputKind::SimilarityScore,
        ],
        description: "将待识别人脸与参考人脸或人脸底库匹配并输出相似度与身份结果。",
    };
