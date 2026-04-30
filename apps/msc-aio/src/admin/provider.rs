use dioxus::prelude::*;
use dioxus_components::{
    AdminAction, AdminActionIcon, AdminCommand, AdminMenu, AdminProvider, AdminSection,
    AdminTopbar, WorkbenchButton,
};

use crate::app::Route;
use crate::services::SharedAuthApi;
use crate::state::{AuthSession, BrandingPrefs, ThemePrefs};

pub struct DefaultAdminProvider {
    auth: AuthSession,
    theme: ThemePrefs,
    branding: BrandingPrefs,
    auth_api: SharedAuthApi,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdminDomain {
    Overview,
    Agents,
    Knowledge,
    System,
    Audit,
}

impl AdminDomain {
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "总览",
            Self::Agents => "Agent资产",
            Self::Knowledge => "知识库",
            Self::System => "系统管理",
            Self::Audit => "审计日志",
        }
    }

    fn route(self) -> Route {
        match self {
            Self::Overview => Route::Home,
            Self::Agents => Route::Agents,
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
    ) -> Self {
        Self {
            auth,
            theme,
            branding,
            auth_api,
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
        let logo = brand_state.logo;
        let site_name = brand_state.site_name;
        let is_dark = *theme.dark_mode.read();
        let dark_mode = theme.dark_mode;
        let logged_in = auth.logged_in;
        let ready = auth.ready;
        let username = auth.username;
        let auth_api = self.auth_api.clone();

        AdminTopbar {
            brand: Some(render_brand(
                logo.as_ref(),
                site_name.as_str(),
                user.as_str(),
                current,
            )),
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
                    cmd: AdminCommand::default(),
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
        vec![section_for_domain(domain_for_route(current))]
    }
}

fn domain_for_route(route: &Route) -> AdminDomain {
    match route {
        Route::Home | Route::Dashboard => AdminDomain::Overview,
        Route::Agents | Route::AgentEditor { .. } => AdminDomain::Agents,
        Route::KnowledgeNotes | Route::KnowledgeSoftware | Route::KnowledgePackages => {
            AdminDomain::Knowledge
        }
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
            menus: vec![AdminMenu::leaf("知识图谱概览", Route::Home, |route| {
                matches!(route, Route::Home | Route::Dashboard)
            })],
        },
        AdminDomain::Agents => AdminSection {
            label: domain.label().to_string(),
            menus: vec![AdminMenu::leaf("技能资产", Route::Agents, |route| {
                matches!(route, Route::Agents | Route::AgentEditor { .. })
            })],
        },
        AdminDomain::Knowledge => AdminSection {
            label: domain.label().to_string(),
            menus: vec![
                AdminMenu::leaf("笔记", Route::KnowledgeNotes, |route| {
                    matches!(route, Route::KnowledgeNotes)
                }),
                AdminMenu::leaf("软件", Route::KnowledgeSoftware, |route| {
                    matches!(route, Route::KnowledgeSoftware)
                }),
                AdminMenu::leaf("安装包", Route::KnowledgePackages, |route| {
                    matches!(route, Route::KnowledgePackages)
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

fn render_brand(
    logo: Option<&crate::state::BrandingLogo>,
    site_name: &str,
    username: &str,
    current: &Route,
) -> dioxus::prelude::Element {
    let username = if username.is_empty() {
        "未登录用户".to_string()
    } else {
        username.to_string()
    };
    let active_domain = domain_for_route(current);

    let brand_detail = if let Some(logo) = logo {
        format!("{username} · {}", logo.backend_label)
    } else {
        username.clone()
    };

    let brand_panel = rsx! {
        div { class: "topbar-brand topbar-brand--text-only",
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
                AdminDomain::Agents,
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
