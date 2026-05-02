use dioxus::prelude::*;
use dioxus_components::{
    AdminAction, AdminActionIcon, AdminCommand, AdminMenu, AdminProvider, AdminSection,
    AdminTopbar, WorkbenchButton,
};

use crate::app::Route;
use crate::services::BrandingLogoSource;
use crate::services::SharedAuthApi;
use crate::state::{AuthSession, BrandingPrefs, BrandingState, PermissionState, ThemePrefs};

pub struct DefaultAdminProvider {
    auth: AuthSession,
    theme: ThemePrefs,
    branding: BrandingPrefs,
    auth_api: SharedAuthApi,
    permissions: PermissionState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdminDomain {
    Overview,
    Knowledge,
    System,
    Audit,
}

impl AdminDomain {
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "总览",
            Self::Knowledge => "知识库",
            Self::System => "系统管理",
            Self::Audit => "审计日志",
        }
    }

    fn route(self) -> Route {
        match self {
            Self::Overview => Route::Home,
            Self::Knowledge => Route::KnowledgeNotes,
            Self::System => Route::SystemUsers,
            Self::Audit => Route::Audit,
        }
    }
}

impl DefaultAdminProvider {
    pub fn new(
        auth: AuthSession,
        theme: ThemePrefs,
        branding: BrandingPrefs,
        auth_api: SharedAuthApi,
        permissions: PermissionState,
    ) -> Self {
        Self {
            auth,
            theme,
            branding,
            auth_api,
            permissions,
        }
    }
}

impl AdminProvider<Route> for DefaultAdminProvider {
    fn topbar(&self, current: &Route) -> AdminTopbar<Route> {
        let auth = self.auth;
        let theme = self.theme;
        let branding = self.branding;
        let user = auth.username.read().clone();
        let brand_state = branding.state.read().clone();
        let is_dark = *theme.dark_mode.read();
        let dark_mode = theme.dark_mode;
        let logged_in = auth.logged_in;
        let ready = auth.ready;
        let username = auth.username;
        let auth_api = self.auth_api.clone();

        AdminTopbar {
            brand: Some(render_brand(&brand_state, user.as_str(), current)),
            eyebrow: None,
            title: String::new(),
            left: Vec::new(),
            right: vec![
                AdminAction {
                    class: "icon-button".to_string(),
                    title: if is_dark {
                        "切换到白天模式".to_string()
                    } else {
                        "切换到黑夜模式".to_string()
                    },
                    tone: None,
                    icon: if is_dark {
                        AdminActionIcon::Sun
                    } else {
                        AdminActionIcon::Moon
                    },
                    cmd: AdminCommand::run(move || {
                        let mut dark_mode = dark_mode;
                        dark_mode.set(!is_dark);
                    }),
                },
                AdminAction {
                    class: "icon-button".to_string(),
                    title: "搜索".to_string(),
                    tone: None,
                    icon: AdminActionIcon::Search,
                    cmd: AdminCommand::run(|| {
                        document::eval(
                            "if (window.mscFocusCommandSearch) window.mscFocusCommandSearch();",
                        );
                    }),
                },
                AdminAction {
                    class: "icon-button".to_string(),
                    title: "通知".to_string(),
                    tone: None,
                    icon: AdminActionIcon::Bell,
                    cmd: AdminCommand::default(),
                },
                AdminAction {
                    class: "icon-button".to_string(),
                    title: "退出登录".to_string(),
                    tone: None,
                    icon: AdminActionIcon::LogOut,
                    cmd: AdminCommand::run_to(Route::Login, move || {
                        let auth_api = auth_api.clone();
                        let mut logged_in = logged_in;
                        let mut ready = ready;
                        let mut username = username;
                        spawn(async move {
                            let _ = auth_api.logout().await;
                        });
                        logged_in.set(false);
                        username.set(String::new());
                        ready.set(true);
                    }),
                },
            ],
        }
    }

    fn menu(&self, current: &Route) -> Vec<AdminSection<Route>> {
        let section = section_for_domain(domain_for_route(current));
        let perms = self.permissions;
        vec![filter_section_by_permissions(section, perms)]
    }
}

fn domain_for_route(route: &Route) -> AdminDomain {
    match route {
        Route::Home | Route::Dashboard => AdminDomain::Overview,
        Route::Agents | Route::AgentEditor { .. } => AdminDomain::Knowledge,
        Route::KnowledgeNotes
        | Route::KnowledgePackages
        | Route::KnowledgeCliMarket
        | Route::KnowledgeCliMarketImports
        | Route::KnowledgeCliMarketDocs
        | Route::DownloadStation => AdminDomain::Knowledge,
        Route::SystemUsers
        | Route::SystemMenus
        | Route::SystemRoles
        | Route::SystemDepartments
        | Route::SystemDictionaries
        | Route::SystemAgentNodes
        | Route::SystemAgentPairingApproval { .. }
        | Route::SystemSettings => AdminDomain::System,
        Route::Audit => AdminDomain::Audit,
        Route::Login => AdminDomain::Overview,
    }
}

