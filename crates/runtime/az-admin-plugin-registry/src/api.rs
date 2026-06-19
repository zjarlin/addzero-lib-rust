//! Admin 插件注册表 API，基于 `inventory` 实现编译期声明式注册。
//!
//! 本 crate 为 admin 工作面提供**双轴上下文导航树**的注册与查询基础设施。
//! 各业务域通过 `register_admin_domain!`、`register_admin_branch!`、
//! `register_admin_page!`、`register_admin_root_page!` 宏在编译期声明自己的域、分支和页面节点，
//! starter 插件可通过 `declare_admin_plugin!` 或 `declare_admin_root_page_plugin!`
//! 暴露链接保活入口。
//! 运行时通过 `registered_domains()`、`section_for_path()` 等函数
//! 按排序和路径匹配组装出完整的导航树。
//!
//! ## 核心类型
//!
//! - [`AdminDomainRegistration`] — 主轴上下文域（业务域/产品壳）
//! - [`AdminNavigationRegistration`] — 侧轴导航节点（分支或页面）
//! - [`RegisteredAdminNode`] — 带子节点的递归导航树节点
//! - [`RegisteredAdminSection`] — 某域下的完整菜单区段
//!
//! ## 关键特性
//!
//! - 支持动态路径段匹配（`:param` 风格）
//! - 按 `order → label → id` 排序保证导航顺序稳定
//! - 权限过滤通过 `permissions_any_of` 字段声明
//! - 空域（无节点）自动从注册列表中移除

use std::collections::BTreeMap;

use az_str::api::{normalize_url_path, split_url_path_segments};
pub use inventory;

/// Admin 主轴上下文域注册项。
///
/// 一个 domain 通常对应业务域、产品壳或大型路由组；只有至少挂载一个导航节点的
/// domain 会出现在运行时查询结果中。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdminDomainRegistration {
    /// 全局唯一的 domain 标识。
    pub id: &'static str,
    /// 展示给 admin shell 的中文或业务标签。
    pub label: &'static str,
    /// 主轴排序值，数值越小越靠前。
    pub order: u16,
    /// 当前 domain 被选中但没有更具体路径时使用的默认入口。
    pub default_href: &'static str,
}

/// Admin 侧轴导航节点类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum AdminNavigationKind {
    /// 可展开分组节点，通常用于承载一组子页面。
    Branch,
    /// 可直接打开的页面节点。
    Page,
}

impl AdminNavigationKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// Admin 侧轴导航注册项。
///
/// 所有节点都属于某个 domain，并可通过 `parent_id` 组成树形菜单。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdminNavigationRegistration {
    /// 节点类型。
    pub kind: AdminNavigationKind,
    /// 当前节点在所属 domain 内的唯一标识。
    pub id: &'static str,
    /// 所属主轴 domain 标识。
    pub domain_id: &'static str,
    /// 父节点标识；`None` 表示挂到 domain 根部。
    pub parent_id: Option<&'static str>,
    /// 展示给 admin shell 的节点标签。
    pub label: &'static str,
    /// 同级排序值，数值越小越靠前。
    pub order: u16,
    /// 点击节点时进入的路由。
    pub href: &'static str,
    /// 判定当前路由是否命中该节点的路径模式，支持 `:param` 动态段。
    pub active_patterns: &'static [&'static str],
    /// 访问该节点任一所需权限；空切片表示不声明权限要求。
    pub permissions_any_of: &'static [&'static str],
}

/// 运行时组装后的递归导航节点。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredAdminNode {
    /// 节点唯一标识。
    pub id: &'static str,
    /// 节点类型。
    pub kind: AdminNavigationKind,
    /// 展示标签。
    pub label: &'static str,
    /// 目标路由。
    pub href: &'static str,
    /// 激活匹配模式。
    pub active_patterns: &'static [&'static str],
    /// 访问权限声明。
    pub permissions_any_of: &'static [&'static str],
    /// 子节点，已经按稳定排序规则排好序。
    pub children: Vec<RegisteredAdminNode>,
}

/// 某个 domain 下可直接交给 admin shell 渲染的菜单区段。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredAdminSection {
    /// domain 展示标签。
    pub label: &'static str,
    /// domain 默认入口路由。
    pub default_href: &'static str,
    /// 当前 domain 的根级菜单节点。
    pub menus: Vec<RegisteredAdminNode>,
}

inventory::collect!(AdminDomainRegistration);
inventory::collect!(AdminNavigationRegistration);

