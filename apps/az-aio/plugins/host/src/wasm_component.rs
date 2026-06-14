#![forbid(unsafe_code)]

use std::{path::Path, path::PathBuf};

use az_aio_plugin_api::api::{
    AzAioPlugin, ContributionSet, PluginDescriptor, contributions_from_json, descriptor_from_json,
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
    pub fn from_file(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
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

impl AzAioPlugin for WasmComponentPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        self.descriptor.clone()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(self.contributions.clone())
    }

    fn on_load(&mut self) -> anyhow::Result<()> {
        call_result_unit(&self.path, &self.descriptor.id, "on-load")
    }

    fn on_enable(&mut self) -> anyhow::Result<()> {
        call_result_unit(&self.path, &self.descriptor.id, "on-enable")
    }

    fn on_disable(&mut self) -> anyhow::Result<()> {
        call_result_unit(&self.path, &self.descriptor.id, "on-disable")
    }

    fn on_unload(&mut self) -> anyhow::Result<()> {
        call_result_unit(&self.path, &self.descriptor.id, "on-unload")
    }
}

fn call_result_string(path: &Path, plugin_id: &str, export_name: &str) -> anyhow::Result<String> {
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
    result.map_err(|message| wasm_error(plugin_id, message))
}

fn call_result_unit(path: &Path, plugin_id: &str, export_name: &str) -> anyhow::Result<()> {
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
    result.map_err(|message| wasm_error(plugin_id, message))
}

fn load_component(path: &Path, plugin_id: &str) -> anyhow::Result<(Engine, Component)> {
    let engine = Engine::default();
    let component =
        Component::from_file(&engine, path).map_err(|error| wasm_error(plugin_id, error))?;
    Ok((engine, component))
}

fn wasm_error(plugin_id: &str, error: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!("Wasm 组件 `{plugin_id}` 运行失败：{error}")
}
