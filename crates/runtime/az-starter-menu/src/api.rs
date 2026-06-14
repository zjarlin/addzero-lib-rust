//! 菜单中心系统导航注册。
//!
//! 本 crate 只负责把菜单挂载说明入口注册到 admin 双轴导航树中。
//! 具体页面数据由 admin 应用侧 provider 按路由加载。

const SYSTEM_DOMAIN_ID: &str = "system";
const MENU_MOUNTING_NODE_ID: &str = "system-menu-mounting";

az_admin_plugin_registry::declare_admin_root_page_plugin! {
    id: MENU_MOUNTING_NODE_ID,
    domain: SYSTEM_DOMAIN_ID,
    label: "菜单挂载",
    order: 40,
    href: "/system/menu/mounting",
}
