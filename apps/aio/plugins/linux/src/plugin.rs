use std::sync::Arc;

use az_aio_platform::plugin::contract::{
    AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
    DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
    PluginDescriptor,
};
use rudi::Singleton;

use crate::{
    backend::routes::{LinuxApiState, linux_router},
    descriptor::{ROUTE, contributions, descriptor},
};

#[derive(Default)]
pub struct LinuxPlugin;

impl NativePluginProvider for LinuxPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: "intelligent-gateway".to_string(),
                label: "智能网关".to_string(),
                default_href: ROUTE.to_string(),
                order: 300,
                menus: vec![AdminMenuNode {
                    id: "linux.nav".to_string(),
                    kind: AdminMenuNodeKind::Page,
                    label: "Linux 节点".to_string(),
                    href: ROUTE.to_string(),
                    icon: "🐧".to_string(),
                    order: 30,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["render-bootstrap-script".to_string()],
                    children: Vec::new(),
                }],
            }],
        }
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let install_base_url = if context.api_base_url.trim().is_empty() {
            "http://<aio-host>:<port>".to_string()
        } else {
            context.api_base_url.clone()
        };
        let state = LinuxApiState::new(install_base_url);
        Ok(NativePluginRuntime {
            renderers: Vec::new(),
            router: linux_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "linux")]
pub fn linux_plugin() -> DynAdminPluginProvider {
    Arc::new(LinuxPlugin)
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::contract::PluginKind;

    use super::*;

    #[test]
    fn descriptor_exposes_client_first_linux_contract() -> anyhow::Result<()> {
        let plugin = LinuxPlugin;
        let descriptor = plugin.descriptor();
        let contributions = plugin.contributions()?;

        assert_eq!(descriptor.id, "linux");
        assert_eq!(descriptor.kind, PluginKind::Native);
        assert!(contributions.pages.iter().any(|page| page.route == "/linux"));
        assert!(contributions.backend_apis.iter().any(|api| {
            api.method == "GET" && api.path == "/api/linux/bootstrap-script"
        }));
        assert_eq!(
            plugin
                .admin_menu(&contributions)
                .sections
                .first()
                .map(|section| section.label.as_str()),
            Some("智能网关")
        );
        Ok(())
    }
}
