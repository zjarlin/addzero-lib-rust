use crate::catalog::model::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

pub const SPEC: AlgorithmComponentSpec = AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::OcrTextRecognition,
        label: "OCR文字识别",
        task: AlgorithmTaskKind::Recognition,
        target: AlgorithmTargetKind::Text,
        inputs: &[AlgorithmInputKind::Image, AlgorithmInputKind::RegionOfInterest],
        outputs: &[AlgorithmOutputKind::BoundingBox, AlgorithmOutputKind::Text],
        description: "识别图片中的文字区域并输出文本内容。",
    };
