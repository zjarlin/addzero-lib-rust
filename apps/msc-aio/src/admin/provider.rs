use dioxus::prelude::*;
use dioxus_components::{
    AdminAction, AdminActionIcon, AdminCommand, AdminShellProvider, AdminShellState, AdminTopbar,
    WorkbenchButton,
};

use crate::admin::navigation::{domain_for_route, registered_domains, section_for_route};
use crate::app::Route;
use crate::services::{BrandingLogoSource, SharedAuthApi};
use crate::state::{AuthSession, BrandingPrefs, BrandingState, PermissionState, ThemePrefs};

pub struct AdminProvider {
    auth: AuthSession,
    theme: ThemePrefs,
    branding: BrandingPrefs,
    auth_api: SharedAuthApi,
    permissions: PermissionState,
}

impl AdminProvider {
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

impl AdminShellProvider<Route> for AdminProvider {
    fn shell(&self, current: &Route) -> AdminShellState<Route> {
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

        AdminShellState {
            topbar: AdminTopbar {
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
            },
            menu: section_for_route(current, self.permissions)
                .map(|section| vec![section])
                .unwrap_or_default(),
            right_panel: None,
        }
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
    let active_domain_id = domain_for_route(current).map(|domain| domain.id);
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
            DomainSwitcher { active_domain_id }
        }
    }
}

#[component]
fn DomainSwitcher(active_domain_id: Option<&'static str>) -> Element {
    let domains = registered_domains();

    rsx! {
        div { class: "domain-switcher",
            for domain in domains {
                DomainLink {
                    domain,
                    active: active_domain_id == Some(domain.id),
                }
            }
        }
    }
}

#[component]
fn DomainLink(
    domain: addzero_admin_plugin_registry::AdminDomainRegistration,
    active: bool,
) -> Element {
    let class = if active {
        "domain-switcher__button domain-switcher__button--active"
    } else {
        "domain-switcher__button"
    };
    let Some(to) = domain.default_href.parse::<Route>().ok() else {
        return rsx! {};
    };

    rsx! {
        Link { to,
            WorkbenchButton { class: class.to_string(), "{domain.label}" }
        }
    }
}
