use std::{env, path::PathBuf};

use az_aio_plugin_api::api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, PluginActivation, PluginDescriptor, PluginKind, UiContribution,
    UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

use crate::shell_scan::{managed_generated_file, scan_shell_sources};

const PLUGIN_ID: &str = "git/clis";
const DEFAULT_SOURCE_ROOT: &str = ".config/shell";
const DEFAULT_OUTPUT_FILE: &str = ".add_fn";

#[derive(Clone, Debug)]
pub struct GitClisPlugin {
    source_root: PathBuf,
    output_path: PathBuf,
    extra_cli_roots: Vec<PathBuf>,
}

impl GitClisPlugin {
    pub fn new(
        source_root: impl Into<PathBuf>,
        output_path: impl Into<PathBuf>,
        extra_cli_roots: Vec<PathBuf>,
    ) -> Self {
        Self {
            source_root: source_root.into(),
            output_path: output_path.into(),
            extra_cli_roots,
        }
    }
}

impl Default for GitClisPlugin {
    fn default() -> Self {
        let home = home_dir();
        Self::new(
            home.join(DEFAULT_SOURCE_ROOT),
            home.join(DEFAULT_OUTPUT_FILE),
            vec![home.join(".local/bin"), home.join("bin"), home.join(".bin")],
        )
    }
}

impl NativeAzAioPlugin for GitClisPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        let mut permissions = vec![
            format!("读取 {}", self.source_root.display()),
            format!("写入 {}", self.output_path.display()),
        ];
        permissions.extend(
            self.extra_cli_roots
                .iter()
                .map(|root| format!("读取 {}", root.display())),
        );

        PluginDescriptor {
            id: PLUGIN_ID.to_string(),
            name: "Git 命令行".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "扫描终端片段和用户命令脚本，供桌面端命令管理器使用。".to_string(),
            activation: PluginActivation::Eager,
            priority: 700,
            dependencies: Vec::new(),
            capabilities: vec![
                "shell-scan".to_string(),
                "cli-page".to_string(),
                "env-page".to_string(),
                "add-fn-catalog".to_string(),
            ],
            permissions,
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        let scan = scan_shell_sources(&self.source_root, &self.extra_cli_roots);
        let generated_file = managed_generated_file(&scan, &self.source_root, &self.output_path);

        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            ui_contributions: clis_ui_contributions(),
            backend_apis: clis_backend_apis(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: scan.shell_entries().cloned().collect(),
            generated_files: vec![generated_file],
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(GitClisPlugin);

pub fn ensure_linked() {}

fn clis_ui_contributions() -> Vec<UiContribution> {
    vec![ui_contribution(
        "git.clis.ui.content",
        UiContributionSlot::Content,
        "命令行内容区",
        "git.clis.manager",
        Some("/plugins"),
        30,
    )]
}

fn clis_backend_apis() -> Vec<BackendApiContribution> {
    vec![backend_api(
        "git.clis.api.scan",
        "GET",
        "/api/git/clis",
        "扫描命令行片段",
        "返回 shell 片段、函数、别名和脚本贡献。",
        10,
    )]
}

fn ui_contribution(
    id: &str,
    slot: UiContributionSlot,
    label: &str,
    renderer_id: &str,
    route: Option<&str>,
    order: i32,
) -> UiContribution {
    UiContribution {
        id: id.to_string(),
        slot,
        label: label.to_string(),
        renderer_id: renderer_id.to_string(),
        route: route.map(str::to_string),
        order,
    }
}

fn backend_api(
    id: &str,
    method: &str,
    path: &str,
    label: &str,
    description: &str,
    order: i32,
) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        order,
    }
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
