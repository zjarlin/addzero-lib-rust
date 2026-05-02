use addzero_admin_plugin_registry as registry;
pub use addzero_admin_plugin_registry::AdminDomainRegistration;
use dioxus_components::{AdminMenu, AdminSection};

use crate::admin::domains::{
    AGENTS_DOMAIN_ID, CHAT_DOMAIN_ID, KNOWLEDGE_DOMAIN_ID, OVERVIEW_DOMAIN_ID, SYSTEM_DOMAIN_ID,
};
use crate::app::Route;

const AUDIT_DOMAIN_ID: &str = "audit";

#[derive(Clone, Copy)]
struct StaticPage {
    domain_id: &'static str,
    label: &'static str,
    href: &'static str,
    order: u16,
    active_patterns: &'static [&'static str],
}

pub fn primary_domain() -> Option<AdminDomainRegistration> {
    if registry_is_complete() {
        registry::primary_domain()
    } else {
        fallback_domains().into_iter().next()
    }
}

pub fn domain_for_route(route: &Route) -> Option<AdminDomainRegistration> {
    if registry_is_complete() {
        registry::domain_for_path(route.to_string().as_str())
    } else {
        let path = route.to_string();
        let page = fallback_pages()
            .into_iter()
            .find(|page| registry::path_matches_patterns(path.as_str(), page.active_patterns))?;
        fallback_domains()
            .into_iter()
            .find(|domain| domain.id == page.domain_id)
    }
}

pub fn registered_domains() -> Vec<AdminDomainRegistration> {
    if registry_is_complete() {
        registry::registered_domains()
    } else {
        fallback_domains()
    }
}

pub fn section_for_route(route: &Route) -> Option<AdminSection<Route>> {
    if registry_is_complete() {
        let section = registry::section_for_path(route.to_string().as_str())?;
        let menus = section
            .menus
            .into_iter()
            .filter_map(|menu| {
                let to = menu.href.parse::<Route>().ok()?;
                let patterns = menu.active_patterns;
                Some(AdminMenu::leaf(menu.label, to, move |route| {
                    registry::path_matches_patterns(route.to_string().as_str(), patterns)
                }))
            })
            .collect();

        Some(AdminSection {
            label: section.label.to_string(),
            menus,
        })
    } else {
        let path = route.to_string();
        let page = fallback_pages()
            .into_iter()
            .find(|page| registry::path_matches_patterns(path.as_str(), page.active_patterns))?;
        let domain = fallback_domains()
            .into_iter()
            .find(|domain| domain.id == page.domain_id)?;
        let mut domain_pages: Vec<_> = fallback_pages()
            .into_iter()
            .filter(|item| item.domain_id == domain.id)
            .collect();
        domain_pages.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then(left.label.cmp(right.label))
                .then(left.href.cmp(right.href))
        });
        let menus = domain_pages
            .into_iter()
            .filter_map(|item| {
                let to = item.href.parse::<Route>().ok()?;
                let patterns = item.active_patterns;
                Some(AdminMenu::leaf(item.label, to, move |route| {
                    registry::path_matches_patterns(route.to_string().as_str(), patterns)
                }))
            })
            .collect();

        Some(AdminSection {
            label: domain.label.to_string(),
            menus,
        })
    }
}

fn registry_is_complete() -> bool {
    let fallback_domains = fallback_domains();
    let fallback_pages = fallback_pages();
    let domains = registry::registered_domains();

    if domains.len() < fallback_domains.len() {
        return false;
    }

    if !fallback_domains
        .iter()
        .all(|expected| domains.iter().any(|domain| domain.id == expected.id))
    {
        return false;
    }

    let menu_count: usize = domains
        .iter()
        .filter_map(|domain| registry::section_for_path(domain.default_href))
        .map(|section| section.menus.len())
        .sum();

    menu_count >= fallback_pages.len()
}

fn fallback_domains() -> Vec<AdminDomainRegistration> {
    vec![
        AdminDomainRegistration {
            id: OVERVIEW_DOMAIN_ID,
            label: "总览",
            order: 10,
            default_href: "/",
        },
        AdminDomainRegistration {
            id: AGENTS_DOMAIN_ID,
            label: "Agent资产",
            order: 20,
            default_href: "/agents",
        },
        AdminDomainRegistration {
            id: CHAT_DOMAIN_ID,
            label: "AI 聊天",
            order: 30,
            default_href: "/chat",
        },
        AdminDomainRegistration {
            id: KNOWLEDGE_DOMAIN_ID,
            label: "知识库",
            order: 40,
            default_href: "/knowledge/notes",
        },
        AdminDomainRegistration {
            id: SYSTEM_DOMAIN_ID,
            label: "系统管理",
            order: 50,
            default_href: "/system/users",
        },
        AdminDomainRegistration {
            id: AUDIT_DOMAIN_ID,
            label: "审计日志",
            order: 60,
            default_href: "/audit",
        },
    ]
}

