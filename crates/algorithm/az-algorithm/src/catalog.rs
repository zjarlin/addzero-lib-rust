//! 算法组件目录与输入输出契约。
//!
//! 算法组件在本 crate 内以静态规格声明，查询函数直接读取固定组件列表。

automod::dir!("src/catalog");

pub use model::{
    AlgorithmComponentDescriptor, AlgorithmComponentKind, AlgorithmComponentSpec,
    AlgorithmInputKind, AlgorithmOutputKind, AlgorithmTargetKind, AlgorithmTaskKind,
};

pub use query::*;

const COMPONENTS: &[AlgorithmComponentSpec] = &[
    face_detection::SPEC,
    face_recognition::SPEC,
    person_detection::SPEC,
    ocr_text_recognition::SPEC,
    flame_detection::SPEC,
    safety_helmet_detection::SPEC,
    vehicle_detection::SPEC,
    qr_code_recognition::SPEC,
    worker_hit_counting::SPEC,
];
