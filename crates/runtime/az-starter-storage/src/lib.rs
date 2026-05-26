//! 文件中心系统导航注册。
//!
//! 本 crate 只负责把包仓库入口注册到 admin 双轴导航树中。
//! 具体页面数据由 admin 应用侧 provider 按路由加载。

const SYSTEM_DOMAIN_ID: &str = "system";
const STORAGE_PACKAGES_NODE_ID: &str = "system-storage-packages";

az_admin_plugin_registry::register_admin_page! {
    id: STORAGE_PACKAGES_NODE_ID,
    domain: SYSTEM_DOMAIN_ID,
    parent: None,
    label: "包仓库",
    order: 60,
    href: "/system/storage/packages",
    active_patterns: &["/system/storage/packages"],
    permissions_any_of: &[],
}

pub fn ensure_linked() {}
