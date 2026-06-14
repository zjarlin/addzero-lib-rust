//! Admin 插件注册表到 AZ AIO 的桥接层。
//!
//! 本 crate 将 `az_admin_plugin_registry` 中通过 `register_admin_domain!` /
//! `register_admin_page!` 等宏注册的 admin 域名和页面节点，映射为 AZ AIO 的
//! `PageContribution` 和 `NavItemContribution`，使 admin 路由可在 AZ AIO 界面的
//! nav 和路由匹配中正常工作。

use az_admin_plugin_registry::api::{registered_domains, section_for_path};
use az_aio_plugin_api::api::{
    ContributionSet, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginActivation, PluginDescriptor, PluginKind,
    NavItemContribution, PageContribution, UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

const PLUGIN_ID: &str = "admin-bridge";
const PLUGIN_NAME: &str = "Admin 桥接";
const RENDERER_ID: &str = "admin-bridge.content";
const PLUGIN_DESCRIPTION: &str = "将 admin registry 中已注册的域名和路由暴露为 AZ AIO 的插件贡献。";

#[derive(Default)]
pub struct AdminBridgePlugin;

impl NativeAzAioPlugin for AdminBridgePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PLUGIN_ID.to_string(),
            name: PLUGIN_NAME.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: PLUGIN_DESCRIPTION.to_string(),
            activation: PluginActivation::Eager,
            priority: 500,
            dependencies: Vec::new(),
            capabilities: vec!["nav-items".to_string(), "pages".to_string()],
            permissions: Vec::new(),
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        let domains = registered_domains();
        if domains.is_empty() {
            return Ok(ContributionSet::default());
        }

        let mut nav_items: Vec<NavItemContribution> = Vec::new();
        let mut pages: Vec<PageContribution> = Vec::new();

        for (idx, domain) in domains.iter().enumerate() {
            let route = format!("/admin/{}", domain.id);
            nav_items.push(NavItemContribution {
                id: format!("admin-bridge.nav.{}", domain.id),
                label: domain.label.to_string(),
                icon: "◇".to_string(),
                route: route.clone(),
                order: 50 + idx as i32,
            });

            pages.push(PageContribution {
                route: route.clone(),
                title: domain.label.to_string(),
                subtitle: format!("{} admin 域", domain.label),
                renderer_id: RENDERER_ID.to_string(),
                placeholder_mark: "◇".to_string(),
                order: 50 + idx as i32,
            });
        }

        Ok(ContributionSet {
            nav_items,
            pages,
            ui_contributions: Vec::new(),
            backend_apis: Vec::new(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime {
            renderers: vec![],
            router: axum::Router::new(),
            startup: None,
        })
    }
}

register_native_plugin!(AdminBridgePlugin);

pub fn ensure_linked() {}
