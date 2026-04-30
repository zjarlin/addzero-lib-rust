use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_components::SharedAdminProvider;

use crate::admin::DefaultAdminProvider;
use crate::app::Route;
use crate::services::{
    SharedAgentRuntimeApi, SharedAuthApi, SharedLogoStorageApi, SharedSkillsApi, StoredLogoDto,
    build_preview_url, default_agent_runtime_api, default_auth_api, default_logo_storage_api,
    default_skills_api,
};

pub const DEFAULT_SITE_NAME: &str = "MSC_AIO";

#[derive(Clone, Copy)]
pub struct ThemePrefs {
    pub dark_mode: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct AuthSession {
    pub logged_in: Signal<bool>,
    pub username: Signal<String>,
    pub ready: Signal<bool>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BrandingLogo {
    pub object_key: String,
    pub relative_path: String,
    pub file_name: String,
    pub content_type: String,
    pub backend_label: String,
}

impl BrandingLogo {
    pub fn preview_url(&self) -> String {
        build_preview_url(&self.relative_path)
    }
}

impl From<StoredLogoDto> for BrandingLogo {
    fn from(value: StoredLogoDto) -> Self {
        Self {
            object_key: value.object_key,
            relative_path: value.relative_path,
            file_name: value.file_name,
            content_type: value.content_type,
            backend_label: value.backend_label,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct BrandingState {
    pub logo: Option<BrandingLogo>,
    pub site_name: String,
}

impl Default for BrandingState {
    fn default() -> Self {
        Self {
            logo: None,
            site_name: DEFAULT_SITE_NAME.to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BrandingPrefs {
    pub state: Signal<BrandingState>,
}

#[derive(Clone)]
pub struct AppServices {
    pub auth_api: SharedAuthApi,
    pub skills: SharedSkillsApi,
    pub agent_runtime: SharedAgentRuntimeApi,
    pub logo_storage: SharedLogoStorageApi,
    pub admin: SharedAdminProvider<Route>,
    pub branding: BrandingPrefs,
}

impl AppServices {
    pub fn new(auth: AuthSession, theme: ThemePrefs, branding: BrandingPrefs) -> Self {
        let auth_api = default_auth_api();
        let skills = default_skills_api();
        let agent_runtime = default_agent_runtime_api();
        Self {
            auth_api: auth_api.clone(),
            skills,
            agent_runtime,
            logo_storage: default_logo_storage_api(),
            admin: Rc::new(DefaultAdminProvider::new(auth, theme, branding, auth_api)),
            branding,
        }
    }
}
