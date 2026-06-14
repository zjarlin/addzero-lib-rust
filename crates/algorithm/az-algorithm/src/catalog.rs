//! 算法组件目录与输入输出契约。
//!
//! 算法组件在本 crate 内以静态规格声明，查询函数直接读取固定组件列表。

mod model;
pub use model::{
    AlgorithmComponentDescriptor, AlgorithmComponentKind, AlgorithmComponentSpec,
    AlgorithmInputKind, AlgorithmOutputKind, AlgorithmTargetKind, AlgorithmTaskKind,
};

mod query;
pub use query::*;

// 算法组件注册文件。
mod face_detection;
mod face_recognition;
mod flame_detection;
mod ocr_text_recognition;
mod person_detection;
mod qr_code_recognition;
mod safety_helmet_detection;
mod vehicle_detection;
mod worker_hit_counting;

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
