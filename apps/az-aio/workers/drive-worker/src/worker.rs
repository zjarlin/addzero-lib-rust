use az_aio_plugin_api::api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, PluginActivation, PluginDescriptor, PluginKind,
};
use az_aio_plugin_api::register_native_plugin;

#[derive(Default)]
pub struct DriveWorkerPlugin;

impl NativeAzAioPlugin for DriveWorkerPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "drive-worker".to_string(),
            name: "Drive CRDT Sync".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "WebSocket-based line-CRDT text synchronization backed by the Drive Git Pool store."
                .to_string(),
            activation: PluginActivation::Eager,
            priority: 900,
            dependencies: Vec::new(),
            capabilities: vec![
                "drive-crdt-sync".to_string(),
                "websocket-sync".to_string(),
                "backend-api".to_string(),
            ],
            permissions: vec![
                "read-drive-metadata".to_string(),
                "write-drive-metadata".to_string(),
                "read-drive-objects".to_string(),
                "write-drive-objects".to_string(),
                "network-drive-sync".to_string(),
            ],
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            ui_contributions: Vec::new(),
            backend_apis: vec![
                backend_api(
                    "drive-worker.ws",
                    "GET",
                    "/ws/drive-sync",
                    "Drive CRDT WebSocket",
                    "accepts WebSocket connections for line-CRDT text sync backed by the Drive Git Pool store.",
                    10,
                ),
                backend_api(
                    "drive-worker.health",
                    "GET",
                    "/api/drive-worker/health",
                    "Drive Worker health",
                    "returns ok when the worker thread is alive and stores are accessible.",
                    20,
                ),
            ],
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(DriveWorkerPlugin);

pub fn ensure_linked() {}

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

    use super::DriveWorkerPlugin;

    #[test]
    fn plugin_declares_drive_sync_surfaces() -> anyhow::Result<()> {
        let descriptor = DriveWorkerPlugin.descriptor();
        assert_eq!(descriptor.id, "drive-worker");
        assert!(descriptor
            .capabilities
            .iter()
            .any(|c| c == "drive-crdt-sync"));

        let contributions = DriveWorkerPlugin.contributions()?;
        assert!(contributions
            .backend_apis
            .iter()
            .any(|api| api.path == "/ws/drive-sync"));
        assert!(contributions
            .backend_apis
            .iter()
            .any(|api| api.path == "/api/drive-worker/health"));
        Ok(())
    }
}