fn fallback_pages() -> Vec<StaticPage> {
    vec![
        StaticPage {
            domain_id: OVERVIEW_DOMAIN_ID,
            label: "知识图谱概览",
            href: "/",
            order: 10,
            active_patterns: &["/", "/dashboard"],
        },
        StaticPage {
            domain_id: AGENTS_DOMAIN_ID,
            label: "技能资产",
            href: "/agents",
            order: 10,
            active_patterns: &["/agents", "/agents/:name"],
        },
        StaticPage {
            domain_id: CHAT_DOMAIN_ID,
            label: "聊天工作台",
            href: "/chat",
            order: 10,
            active_patterns: &["/chat"],
        },
        StaticPage {
            domain_id: KNOWLEDGE_DOMAIN_ID,
            label: "笔记",
            href: "/knowledge/notes",
            order: 10,
            active_patterns: &["/knowledge/notes"],
        },
        StaticPage {
            domain_id: KNOWLEDGE_DOMAIN_ID,
            label: "软件",
            href: "/knowledge/software",
            order: 20,
            active_patterns: &["/knowledge/software"],
        },
        StaticPage {
            domain_id: KNOWLEDGE_DOMAIN_ID,
            label: "下载与安装",
            href: "/knowledge/packages",
            order: 30,
            active_patterns: &["/knowledge/packages", "/files"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "用户",
            href: "/system/users",
            order: 10,
            active_patterns: &["/system/users"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "菜单",
            href: "/system/menus",
            order: 20,
            active_patterns: &["/system/menus"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "角色",
            href: "/system/roles",
            order: 30,
            active_patterns: &["/system/roles"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "部门",
            href: "/system/departments",
            order: 40,
            active_patterns: &["/system/departments"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "字典管理",
            href: "/system/dictionaries",
            order: 45,
            active_patterns: &["/system/dictionaries"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "Agent 节点",
            href: "/system/agent-nodes",
            order: 48,
            active_patterns: &["/system/agent-nodes", "/system/agent-nodes/pairings/:id"],
        },
        StaticPage {
            domain_id: SYSTEM_DOMAIN_ID,
            label: "系统设置",
            href: "/system/settings",
            order: 50,
            active_patterns: &["/system/settings"],
        },
        StaticPage {
            domain_id: AUDIT_DOMAIN_ID,
            label: "审计日志",
            href: "/audit",
            order: 10,
            active_patterns: &["/audit"],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{domain_for_route, registered_domains, section_for_route};
    use crate::app::Route;

    #[test]
    fn registered_domains_should_follow_plugin_order() {
        let ids: Vec<_> = registered_domains()
            .into_iter()
            .map(|domain| domain.id)
            .collect();

        assert_eq!(
            ids,
            vec!["overview", "agents", "chat", "knowledge", "system", "audit"]
        );
    }

    #[test]
    fn section_for_route_should_resolve_registered_pages() {
        let section = section_for_route(&Route::KnowledgeNotes).expect("knowledge section");
        let labels: Vec<_> = section
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect();
        let files_section = section_for_route(&Route::Files).expect("files alias section");

        assert_eq!(section.label, "知识库");
        assert_eq!(labels, vec!["笔记", "软件", "下载与安装"]);
        assert_eq!(files_section.label, "知识库");
        assert_eq!(
            domain_for_route(&Route::SystemUsers)
                .expect("system domain")
                .label,
            "系统管理"
        );
        assert_eq!(
            domain_for_route(&Route::Files)
                .expect("files alias domain")
                .label,
            "知识库"
        );
    }
}

#[macro_export]
macro_rules! register_admin_domain {
    (
        id: $id:expr,
        label: $label:expr,
        order: $order:expr,
        default_href: $default_href:expr $(,)?
    ) => {
        ::addzero_admin_plugin_registry::register_admin_domain! {
            id: $id,
            label: $label,
            order: $order,
            default_href: $default_href,
        }
    };
}

#[macro_export]
macro_rules! register_admin_page {
    (
        domain: $domain_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr $(,)?
    ) => {
        ::addzero_admin_plugin_registry::register_admin_page! {
            domain: $domain_id,
            label: $label,
            order: $order,
            href: $href,
            active_patterns: $active_patterns,
        }
    };
}
