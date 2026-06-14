use az_aio_plugin_api::api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, NavItemContribution, PageContribution, PluginActivation, PluginDescriptor,
    PluginKind, SettingsDefaultContribution, SettingsSectionContribution, UiContribution,
    UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

const DEFAULT_SYNC_ROOT_SETTING: &str = "az-sync";

#[derive(Default)]
pub struct SyncPlugin;

impl NativeAzAioPlugin for SyncPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "sync".to_string(),
            name: "同步".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供基于行级 CRDT 的文件内容同步、WebSocket 同步契约和 Finder 状态桥接。"
                .to_string(),
            activation: PluginActivation::Eager,
            priority: 950,
            dependencies: Vec::new(),
            capabilities: vec![
                "crdt-line-sync".to_string(),
                "websocket-sync".to_string(),
                "backend-api".to_string(),
                "sync-state".to_string(),
                "finder-status".to_string(),
            ],
            permissions: vec![
                "read-sync-root".to_string(),
                "write-sync-root".to_string(),
                "network-sync".to_string(),
                "finder-status-state".to_string(),
            ],
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "sync.nav".to_string(),
                label: "同步".to_string(),
                icon: "⇄".to_string(),
                route: "/sync".to_string(),
                order: 30,
            }],
            pages: vec![PageContribution {
                route: "/sync".to_string(),
                title: "同步".to_string(),
                subtitle: "管理默认同步目录、设备状态和 CRDT 文件同步。".to_string(),
                renderer_id: "placeholder".to_string(),
                placeholder_mark: "⇄".to_string(),
                order: 30,
            }],
            ui_contributions: vec![
                ui_contribution(
                    "sync.ui.content",
                    UiContributionSlot::Content,
                    "同步内容区",
                    "sync.panel",
                    Some("/sync"),
                    10,
                ),
                ui_contribution(
                    "sync.ui.settings",
                    UiContributionSlot::SettingsContent,
                    "同步设置",
                    "sync.settings",
                    Some("/settings"),
                    40,
                ),
            ],
            backend_apis: sync_backend_apis(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: vec![SettingsSectionContribution {
                id: "sync.defaults".to_string(),
                label: "同步默认值".to_string(),
                order: 30,
                defaults: vec![
                    SettingsDefaultContribution {
                        key: "sync.default_root".to_string(),
                        label: "默认同步目录".to_string(),
                        value: DEFAULT_SYNC_ROOT_SETTING.to_string(),
                        description: "未显式配置时同步用户家目录下的 az-sync。".to_string(),
                        order: 10,
                    },
                    SettingsDefaultContribution {
                        key: "sync.transport".to_string(),
                        label: "同步传输".to_string(),
                        value: "websocket".to_string(),
                        description: "客户端和服务端默认通过 WebSocket 交换 CRDT 增量。"
                            .to_string(),
                        order: 20,
                    },
                    SettingsDefaultContribution {
                        key: "sync.conflict_model".to_string(),
                        label: "冲突模型".to_string(),
                        value: "line-crdt".to_string(),
                        description: "文本内容使用行级 CRDT 合并，不用最后写入覆盖。".to_string(),
                        order: 30,
                    },
                ],
            }],
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(SyncPlugin);

pub fn ensure_linked() {}

fn sync_backend_apis() -> Vec<BackendApiContribution> {
    vec![
        backend_api(
            "sync.api.status",
            "GET",
            "/api/sync/status",
            "同步状态",
            "返回当前设备、默认同步根、同步文件和 WebSocket 传输状态。",
            10,
        ),
        backend_api(
            "sync.api.files",
            "GET",
            "/api/sync/files",
            "同步文件列表",
            "列出已纳入 CRDT 同步索引的文件。",
            20,
        ),
        backend_api(
            "sync.api.roots",
            "POST",
            "/api/sync/roots",
            "添加同步根",
            "按用户家目录相对路径添加同步根目录。",
            30,
        ),
        backend_api(
            "sync.api.apply-text",
            "POST",
            "/api/sync/files/apply-text",
            "应用文本内容",
            "把本地文件全文按行级 CRDT 差异写入同步文档。",
            40,
        ),
        backend_api(
            "sync.api.delete-text",
            "POST",
            "/api/sync/files/delete-text",
            "删除文本片段",
            "按 Unicode 字符位置删除文本片段并生成 CRDT 增量。",
            50,
        ),
        backend_api(
            "sync.api.import-update",
            "POST",
            "/api/sync/files/import-update",
            "导入远端增量",
            "导入另一台设备通过 WebSocket 推送的 CRDT 更新。",
            60,
        ),
        backend_api(
            "sync.api.websocket",
            "GET",
            "/api/sync/ws",
            "WebSocket 同步通道",
            "建立低延迟 CRDT 增量交换通道。",
            70,
        ),
        backend_api(
            "sync.api.finder-status",
            "GET",
            "/api/sync/finder/status",
            "Finder 状态",
            "读取 macOS Finder Sync 扩展兼容的本地状态 JSON。",
            80,
        ),
        backend_api(
            "sync.api.finder-refresh",
            "POST",
            "/api/sync/finder/refresh",
            "刷新 Finder 状态",
            "把当前同步根和文件状态写入 Finder Sync 扩展读取的 state.json。",
            90,
        ),
    ]
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

#[cfg(test)]
mod tests {
    use az_aio_plugin_api::api::NativeAzAioPlugin;

    use super::SyncPlugin;

    #[test]
    fn plugin_declares_sync_surfaces() -> anyhow::Result<()> {
        let descriptor = SyncPlugin.descriptor();
        assert_eq!(descriptor.id, "sync");
        assert!(descriptor
            .capabilities
            .iter()
            .any(|capability| capability == "crdt-line-sync"));
        assert!(descriptor
            .capabilities
            .iter()
            .any(|capability| capability == "finder-status"));

        let contributions = SyncPlugin.contributions()?;
        assert!(contributions
            .backend_apis
            .iter()
            .any(|api| api.path == "/api/sync/ws"));
        assert!(contributions
            .settings_sections
            .iter()
            .flat_map(|section| section.defaults.iter())
            .any(|default| default.key == "sync.default_root" && default.value == "az-sync"));
        Ok(())
    }
}
