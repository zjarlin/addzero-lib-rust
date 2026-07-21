//! 用户中心系统导航注册。
//!
//! 本 crate 只负责把用户管理入口注册到 admin 双轴导航树中。
//! 具体页面数据由 admin 应用侧 provider 按路由加载。

const SYSTEM_DOMAIN_ID: &str = "system";
const IDENTITY_USERS_NODE_ID: &str = "system-identity-users";

az_admin_plugin_registry::declare_admin_root_page_plugin! {
    id: IDENTITY_USERS_NODE_ID,
    domain: SYSTEM_DOMAIN_ID,
    label: "用户管理",
    order: 10,
    href: "/system/identity/users",
}
