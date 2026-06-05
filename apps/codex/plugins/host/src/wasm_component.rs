#![forbid(unsafe_code)]

use std::{path::Path, path::PathBuf};

use codex_plugin_api::{
    CodexPlugin, ContributionSet, PluginDescriptor, PluginError, contributions_from_json,
    descriptor_from_json,
};
use wasmtime::{
    Engine, Store,
    component::{Component, Linker},
};

pub struct WasmComponentPlugin {
    path: PathBuf,
    descriptor: PluginDescriptor,
    contributions: ContributionSet,
}

impl WasmComponentPlugin {
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self, PluginError> {
        let path = path.into();
        let fallback_id = path.display().to_string();
        let descriptor_json = call_result_string(&path, &fallback_id, "describe")?;
        let descriptor = descriptor_from_json(&descriptor_json)?;
        let contributions_json = call_result_string(&path, &descriptor.id, "contributions")?;
        let contributions = contributions_from_json(&contributions_json)?;

        Ok(Self {
            path,
            descriptor,
            contributions,
        })
    }
}

impl CodexPlugin for WasmComponentPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        self.descriptor.clone()
    }

    fn contributions(&self) -> Result<ContributionSet, PluginError> {
        Ok(self.contributions.clone())
    }

    fn on_load(&mut self) -> Result<(), PluginError> {
        call_result_unit(&self.path, &self.descriptor.id, "on-load")
    }

    fn on_enable(&mut self) -> Result<(), PluginError> {
        call_result_unit(&self.path, &self.descriptor.id, "on-enable")
    }

    fn on_disable(&mut self) -> Result<(), PluginError> {
        call_result_unit(&self.path, &self.descriptor.id, "on-disable")
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        call_result_unit(&self.path, &self.descriptor.id, "on-unload")
    }
}

fn call_result_string(
    path: &Path,
    plugin_id: &str,
    export_name: &str,
) -> Result<String, PluginError> {
    let (engine, component) = load_component(path, plugin_id)?;
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(|error| wasm_error(plugin_id, error))?;
    let function = instance
        .get_typed_func::<(), (Result<String, String>,)>(&mut store, export_name)
        .map_err(|error| wasm_error(plugin_id, error))?;
    let (result,) = function
        .call(&mut store, ())
        .map_err(|error| wasm_error(plugin_id, error))?;
    function
        .post_return(&mut store)
        .map_err(|error| wasm_error(plugin_id, error))?;
    result.map_err(|message| PluginError::Wasm {
        plugin: plugin_id.to_string(),
        message,
    })
}

fn call_result_unit(path: &Path, plugin_id: &str, export_name: &str) -> Result<(), PluginError> {
    let (engine, component) = load_component(path, plugin_id)?;
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(|error| wasm_error(plugin_id, error))?;
    let function = instance
        .get_typed_func::<(), (Result<(), String>,)>(&mut store, export_name)
        .map_err(|error| wasm_error(plugin_id, error))?;
    let (result,) = function
        .call(&mut store, ())
        .map_err(|error| wasm_error(plugin_id, error))?;
    function
        .post_return(&mut store)
        .map_err(|error| wasm_error(plugin_id, error))?;
    result.map_err(|message| PluginError::Wasm {
        plugin: plugin_id.to_string(),
        message,
    })
}

fn load_component(path: &Path, plugin_id: &str) -> Result<(Engine, Component), PluginError> {
    let engine = Engine::default();
    let component =
        Component::from_file(&engine, path).map_err(|error| wasm_error(plugin_id, error))?;
    Ok((engine, component))
}

fn wasm_error(plugin_id: &str, error: impl std::fmt::Display) -> PluginError {
    PluginError::Wasm {
        plugin: plugin_id.to_string(),
        message: error.to_string(),
    }
}
