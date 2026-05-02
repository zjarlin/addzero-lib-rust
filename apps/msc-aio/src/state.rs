use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_components::SharedAdminProvider;

use crate::admin::DefaultAdminProvider;
use crate::app::Route;
use crate::services::{
    BrandingLogoSource, BrandingSettingsDto, BrandingSettingsUpdate, SharedAgentRuntimeApi,
    SharedAssetGraphApi, SharedAuthApi, SharedBrandingSettingsApi, SharedCliMarketApi,
    SharedKnowledgeGraphApi, SharedLogoStorageApi, SharedSkillsApi, SharedSoftwareCatalogApi,
    SharedSystemManagementApi,
    StoredLogoDto, build_preview_url, default_agent_runtime_api, default_asset_graph_api,
    default_auth_api, default_branding_settings_api, default_cli_market_api,
    default_knowledge_graph_api, default_logo_storage_api, default_skills_api,
    default_software_catalog_api, default_system_management_api,
};

pub const DEFAULT_SITE_NAME: &str = "MSC_AIO";
pub const DEFAULT_BRAND_COPY: &str = "顶部品牌区默认使用 App 图标，可切换为上传品牌资产。";
pub const DEFAULT_HEADER_BADGE: &str = "Knowledge Workspace";
pub const APP_ICON_ASSET_PATH: &str = "/assets/app-icon.png";

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

impl From<BrandingLogo> for StoredLogoDto {
    fn from(value: BrandingLogo) -> Self {
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
    pub logo_source: BrandingLogoSource,
    pub site_name: String,
    pub brand_copy: String,
    pub header_badge: String,
}

impl Default for BrandingState {
    fn default() -> Self {
        Self {
            logo: None,
            logo_source: BrandingLogoSource::AppIcon,
            site_name: DEFAULT_SITE_NAME.to_string(),
            brand_copy: DEFAULT_BRAND_COPY.to_string(),
            header_badge: DEFAULT_HEADER_BADGE.to_string(),
        }
    }
}

impl BrandingState {
    pub fn active_logo_url(&self) -> Option<String> {
        match self.logo_source {
            BrandingLogoSource::AppIcon => Some(APP_ICON_ASSET_PATH.to_string()),
            BrandingLogoSource::CustomUpload => self.logo.as_ref().map(BrandingLogo::preview_url),
            BrandingLogoSource::TextOnly => None,
        }
    }

    pub fn stored_logo_dto(&self) -> Option<StoredLogoDto> {
        self.logo.clone().map(StoredLogoDto::from)
    }

    pub fn to_settings_update(&self) -> BrandingSettingsUpdate {
        BrandingSettingsUpdate {
            site_name: self.site_name.clone(),
            logo_source: self.logo_source,
            logo: self.stored_logo_dto(),
            brand_copy: self.brand_copy.clone(),
            header_badge: self.header_badge.clone(),
        }
    }
}

impl From<BrandingSettingsDto> for BrandingState {
    fn from(value: BrandingSettingsDto) -> Self {
        Self {
            logo: value.logo.map(BrandingLogo::from),
            logo_source: value.logo_source,
            site_name: value.site_name,
            brand_copy: value.brand_copy,
            header_badge: value.header_badge,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BrandingPrefs {
    pub state: Signal<BrandingState>,
}

/// 用户有效权限码集合。
/// `None` = 尚未加载；`Some(None)` = admin 超级管理员（不限制）；`Some(Some(codes))` = 有权限码列表。
#[derive(Clone, Copy)]
pub struct PermissionState {
    pub codes: Signal<Option<Option<Vec<String>>>>,
}

impl PermissionState {
    pub fn new() -> Self {
        Self {
            codes: Signal::new(None),
        }
    }

    /// 检查是否拥有指定权限码。admin 用户始终返回 true。
    pub fn has(&self, code: &str) -> bool {
        match self.codes.read().as_ref() {
            None => false,                             // 尚未加载，默认拒绝
            Some(None) => true,                        // admin，全权
            Some(Some(list)) => list.contains(&code.to_string()),
        }
    }

    /// 权限是否已加载完成
    pub fn is_loaded(&self) -> bool {
        self.codes.read().is_some()
    }
}

#[derive(Clone)]
pub struct AppServices {
    pub auth_api: SharedAuthApi,
    pub skills: SharedSkillsApi,
    pub agent_runtime: SharedAgentRuntimeApi,
    pub asset_graph: SharedAssetGraphApi,
    pub knowledge_graph: SharedKnowledgeGraphApi,
    pub cli_market: SharedCliMarketApi,
    pub software_catalog: SharedSoftwareCatalogApi,
    pub system: SharedSystemManagementApi,
    pub logo_storage: SharedLogoStorageApi,
    pub branding_settings: SharedBrandingSettingsApi,
    pub admin: SharedAdminProvider<Route>,
    pub branding: BrandingPrefs,
    pub permissions: PermissionState,
}

impl AppServices {
    pub fn new(auth: AuthSession, theme: ThemePrefs, branding: BrandingPrefs, permissions: PermissionState) -> Self {
        let auth_api = default_auth_api();
        let skills = default_skills_api();
        let agent_runtime = default_agent_runtime_api();
        Self {
            auth_api: auth_api.clone(),
            skills,
            agent_runtime,
            asset_graph: default_asset_graph_api(),
            knowledge_graph: default_knowledge_graph_api(),
            cli_market: default_cli_market_api(),
            software_catalog: default_software_catalog_api(),
            system: default_system_management_api(),
            logo_storage: default_logo_storage_api(),
            branding_settings: default_branding_settings_api(),
            admin: Rc::new(DefaultAdminProvider::new(auth, theme, branding, auth_api, permissions)),
            branding,
            permissions,
        }
    }
}
