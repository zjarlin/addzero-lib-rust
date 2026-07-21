use crate::catalog::model::{
    AlgorithmComponentKind, AlgorithmComponentSpec, AlgorithmInputKind, AlgorithmOutputKind,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

pub const SPEC: AlgorithmComponentSpec = AlgorithmComponentSpec {
        kind: AlgorithmComponentKind::WorkerHitCounting,
        label: "工人敲击计数",
        task: AlgorithmTaskKind::Counting,
        target: AlgorithmTargetKind::WorkerHit,
        inputs: &[
            AlgorithmInputKind::VideoFrames,
            AlgorithmInputKind::PersonTracks,
            AlgorithmInputKind::ActionScores,
            AlgorithmInputKind::TargetObservations,
            AlgorithmInputKind::ContactPoints,
        ],
        outputs: &[
            AlgorithmOutputKind::PersonTrackId,
            AlgorithmOutputKind::ActionState,
            AlgorithmOutputKind::EventCount,
            AlgorithmOutputKind::EventTimestamp,
            AlgorithmOutputKind::TargetId,
            AlgorithmOutputKind::ContactPoint,
            AlgorithmOutputKind::InvalidReason,
            AlgorithmOutputKind::Confidence,
        ],
        description:
            "基于人员轨迹、接触点、目标类型和目标响应，按每个人分别统计有效敲击和无效候选。",
    };
