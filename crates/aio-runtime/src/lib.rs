//! AIO WASM 插件运行时 — Wasmtime 集成。
//!
//! 负责：
//! - 加载/卸载 `.aio-plugin` 包
//! - 沙箱隔离（独立 Wasmtime 实例）
//! - 热更新（不重启宿主）
//! - 调用 WIT 契约方法

use aio_plugin_api::{PluginError, PluginHandle, PluginManifest, PluginRegistry, PluginState};
use std::collections::BTreeMap;
use std::sync::RwLock;
use uuid::Uuid;

/// In-memory plugin registry (Wasmtime implementation stub).
///
/// Currently uses a simple map. When Wasmtime is integrated, each entry
/// will hold a `wasmtime::Instance` + `wasmtime::Store`.
pub struct RuntimePluginRegistry {
    plugins: RwLock<BTreeMap<Uuid, PluginHandle>>,
}

impl RuntimePluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for RuntimePluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry for RuntimePluginRegistry {
    fn load(
        &self,
        manifest: PluginManifest,
        _wasm_bytes: Vec<u8>,
    ) -> Result<PluginHandle, PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;

        // Prevent duplicate loads
        if plugins.values().any(|p| p.manifest.id == manifest.id) {
            return Err(PluginError::AlreadyLoaded(manifest.id.clone()));
        }

        let handle = PluginHandle {
            id: Uuid::new_v4(),
            manifest,
            state: PluginState::Installed,
        };
        plugins.insert(handle.id, handle.clone());
        Ok(handle)
    }

    fn unload(&self, id: &Uuid) -> Result<(), PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;
        plugins
            .remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        Ok(())
    }

    fn enable(&self, id: &Uuid) -> Result<(), PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;
        let handle = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        if handle.state == PluginState::Active {
            return Ok(());
        }
        handle.state = PluginState::Active;
        Ok(())
    }

    fn disable(&self, id: &Uuid) -> Result<(), PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;
        let handle = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        handle.state = PluginState::Disabled;
        Ok(())
    }

    fn list(&self) -> Vec<PluginHandle> {
        self.plugins
            .read()
            .map(|guard| guard.values().cloned().collect())
            .unwrap_or_default()
    }
}
