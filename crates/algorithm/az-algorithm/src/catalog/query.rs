//! 算法组件查询接口。

use crate::catalog::{
    face_detection, face_recognition, flame_detection,
    model::{
        AlgorithmComponentDescriptor, AlgorithmComponentKind, AlgorithmComponentSpec,
        AlgorithmTargetKind, AlgorithmTaskKind,
    },
    ocr_text_recognition, person_detection, qr_code_recognition, safety_helmet_detection,
    vehicle_detection, worker_hit_counting,
};

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

/// 返回全部算法组件规格。
#[must_use]
pub fn algorithm_components() -> Vec<&'static AlgorithmComponentSpec> {
    COMPONENTS.iter().collect()
}

/// 根据稳定 code 查找算法组件。
#[must_use]
pub fn algorithm_component_by_code(code: &str) -> Option<&'static AlgorithmComponentSpec> {
    AlgorithmComponentKind::from_code(code).and_then(AlgorithmComponentKind::spec)
}

/// 根据中文名称查找算法组件。
#[must_use]
pub fn algorithm_component_by_label(label: &str) -> Option<&'static AlgorithmComponentSpec> {
    COMPONENTS.iter().find(|component| component.label == label)
}

/// 按任务类型过滤算法组件。
pub fn algorithm_components_by_task(
    task: AlgorithmTaskKind,
) -> impl Iterator<Item = &'static AlgorithmComponentSpec> {
    COMPONENTS
        .iter()
        .filter(move |component| component.task == task)
}

/// 按目标对象过滤算法组件。
pub fn algorithm_components_by_target(
    target: AlgorithmTargetKind,
) -> impl Iterator<Item = &'static AlgorithmComponentSpec> {
    COMPONENTS
        .iter()
        .filter(move |component| component.target == target)
}

/// 返回全部组件的可序列化描述。
#[must_use]
pub fn algorithm_component_descriptors() -> Vec<AlgorithmComponentDescriptor> {
    COMPONENTS
        .iter()
        .map(AlgorithmComponentSpec::to_descriptor)
        .collect()
}
