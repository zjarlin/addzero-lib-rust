//! 审计中心系统导航注册。
//!
//! 本 crate 只负责把审计日志入口注册到 admin 双轴导航树中。
//! 具体页面数据由 admin 应用侧 provider 按路由加载。

const SYSTEM_DOMAIN_ID: &str = "system";
const AUDIT_EVENTS_NODE_ID: &str = "system-audit-events";

az_admin_plugin_registry::declare_admin_root_page_plugin! {
    id: AUDIT_EVENTS_NODE_ID,
    domain: SYSTEM_DOMAIN_ID,
    label: "审计日志",
    order: 50,
    href: "/system/audit/events",
}
