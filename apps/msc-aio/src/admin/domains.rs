pub const OVERVIEW_DOMAIN_ID: &str = "overview";
pub const AGENTS_DOMAIN_ID: &str = "agents";
pub const CHAT_DOMAIN_ID: &str = "chat";
pub const KNOWLEDGE_DOMAIN_ID: &str = "knowledge";
pub const SYSTEM_DOMAIN_ID: &str = "system";

crate::register_admin_domain! {
    id: OVERVIEW_DOMAIN_ID,
    label: "总览",
    order: 10,
    default_href: "/",
}

crate::register_admin_domain! {
    id: AGENTS_DOMAIN_ID,
    label: "Agent资产",
    order: 20,
    default_href: "/agents",
}

crate::register_admin_domain! {
    id: CHAT_DOMAIN_ID,
    label: "AI 聊天",
    order: 30,
    default_href: "/chat",
}

crate::register_admin_domain! {
    id: KNOWLEDGE_DOMAIN_ID,
    label: "知识库",
    order: 40,
    default_href: "/knowledge/notes",
}

crate::register_admin_domain! {
    id: SYSTEM_DOMAIN_ID,
    label: "系统管理",
    order: 50,
    default_href: "/system/users",
}
