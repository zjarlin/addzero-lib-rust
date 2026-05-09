//! Minimal Wasmtime-backed host for `az-wasm-plugin-api` plugins.

use az_wasm_plugin_api::{PluginError, PluginHandle, PluginManifest, PluginRegistry, PluginState};
use std::collections::BTreeMap;
use std::sync::RwLock;
use uuid::Uuid;
use wasmtime::{Engine, Instance, Linker, Module, Store};

const BUILTIN_ENTRY_PREFIX: &str = "builtin:";
const ON_LOAD_EXPORT: &str = "aio_on_load";
const ON_ENABLE_EXPORT: &str = "aio_on_enable";
const ON_DISABLE_EXPORT: &str = "aio_on_disable";
const ON_UNLOAD_EXPORT: &str = "aio_on_unload";

/// In-memory plugin registry backed by Wasmtime for external plugins.
pub struct RuntimePluginRegistry {
    engine: Engine,
    plugins: RwLock<BTreeMap<Uuid, RuntimePlugin>>,
}

struct RuntimePlugin {
    handle: PluginHandle,
    execution: PluginExecution,
}

enum PluginExecution {
    Builtin,
    Wasm(WasmPluginInstance),
}

struct WasmPluginInstance {
    store: Store<()>,
    instance: Instance,
}

impl RuntimePluginRegistry {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
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
        wasm_bytes: Vec<u8>,
    ) -> Result<PluginHandle, PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;

        if plugins
            .values()
            .any(|plugin| plugin.handle.manifest.id == manifest.id)
        {
            return Err(PluginError::AlreadyLoaded(manifest.id.clone()));
        }

        let execution = if is_builtin_manifest(&manifest) {
            PluginExecution::Builtin
        } else {
            let mut instance = instantiate_wasm_plugin(&self.engine, &manifest, &wasm_bytes)?;
            instance.call_lifecycle(ON_LOAD_EXPORT)?;
            PluginExecution::Wasm(instance)
        };

        let handle = PluginHandle {
            id: Uuid::new_v4(),
            manifest,
            state: PluginState::Installed,
        };
        plugins.insert(
            handle.id,
            RuntimePlugin {
                handle: handle.clone(),
                execution,
            },
        );
        Ok(handle)
    }

    fn unload(&self, id: &Uuid) -> Result<(), PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;
        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        if let PluginExecution::Wasm(instance) = &mut plugin.execution {
            instance.call_lifecycle(ON_UNLOAD_EXPORT)?;
        }
        plugins.remove(id);
        Ok(())
    }

    fn enable(&self, id: &Uuid) -> Result<(), PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;
        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        if plugin.handle.state == PluginState::Active {
            return Ok(());
        }
        if let PluginExecution::Wasm(instance) = &mut plugin.execution {
            instance.call_lifecycle(ON_ENABLE_EXPORT)?;
        }
        plugin.handle.state = PluginState::Active;
        Ok(())
    }

    fn disable(&self, id: &Uuid) -> Result<(), PluginError> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| PluginError::Other(format!("registry lock poisoned: {e}")))?;
        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        if plugin.handle.state == PluginState::Disabled {
            return Ok(());
        }
        if let PluginExecution::Wasm(instance) = &mut plugin.execution {
            instance.call_lifecycle(ON_DISABLE_EXPORT)?;
        }
        plugin.handle.state = PluginState::Disabled;
        Ok(())
    }

    fn list(&self) -> Vec<PluginHandle> {
        self.plugins
            .read()
            .map(|guard| guard.values().map(|plugin| plugin.handle.clone()).collect())
            .unwrap_or_default()
    }
}

impl WasmPluginInstance {
    fn call_lifecycle(&mut self, export: &str) -> Result<(), PluginError> {
        let Some(func) = self.instance.get_func(&mut self.store, export) else {
            return Ok(());
        };
        let typed = func
            .typed::<(), i32>(&self.store)
            .map_err(|err| PluginError::Wasm(format!("{export} has invalid signature: {err}")))?;
        let status = typed
            .call(&mut self.store, ())
            .map_err(|err| PluginError::Wasm(format!("{export} failed: {err}")))?;
        if status == 0 {
            Ok(())
        } else {
            Err(PluginError::Wasm(format!(
                "{export} returned non-zero status {status}"
            )))
        }
    }
}

