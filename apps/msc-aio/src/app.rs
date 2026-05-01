use dioxus::prelude::*;
use dioxus_components::AdminShell;

use crate::scenes::{
    agent_nodes::{SystemAgentNodes, SystemAgentPairingApproval},
    agents::{AgentEditor, Agents},
    auth::LoginPage,
    cli_market::{KnowledgeCliMarket, KnowledgeCliMarketDocs, KnowledgeCliMarketImports},
    dashboard::{Audit, Dashboard},
    download_station::DownloadStationScene,
    knowledge_base::{KnowledgeNotes, KnowledgePackages},
    system_management::{
        SystemDepartments, SystemDictionaries, SystemMenus, SystemRoles, SystemUsers,
    },
    system_settings::SystemSettings,
};
use crate::state::{AppServices, AuthSession, BrandingPrefs, ThemePrefs};

const STYLE: &str = include_str!("../assets/admin.css");
const COMMAND_SEARCH_SCRIPT: &str = r#"
(() => {
  if (window.__mscCommandSearchReady) {
    return;
  }
  window.__mscCommandSearchReady = true;

  window.mscFocusCommandSearch = () => {
    const input = document.querySelector('[data-command-search="true"]');
    if (!input) {
      return false;
    }
    input.focus();
    if (typeof input.select === 'function') {
      input.select();
    }
    return true;
  };

  document.addEventListener('keydown', (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      if (window.mscFocusCommandSearch && window.mscFocusCommandSearch()) {
        event.preventDefault();
      }
    }
  });
})();
"#;

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[route("/login")]
    Login,
    #[layout(AppLayout)]
    #[route("/")]
    Home,
    #[route("/dashboard")]
    Dashboard,
    #[route("/agents")]
    Agents,
    #[route("/agents/:name")]
    AgentEditor { name: String },
    #[route("/knowledge/notes")]
    KnowledgeNotes,
    #[route("/knowledge/packages")]
    KnowledgePackages,
    #[route("/knowledge/cli-market")]
    KnowledgeCliMarket,
    #[route("/knowledge/cli-market/imports")]
    KnowledgeCliMarketImports,
    #[route("/knowledge/cli-market/docs")]
    KnowledgeCliMarketDocs,
    #[route("/download-station")]
    DownloadStation,
    #[route("/system/users")]
    SystemUsers,
    #[route("/system/menus")]
    SystemMenus,
    #[route("/system/roles")]
    SystemRoles,
    #[route("/system/departments")]
    SystemDepartments,
    #[route("/system/dictionaries")]
    SystemDictionaries,
    #[route("/system/agent-nodes")]
    SystemAgentNodes,
    #[route("/system/agent-nodes/pairings/:id")]
    SystemAgentPairingApproval { id: String },
    #[route("/system/settings")]
    SystemSettings,
    #[route("/audit")]
    Audit,
}

#[component]
pub fn App() -> Element {
    let logged_in = use_signal(|| false);
    let username = use_signal(String::new);
    let ready = use_signal(|| false);
    let dark_mode = use_signal(|| false);
    let branding_state = use_signal(Default::default);
    let mut auth = AuthSession {
        logged_in,
        username,
        ready,
    };
    let theme = ThemePrefs { dark_mode };
    let branding = BrandingPrefs {
        state: branding_state,
    };
    let app_services = AppServices::new(auth, theme, branding);

    use_context_provider(|| auth);
    use_context_provider(|| theme);
    use_context_provider(|| branding);
    use_context_provider(|| app_services.clone());

    let auth_api = app_services.auth_api.clone();
    let branding_api = app_services.branding_settings.clone();
    let _auth_bootstrap = use_resource(move || {
        let auth_api = auth_api.clone();
        async move {
            match auth_api.current_session().await {
                Ok(session) => {
                    auth.logged_in.set(session.authenticated);
                    auth.username.set(session.username.unwrap_or_default());
                }
                Err(_) => {
                    auth.logged_in.set(false);
                    auth.username.set(String::new());
                }
            }
            auth.ready.set(true);
        }
    });
    let _branding_bootstrap = use_resource(move || {
        let branding_api = branding_api.clone();
        let is_ready = *auth.ready.read();
        let is_logged_in = *auth.logged_in.read();
        let mut branding_state = branding.state;
        async move {
            if !is_ready || !is_logged_in {
                return;
            }
            if let Ok(settings) = branding_api.load_settings().await {
                branding_state.set(settings.into());
            }
        }
    });

    let shell_class = if *theme.dark_mode.read() {
        "theme-root theme-dark"
    } else {
        "theme-root theme-light"
    };

    rsx! {
        document::Style { {STYLE} }
        document::Script { {COMMAND_SEARCH_SCRIPT} }
        div { class: shell_class,
            Router::<Route> {}
        }
    }
}

#[component]
fn Login() -> Element {
    rsx! { LoginPage {} }
}

#[component]
fn Home() -> Element {
    rsx! { Dashboard {} }
}

#[component]
fn DownloadStation() -> Element {
    rsx! { DownloadStationScene {} }
}

#[component]
pub fn AppLayout() -> Element {
    let auth = use_context::<AuthSession>();
    let nav = use_navigator();
    let redirect_nav = nav.clone();

    use_effect(move || {
        if *auth.ready.read() && !*auth.logged_in.read() {
            redirect_nav.replace(Route::Login);
        }
    });

    if !*auth.ready.read() {
        return rsx! { div { class: "empty-state", "正在恢复登录态…" } };
    }

    if !*auth.logged_in.read() {
        return rsx! {};
    }

    rsx! {
        AdminShell::<Route> {
            provider: use_context::<AppServices>().admin.clone(),
            content: rsx!(Outlet::<Route> {})
        }
    }
}
