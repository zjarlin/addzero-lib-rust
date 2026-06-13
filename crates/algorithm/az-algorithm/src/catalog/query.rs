//! 算法组件查询接口。

use super::model::{
    AlgorithmComponentDescriptor, AlgorithmComponentKind, AlgorithmComponentSpec,
    AlgorithmTargetKind, AlgorithmTaskKind,
};

/// 返回全部算法组件规格。
#[must_use]
pub fn algorithm_components() -> Vec<&'static AlgorithmComponentSpec> {
    inventory::iter::<AlgorithmComponentSpec>
        .into_iter()
        .collect()
}

/// 根据稳定 code 查找算法组件。
#[must_use]
pub fn algorithm_component_by_code(code: &str) -> Option<&'static AlgorithmComponentSpec> {
    AlgorithmComponentKind::from_code(code).and_then(AlgorithmComponentKind::spec)
}

/// 根据中文名称查找算法组件。
#[must_use]
pub fn algorithm_component_by_label(label: &str) -> Option<&'static AlgorithmComponentSpec> {
    inventory::iter::<AlgorithmComponentSpec>
        .into_iter()
        .find(|component| component.label == label)
}

/// 按任务类型过滤算法组件。
pub fn algorithm_components_by_task(
    task: AlgorithmTaskKind,
) -> impl Iterator<Item = &'static AlgorithmComponentSpec> {
    inventory::iter::<AlgorithmComponentSpec>
        .into_iter()
        .filter(move |component| component.task == task)
}

/// 按目标对象过滤算法组件。
pub fn algorithm_components_by_target(
    target: AlgorithmTargetKind,
) -> impl Iterator<Item = &'static AlgorithmComponentSpec> {
    inventory::iter::<AlgorithmComponentSpec>
        .into_iter()
        .filter(move |component| component.target == target)
}

/// 返回全部组件的可序列化描述。
#[must_use]
pub fn algorithm_component_descriptors() -> Vec<AlgorithmComponentDescriptor> {
    inventory::iter::<AlgorithmComponentSpec>
        .into_iter()
        .map(AlgorithmComponentSpec::to_descriptor)
        .collect()
}
