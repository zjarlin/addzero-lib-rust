//! 后台主轴场景预留 provider。
//!
//! 这里只声明空的主轴上下文入口，不生成业务页面；具体菜单节点仍由各插件自治贡献。

use std::sync::Arc;

use axum::Router;
use rudi::Singleton;

use crate::plugin::contract::{
    AdminMenuSection, AdminMenuTree, AdminPluginContribution, AdminPluginProvider,
    ContributionSet, DynAdminPluginProvider, NativePluginContext, NativePluginRuntime,
    PluginActivation, PluginDescriptor, PluginKind,
};

pub const KNOWLEDGE_BASE_DOMAIN_ID: &str = "knowledge-base";
pub const KNOWLEDGE_BASE_DOMAIN_LABEL: &str = "知识库";
pub const INTELLIGENT_GATEWAY_DOMAIN_ID: &str = "intelligent-gateway";
pub const INTELLIGENT_GATEWAY_DOMAIN_LABEL: &str = "智能网关";

#[derive(Clone, Debug, Default)]
pub struct AdminSceneProvider;

impl AdminPluginProvider for AdminSceneProvider {
    fn admin_descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "admin-scenes".to_string(),
            name: "后台场景".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "预留后台主轴上下文：知识库与智能网关。".to_string(),
            activation: PluginActivation::Eager,
            priority: 980,
            dependencies: Vec::new(),
            capabilities: vec!["admin-context-axis".to_string()],
            permissions: Vec::new(),
            kind: PluginKind::Native,
        }
    }

    fn admin_contribution(&self) -> anyhow::Result<AdminPluginContribution> {
        Ok(AdminPluginContribution {
            menu: AdminMenuTree {
                sections: vec![
                    AdminMenuSection {
                        domain_id: KNOWLEDGE_BASE_DOMAIN_ID.to_string(),
                        label: KNOWLEDGE_BASE_DOMAIN_LABEL.to_string(),
                        default_href: String::new(),
                        order: 200,
                        menus: Vec::new(),
                    },
                    AdminMenuSection {
                        domain_id: INTELLIGENT_GATEWAY_DOMAIN_ID.to_string(),
                        label: INTELLIGENT_GATEWAY_DOMAIN_LABEL.to_string(),
                        default_href: String::new(),
                        order: 300,
                        menus: Vec::new(),
                    },
                ],
            },
            resources: Vec::new(),
            cli: Vec::new(),
            native: ContributionSet::default(),
        })
    }

    fn admin_runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime {
            renderers: Vec::new(),
            router: Router::new(),
            startup: None,
        })
    }
}

#[Singleton(name = "admin-scenes")]
pub fn admin_scene_provider() -> DynAdminPluginProvider {
    Arc::new(AdminSceneProvider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_provider_reserves_knowledge_base_and_gateway_axes() {
        let provider = AdminSceneProvider;
        let contribution = provider.admin_contribution().unwrap();
        let labels = contribution
            .menu
            .sections
            .iter()
            .map(|section| section.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["知识库", "智能网关"]);
        assert!(contribution.menu.sections.iter().all(|section| section.menus.is_empty()));
    }
}
