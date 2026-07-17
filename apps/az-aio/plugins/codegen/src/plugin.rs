//! 代码生成插件注册与 Admin 契约。

use std::{env, sync::Arc};

use az_aio_platform::plugin::api::{
    AdminCliContribution, AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree,
    BackendApiContribution, ContributionSet, DynAdminPluginProvider, NativePluginContext,
    NativePluginProvider, NativePluginRuntime, NativeUiRenderer, NavItemContribution,
    PageContribution, PluginActivation, PluginDescriptor, PluginKind, UiContribution,
    UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    contract::{OP_RUST_FILE_GENERATE, RUST_FILES_PATH, STATUS_PATH},
    generator::ClientRustCodegen,
    routes::{CodegenApiState, codegen_router},
    ui::CodegenPage,
};

const PLUGIN_ID: &str = "codegen";
const ROUTE: &str = "/codegen";
const RENDERER_ID: &str = "codegen.page";

/// 当前客户机 Rust 代码生成插件。
#[derive(Default)]
pub struct CodegenPlugin;

impl NativePluginProvider for CodegenPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PLUGIN_ID.to_string(),
            name: "代码生成".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "在当前客户机目录生成类型安全的 Rust enum 和 struct 源文件。".to_string(),
            activation: PluginActivation::Eager,
            priority: 610,
            dependencies: Vec::new(),
            capabilities: vec![
                "rust-codegen".to_string(),
                "client-filesystem-write".to_string(),
                "axum-api".to_string(),
            ],
            permissions: vec!["写入网页明确选择的客户机目录".to_string()],
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "codegen.nav".to_string(),
                label: "代码生成".to_string(),
                icon: "⌘".to_string(),
                route: ROUTE.to_string(),
                order: 60,
            }],
            pages: vec![PageContribution {
                route: ROUTE.to_string(),
                title: "Rust 代码生成".to_string(),
                subtitle: "在当前客户机生成 enum 和 struct".to_string(),
                renderer_id: RENDERER_ID.to_string(),
                placeholder_mark: "⌘".to_string(),
                order: 60,
            }],
            ui_contributions: vec![UiContribution {
                id: "codegen.ui.content".to_string(),
                slot: UiContributionSlot::Content,
                label: "代码生成内容区".to_string(),
                renderer_id: RENDERER_ID.to_string(),
                route: Some(ROUTE.to_string()),
                order: 60,
            }],
            backend_apis: vec![
                backend_api("codegen.status", "GET", STATUS_PATH, "代码生成节点状态", 10),
                backend_api(
                    OP_RUST_FILE_GENERATE,
                    "POST",
                    RUST_FILES_PATH,
                    "在客户机生成 Rust 文件",
                    20,
                ),
            ],
            ..ContributionSet::default()
        })
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: "lowcode".to_string(),
                label: "低代码".to_string(),
                default_href: String::new(),
                order: 600,
                menus: vec![AdminMenuNode {
                    id: "codegen.root".to_string(),
                    kind: AdminMenuNodeKind::Page,
                    label: "Rust 代码生成".to_string(),
                    href: ROUTE.to_string(),
                    icon: "⌘".to_string(),
                    order: 20,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["codegen:write".to_string()],
                    children: Vec::new(),
                }],
            }],
        }
    }

    fn admin_cli(&self) -> Vec<AdminCliContribution> {
        vec![AdminCliContribution {
            id: "codegen.cli.rust-file-generate".to_string(),
            label: "在客户机生成 Rust 文件".to_string(),
            command: "az codegen rust-file generate --request <request.json>".to_string(),
            resource_id: None,
            operation_id: Some(OP_RUST_FILE_GENERATE.to_string()),
        }]
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let current_directory = env::current_dir()?;
        let base_directory = env::var_os("AIO_CODEGEN_ROOT")
            .map(std::path::PathBuf::from)
            .or_else(|| env::var_os("HOME").map(std::path::PathBuf::from))
            .unwrap_or(current_directory);
        let state = CodegenApiState::new(ClientRustCodegen::new(base_directory));
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: CodegenPage,
            }],
            router: codegen_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "codegen")]
pub fn codegen_plugin() -> DynAdminPluginProvider {
    Arc::new(CodegenPlugin)
}

fn backend_api(
    id: &str,
    method: &str,
    path: &str,
    label: &str,
    order: i32,
) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: format!("{label}: {path}"),
        order,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_exposes_web_and_client_codegen_contracts() -> anyhow::Result<()> {
        let plugin = CodegenPlugin;
        let contributions = plugin.contributions()?;

        // 关键断言：页面与 REST 操作必须由同一插件贡献，客户机执行能力才能被发现。
        assert!(contributions.pages.iter().any(|page| page.route == ROUTE));
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.id == OP_RUST_FILE_GENERATE && api.path == RUST_FILES_PATH)
        );
        assert!(
            plugin
                .descriptor()
                .capabilities
                .contains(&"client-filesystem-write".to_string())
        );
        Ok(())
    }
}
