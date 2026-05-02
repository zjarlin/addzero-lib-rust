use dioxus::prelude::*;
use dioxus_components::{ContentHeader, ListItem, Stack, Surface, SurfaceHeader};

pub const AUDIT_DOMAIN_ID: &str = "audit";

pub fn ensure_linked() {}

addzero_admin_plugin_registry::register_admin_domain! {
    id: AUDIT_DOMAIN_ID,
    label: "审计日志",
    order: 60,
    default_href: "/audit",
}

addzero_admin_plugin_registry::register_admin_page! {
    id: "audit-log",
    domain: AUDIT_DOMAIN_ID,
    parent: None,
    label: "审计日志",
    order: 10,
    href: "/audit",
    active_patterns: &["/audit"],
    permissions_any_of: &["audit"],
}

#[component]
pub fn AuditPage() -> Element {
    rsx! {
        ContentHeader {
            title: "审计日志".to_string(),
            subtitle: "右侧上下文栏继续承担辅助理解，不抢主内容权重。".to_string()
        }
        Surface {
            SurfaceHeader {
                title: "最近日志".to_string(),
                subtitle: "先有时间线，再接过滤器和详情抽屉。".to_string()
            }
            Stack {
                ListItem { title: "09:12 审计任务结束".to_string(), detail: "策略审计通过，没有新增风险项".to_string(), meta: "system".to_string() }
                ListItem { title: "08:41 权限模板变更".to_string(), detail: "P-09 删除了一条遗留白名单".to_string(), meta: "Chen".to_string() }
                ListItem { title: "昨天 18:20 发布完成".to_string(), detail: "release-0426 已部署到 production".to_string(), meta: "Luna".to_string() }
            }
        }
    }
}