fn instantiate_wasm_plugin(
    engine: &Engine,
    manifest: &PluginManifest,
    wasm_bytes: &[u8],
) -> Result<WasmPluginInstance, PluginError> {
    if wasm_bytes.is_empty() {
        return Err(PluginError::Wasm(format!(
            "plugin `{}` did not provide wasm bytes",
            manifest.id
        )));
    }

    let module = Module::from_binary(engine, wasm_bytes)
        .map_err(|err| PluginError::Wasm(format!("failed to compile `{}`: {err}", manifest.id)))?;
    let linker = Linker::new(engine);
    let mut store = Store::new(engine, ());
    let instance = linker.instantiate(&mut store, &module).map_err(|err| {
        PluginError::Wasm(format!("failed to instantiate `{}`: {err}", manifest.id))
    })?;
    Ok(WasmPluginInstance { store, instance })
}

fn is_builtin_manifest(manifest: &PluginManifest) -> bool {
    manifest.entry.starts_with(BUILTIN_ENTRY_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use az_wasm_plugin_api::ExtensionPoint;

    fn manifest(id: &str, entry: &str) -> PluginManifest {
        PluginManifest {
            id: id.to_string(),
            name: "Test Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "test plugin".to_string(),
            author: "test".to_string(),
            min_platform_version: "0.1.0".to_string(),
            entry: entry.to_string(),
            extension_points: vec![ExtensionPoint::TemplateGenerator],
            permissions: Vec::new(),
            metadata: Default::default(),
        }
    }

    fn wasm_module(source: &str) -> Vec<u8> {
        wat::parse_str(source).expect("test wat should compile")
    }

    #[test]
    fn load_should_reject_external_plugin_without_wasm_bytes() {
        let registry = RuntimePluginRegistry::new();

        let err = registry
            .load(manifest("com.example.empty", "plugin.wasm"), Vec::new())
            .unwrap_err();

        assert!(
            err.to_string().contains("did not provide wasm bytes"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn load_should_accept_builtin_plugin_without_wasm_bytes() {
        let registry = RuntimePluginRegistry::new();

        let handle = registry
            .load(manifest("com.example.builtin", "builtin:test"), Vec::new())
            .expect("builtin plugin should load without wasm");

        assert_eq!(handle.state, PluginState::Installed);
    }

    #[test]
    fn load_should_call_on_load_for_external_wasm_plugin() {
        let registry = RuntimePluginRegistry::new();
        let bytes = wasm_module(
            r#"
            (module
                (func (export "aio_on_load") (result i32)
                    i32.const 0)
            )
            "#,
        );

        let handle = registry
            .load(manifest("com.example.valid", "plugin.wasm"), bytes)
            .expect("wasm plugin should load");

        assert_eq!(handle.state, PluginState::Installed);
    }

    #[test]
    fn load_should_fail_when_on_load_returns_non_zero() {
        let registry = RuntimePluginRegistry::new();
        let bytes = wasm_module(
            r#"
            (module
                (func (export "aio_on_load") (result i32)
                    i32.const 7)
            )
            "#,
        );

        let err = registry
            .load(manifest("com.example.fail-load", "plugin.wasm"), bytes)
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("aio_on_load returned non-zero status 7"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn enable_disable_and_unload_should_call_lifecycle_exports() {
        let registry = RuntimePluginRegistry::new();
        let bytes = wasm_module(
            r#"
            (module
                (global $counter (mut i32) (i32.const 0))
                (func (export "aio_on_load") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    i32.const 0)
                (func (export "aio_on_enable") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    i32.const 0)
                (func (export "aio_on_disable") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    i32.const 0)
                (func (export "aio_on_unload") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    i32.const 0)
            )
            "#,
        );
        let handle = registry
            .load(manifest("com.example.lifecycle", "plugin.wasm"), bytes)
            .expect("wasm plugin should load");

        registry
            .enable(&handle.id)
            .expect("enable lifecycle should succeed");
        registry
            .disable(&handle.id)
            .expect("disable lifecycle should succeed");
        registry
            .unload(&handle.id)
            .expect("unload lifecycle should succeed");

        assert!(registry.list().is_empty());
    }

    #[test]
    fn unload_should_keep_plugin_registered_when_on_unload_fails() {
        let registry = RuntimePluginRegistry::new();
        let bytes = wasm_module(
            r#"
            (module
                (func (export "aio_on_load") (result i32)
                    i32.const 0)
                (func (export "aio_on_unload") (result i32)
                    i32.const 9)
            )
            "#,
        );
        let handle = registry
            .load(manifest("com.example.fail-unload", "plugin.wasm"), bytes)
            .expect("wasm plugin should load");

        let err = registry.unload(&handle.id).unwrap_err();

        assert!(
            err.to_string()
                .contains("aio_on_unload returned non-zero status 9"),
            "unexpected error: {err}"
        );
        assert_eq!(registry.list().len(), 1);
    }
}