fn section_for_domain(domain: AdminDomain) -> AdminSection<Route> {
    match domain {
        AdminDomain::Overview => AdminSection {
            label: domain.label().to_string(),
            menus: vec![AdminMenu::leaf("闪念", Route::Home, |route| {
                matches!(route, Route::Home | Route::Dashboard)
            })],
        },
        AdminDomain::Knowledge => AdminSection {
            label: domain.label().to_string(),
            menus: vec![
                AdminMenu::leaf("笔记", Route::KnowledgeNotes, |route| {
                    matches!(route, Route::KnowledgeNotes)
                }),
                AdminMenu::leaf("Skills", Route::Agents, |route| {
                    matches!(route, Route::Agents | Route::AgentEditor { .. })
                }),
                AdminMenu::leaf("安装包", Route::KnowledgePackages, |route| {
                    matches!(route, Route::KnowledgePackages)
                }),
                AdminMenu::branch(
                    "CLI 市场",
                    Some(Route::KnowledgeCliMarket),
                    vec![
                        AdminMenu::leaf("注册表", Route::KnowledgeCliMarket, |route| {
                            matches!(route, Route::KnowledgeCliMarket)
                        }),
                        AdminMenu::leaf("导入任务", Route::KnowledgeCliMarketImports, |route| {
                            matches!(route, Route::KnowledgeCliMarketImports)
                        }),
                        AdminMenu::leaf("CLI 文档", Route::KnowledgeCliMarketDocs, |route| {
                            matches!(route, Route::KnowledgeCliMarketDocs)
                        }),
                    ],
                    |route| {
                        matches!(
                            route,
                            Route::KnowledgeCliMarket
                                | Route::KnowledgeCliMarketImports
                                | Route::KnowledgeCliMarketDocs
                        )
                    },
                ),
                AdminMenu::leaf("下载站", Route::DownloadStation, |route| {
                    matches!(route, Route::DownloadStation)
                }),
            ],
        },
        AdminDomain::System => AdminSection {
            label: domain.label().to_string(),
            menus: vec![
                AdminMenu::leaf("用户", Route::SystemUsers, |route| {
                    matches!(route, Route::SystemUsers)
                }),
                AdminMenu::leaf("菜单", Route::SystemMenus, |route| {
                    matches!(route, Route::SystemMenus)
                }),
                AdminMenu::leaf("角色", Route::SystemRoles, |route| {
                    matches!(route, Route::SystemRoles)
                }),
                AdminMenu::leaf("部门", Route::SystemDepartments, |route| {
                    matches!(route, Route::SystemDepartments)
                }),
                AdminMenu::leaf("字典管理", Route::SystemDictionaries, |route| {
                    matches!(route, Route::SystemDictionaries)
                }),
                AdminMenu::leaf("Agent 节点", Route::SystemAgentNodes, |route| {
                    matches!(
                        route,
                        Route::SystemAgentNodes | Route::SystemAgentPairingApproval { .. }
                    )
                }),
                AdminMenu::leaf("系统设置", Route::SystemSettings, |route| {
                    matches!(route, Route::SystemSettings)
                }),
            ],
        },
        AdminDomain::Audit => AdminSection {
            label: domain.label().to_string(),
            menus: vec![AdminMenu::leaf("审计日志", Route::Audit, |route| {
                matches!(route, Route::Audit)
            })],
        },
    }
}

/// 菜单项对应的权限标识。返回 None 表示始终可见（如仪表盘）。
fn permission_for_menu(menu_label: &str) -> Option<&'static str> {
    match menu_label {
        "闪念" => Some("overview"),
        "笔记" => Some("knowledge:note"),
        "Skills" => Some("knowledge:skill"),
        "安装包" => Some("knowledge:pkg"),
        "CLI 市场" => Some("knowledge:cli"),
        "导入任务" => Some("knowledge:cli"),
        "CLI 文档" => Some("knowledge:cli"),
        "下载站" => Some("knowledge:dl"),
        "用户" => Some("system:user"),
        "菜单" => Some("system:menu"),
        "角色" => Some("system:role"),
        "部门" => Some("system:dept"),
        "字典管理" => Some("system:dict"),
        "Agent 节点" => Some("system:agent"),
        "系统设置" => Some("system:setting"),
        "审计日志" => Some("audit"),
        _ => None,
    }
}