/// 返回排序后的第一个可见 admin domain。
pub fn primary_domain() -> Option<AdminDomainRegistration> {
    registered_domains().into_iter().next()
}

/// 根据当前路径反查命中的 admin domain。
pub fn domain_for_path(path: &str) -> Option<AdminDomainRegistration> {
    let node = active_node(path)?;
    domain_by_id(node.domain_id)
}

/// 返回所有已注册且至少包含一个导航节点的 admin domain。
pub fn registered_domains() -> Vec<AdminDomainRegistration> {
    let mut domains: Vec<_> = inventory::iter::<AdminDomainRegistration>
        .into_iter()
        .copied()
        .collect();
    domains.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(right.label))
            .then(left.id.cmp(right.id))
    });
    domains.retain(|domain| has_nodes_for_domain(domain.id));
    domains
}

/// 根据当前路径组装所属 domain 的完整菜单区段。
pub fn section_for_path(path: &str) -> Option<RegisteredAdminSection> {
    let domain = domain_for_path(path)?;
    Some(RegisteredAdminSection {
        label: domain.label,
        default_href: domain.default_href,
        menus: navigation_tree_for_domain(domain.id),
    })
}

/// 判断路径是否命中任一路由模式。
///
/// 模式支持 `:param` 形式的动态段；查询字符串和 hash 会先被规范化移除。
pub fn path_matches_patterns(path: &str, patterns: &[&str]) -> bool {
    let path = normalize_url_path(path);
    patterns
        .iter()
        .copied()
        .any(|pattern| path_matches_pattern(&path, pattern))
}

fn active_node(path: &str) -> Option<AdminNavigationRegistration> {
    all_nodes()
        .into_iter()
        .find(|node| path_matches_patterns(path, node.active_patterns))
}

fn domain_by_id(domain_id: &str) -> Option<AdminDomainRegistration> {
    inventory::iter::<AdminDomainRegistration>
        .into_iter()
        .copied()
        .find(|domain| domain.id == domain_id)
}

fn all_nodes() -> Vec<AdminNavigationRegistration> {
    let mut nodes: Vec<_> = inventory::iter::<AdminNavigationRegistration>
        .into_iter()
        .copied()
        .filter(|node| domain_by_id(node.domain_id).is_some())
        .collect();
    nodes.sort_by(|left, right| navigation_sort_key(*left).cmp(&navigation_sort_key(*right)));
    nodes
}

fn navigation_tree_for_domain(domain_id: &str) -> Vec<RegisteredAdminNode> {
    let nodes: Vec<_> = all_nodes()
        .into_iter()
        .filter(|node| node.domain_id == domain_id)
        .collect();
    let mut children_by_parent: BTreeMap<Option<&'static str>, Vec<AdminNavigationRegistration>> =
        BTreeMap::new();
    for node in nodes {
        children_by_parent
            .entry(node.parent_id)
            .or_default()
            .push(node);
    }
    build_children(None, &children_by_parent)
}

fn build_children(
    parent_id: Option<&'static str>,
    children_by_parent: &BTreeMap<Option<&'static str>, Vec<AdminNavigationRegistration>>,
) -> Vec<RegisteredAdminNode> {
    let Some(children) = children_by_parent.get(&parent_id) else {
        return Vec::new();
    };

    children
        .iter()
        .copied()
        .map(|node| RegisteredAdminNode {
            id: node.id,
            kind: node.kind,
            label: node.label,
            href: node.href,
            active_patterns: node.active_patterns,
            permissions_any_of: node.permissions_any_of,
            children: build_children(Some(node.id), children_by_parent),
        })
        .collect()
}

fn has_nodes_for_domain(domain_id: &str) -> bool {
    inventory::iter::<AdminNavigationRegistration>
        .into_iter()
        .any(|node| node.domain_id == domain_id)
}

fn navigation_sort_key(
    node: AdminNavigationRegistration,
) -> (
    u16,
    &'static str,
    &'static str,
    Option<&'static str>,
    u16,
    &'static str,
    &'static str,
) {
    let (domain_order, domain_label, domain_key) = domain_sort_key(node.domain_id);
    (
        domain_order,
        domain_label,
        domain_key,
        node.parent_id,
        node.order,
        node.label,
        node.id,
    )
}

fn domain_sort_key(domain_id: &'static str) -> (u16, &'static str, &'static str) {
    if let Some(domain) = domain_by_id(domain_id) {
        (domain.order, domain.label, domain.id)
    } else {
        (u16::MAX, "", domain_id)
    }
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    let path = split_url_path_segments(path);
    let pattern = split_url_path_segments(pattern);
    if path.len() != pattern.len() {
        return false;
    }

    path.iter()
        .zip(pattern.iter())
        .all(|(segment, matcher)| matcher.starts_with(':') || segment == matcher)
}

