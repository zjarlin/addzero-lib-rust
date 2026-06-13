//! 算法组件目录与输入输出契约。
//!
//! 各算法组件通过 [`inventory`] 在编译期声明式注册，运行时通过本模块的查询函数发现。
//!
//! 新增算法组件时，在 `src/catalog/` 下添加一个新文件并在此处声明 `mod xxx;`，
//! 用 [`inventory::submit!`] 注册 [`AlgorithmComponentSpec`] 即可，无需维护集中式清单。

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

// inventory 注册收集点。
//
// 所有通过 `inventory::submit!(AlgorithmComponentSpec { ... })` 注册的组件
// 会在此处被收集，供查询函数遍历。
inventory::collect!(AlgorithmComponentSpec);