/// 递归过滤菜单树：只保留用户拥有权限的菜单项。
fn filter_menu_by_permissions(menu: AdminMenu<Route>, perms: PermissionState) -> Option<AdminMenu<Route>> {
    let visible = match permission_for_menu(&menu.label) {
        None => true, // 无权限码 = 始终可见
        Some(code) => perms.has(code),
    };
    if !visible {
        return None;
    }
    // 过滤子菜单
    let filtered_children: Vec<AdminMenu<Route>> = menu
        .children
        .into_iter()
        .filter_map(|child| filter_menu_by_permissions(child, perms))
        .collect();
    Some(AdminMenu {
        label: menu.label,
        to: menu.to,
        on_select: menu.on_select,
        is_active: menu.is_active,
        children: filtered_children,
    })
}

fn filter_section_by_permissions(section: AdminSection<Route>, perms: PermissionState) -> AdminSection<Route> {
    let filtered_menus = section
        .menus
        .into_iter()
        .filter_map(|menu| filter_menu_by_permissions(menu, perms))
        .collect();
    AdminSection {
        label: section.label,
        menus: filtered_menus,
    }
}

fn render_brand(
    brand_state: &BrandingState,
    username: &str,
    current: &Route,
) -> dioxus::prelude::Element {
    let site_name = brand_state.site_name.as_str();
    let username = if username.is_empty() {
        "未登录用户".to_string()
    } else {
        username.to_string()
    };
    let active_domain = domain_for_route(current);
    let logo_url = brand_state.active_logo_url();

    let brand_detail = match brand_state.logo_source {
        BrandingLogoSource::AppIcon => format!("{username} · App 图标"),
        BrandingLogoSource::CustomUpload => format!("{username} · 自定义 Logo"),
        BrandingLogoSource::TextOnly => username.clone(),
    };

    let brand_panel = rsx! {
        div { class: "topbar-brand",
            if let Some(url) = logo_url {
                div { class: "topbar-brand__mark",
                    img {
                        class: "topbar-brand__mark-image",
                        src: "{url}",
                        alt: "{site_name} Logo"
                    }
                }
            }
            div { class: "topbar-brand__meta",
                span { class: "topbar-brand__label", "{site_name}" }
                span { class: "topbar-brand__detail", "{brand_detail}" }
            }
        }
    };

    rsx! {
        div { class: "topbar-brand-shell",
            {brand_panel}
            DomainSwitcher { active: active_domain }
        }
    }
}

#[component]
fn DomainSwitcher(active: AdminDomain) -> Element {
    rsx! {
        div { class: "domain-switcher",
            for domain in [
                AdminDomain::Overview,
                AdminDomain::Knowledge,
                AdminDomain::System,
                AdminDomain::Audit,
            ] {
                DomainLink { domain, active: active == domain }
            }
        }
    }
}

#[component]
fn DomainLink(domain: AdminDomain, active: bool) -> Element {
    let class = if active {
        "domain-switcher__button domain-switcher__button--active"
    } else {
        "domain-switcher__button"
    };
    let label = domain.label();
    let to = domain.route();

    rsx! {
        Link { to,
            WorkbenchButton { class: class.to_string(), "{label}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdminDomain, domain_for_route, section_for_domain};
    use crate::app::Route;
    use std::collections::BTreeSet;

    #[test]
    fn knowledge_aux_routes_stay_inside_knowledge_domain() {
        assert_eq!(
            domain_for_route(&Route::DownloadStation),
            AdminDomain::Knowledge
        );
        assert_eq!(domain_for_route(&Route::Agents), AdminDomain::Knowledge);
        assert_eq!(AdminDomain::Knowledge.route(), Route::KnowledgeNotes);
    }

    #[test]
    fn knowledge_section_exposes_single_unique_navigation_tree() {
        let section = section_for_domain(AdminDomain::Knowledge);
        let labels = section
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect::<Vec<_>>();
        let unique_labels = labels.iter().copied().collect::<BTreeSet<_>>();
        let routes = section
            .menus
            .iter()
            .map(|menu| menu.to.clone())
            .collect::<Vec<_>>();

        assert_eq!(section.label, "知识库");
        assert_eq!(
            labels,
            vec!["笔记", "Skills", "安装包", "CLI 市场", "下载站"]
        );
        assert_eq!(unique_labels.len(), labels.len());
        assert_eq!(
            routes,
            vec![
                Some(Route::KnowledgeNotes),
                Some(Route::Agents),
                Some(Route::KnowledgePackages),
                Some(Route::KnowledgeCliMarket),
                Some(Route::DownloadStation),
            ]
        );
    }
}
