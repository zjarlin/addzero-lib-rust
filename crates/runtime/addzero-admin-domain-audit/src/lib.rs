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
