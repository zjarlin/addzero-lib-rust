use crate::catalog::model::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

pub const SPEC: AlgorithmComponentSpec = AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::QrCodeRecognition,
        label: "二维码识别",
        task: AlgorithmTaskKind::Recognition,
        target: AlgorithmTargetKind::QrCode,
        inputs: &[AlgorithmInputKind::Image, AlgorithmInputKind::RegionOfInterest],
        outputs: &[
            AlgorithmOutputKind::BoundingBox,
            AlgorithmOutputKind::QrPayload,
            AlgorithmOutputKind::Confidence,
        ],
        description: "识别图片中的二维码区域并输出解码载荷。",
    };