/// 注册 admin 主轴上下文域。
///
/// 该宏通过 `inventory` 在编译期收集注册项。调用方只声明 domain 本身；
/// 如果没有任何导航节点引用该 domain，运行时查询会自动过滤它。
#[macro_export]
macro_rules! register_admin_domain {
    (
        id: $id:expr,
        label: $label:expr,
        order: $order:expr,
        default_href: $default_href:expr $(,)?
    ) => {
        $crate::api::inventory::submit! {
            $crate::api::AdminDomainRegistration {
                id: $id,
                label: $label,
                order: $order,
                default_href: $default_href,
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __register_admin_navigation {
    (
        kind: $kind:expr,
        id: $id:expr,
        domain: $domain_id:expr,
        parent: $parent_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr,
        permissions_any_of: $permissions_any_of:expr $(,)?
    ) => {
        $crate::api::inventory::submit! {
            $crate::api::AdminNavigationRegistration {
                kind: $kind,
                id: $id,
                domain_id: $domain_id,
                parent_id: $parent_id,
                label: $label,
                order: $order,
                href: $href,
                active_patterns: $active_patterns,
                permissions_any_of: $permissions_any_of,
            }
        }
    };
}

/// 注册 admin 侧轴分支节点。
///
/// 分支节点用于组织子页面，仍然可以携带 `href` 作为默认落点。
#[macro_export]
macro_rules! register_admin_branch {
    (
        id: $id:expr,
        domain: $domain_id:expr,
        parent: $parent_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr,
        permissions_any_of: $permissions_any_of:expr $(,)?
    ) => {
        $crate::__register_admin_navigation! {
            kind: $crate::api::AdminNavigationKind::Branch,
            id: $id,
            domain: $domain_id,
            parent: $parent_id,
            label: $label,
            order: $order,
            href: $href,
            active_patterns: $active_patterns,
            permissions_any_of: $permissions_any_of,
        }
    };
}

/// 注册 admin 侧轴页面节点。
///
/// 页面节点是 admin 内容区的真实入口，所属 `(domain, parent)` 决定它在双轴导航树中的位置。
#[macro_export]
macro_rules! register_admin_page {
    (
        id: $id:expr,
        domain: $domain_id:expr,
        parent: $parent_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr,
        permissions_any_of: $permissions_any_of:expr $(,)?
    ) => {
        $crate::__register_admin_navigation! {
            kind: $crate::api::AdminNavigationKind::Page,
            id: $id,
            domain: $domain_id,
            parent: $parent_id,
            label: $label,
            order: $order,
            href: $href,
            active_patterns: $active_patterns,
            permissions_any_of: $permissions_any_of,
        }
    };
}

/// 注册挂在 domain 根部的页面节点。
///
/// 这是 `register_admin_page!` 的浅层便捷包装，默认 `parent: None`、
/// `active_patterns: &[href]` 且不声明权限要求。
#[macro_export]
macro_rules! register_admin_root_page {
    (
        id: $id:expr,
        domain: $domain_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr $(,)?
    ) => {
        $crate::register_admin_page! {
            id: $id,
            domain: $domain_id,
            parent: None,
            label: $label,
            order: $order,
            href: $href,
            active_patterns: &[$href],
            permissions_any_of: &[],
        }
    };
}

/// 为 starter 插件声明一个链接保活入口。
///
/// host crate 可调用生成的 `ensure_linked()`，确保只靠 inventory 注册副作用的插件 crate
/// 不会在链接阶段被当作未使用依赖剔除。
#[macro_export]
macro_rules! declare_admin_plugin {
    () => {
        pub fn ensure_linked() {}
    };
}

/// 声明挂在 domain 根部的 starter 页面插件。
///
/// 这是 `register_admin_root_page!` 和 `declare_admin_plugin!` 的浅层组合，适合只暴露
/// 一个根级 admin 页面入口的 starter crate。复杂插件仍应直接使用更底层的注册宏。
#[macro_export]
macro_rules! declare_admin_root_page_plugin {
    (
        id: $id:expr,
        domain: $domain_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr $(,)?
    ) => {
        $crate::register_admin_root_page! {
            id: $id,
            domain: $domain_id,
            label: $label,
            order: $order,
            href: $href,
        }

        $crate::declare_admin_plugin!();
    };
}
