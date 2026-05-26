//! 字典中心系统导航注册。
//!
//! 本 crate 只负责把字典管理入口注册到 admin 双轴导航树中。
//! 具体页面数据由 admin 应用侧 provider 按路由加载。

const SYSTEM_DOMAIN_ID: &str = "system";
const DICTIONARY_NODE_ID: &str = "system-dictionary-note-types";

az_admin_plugin_registry::register_admin_page! {
    id: DICTIONARY_NODE_ID,
    domain: SYSTEM_DOMAIN_ID,
    parent: None,
    label: "字典管理",
    order: 30,
    href: "/system/dictionary/note-types",
    active_patterns: &["/system/dictionary/note-types"],
    permissions_any_of: &[],
}

pub fn ensure_linked() {}
