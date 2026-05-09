# az-wasm-plugin-api

Host-side contract types for loading and managing WASM plugins.

## What It Provides

- `PluginManifest`
- `ExtensionPoint`
- `PluginState`
- `PluginHandle`
- `PluginRegistry`
- `PluginError`

## Example

```rust
use az_wasm_plugin_api::{ExtensionPoint, PluginManifest};

let manifest = PluginManifest {
    id: "com.example.demo".to_string(),
    name: "Demo Plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "A demo plugin".to_string(),
    author: "example".to_string(),
    min_platform_version: "0.1.0".to_string(),
    entry: "plugin.wasm".to_string(),
    extension_points: vec![ExtensionPoint::ScriptEngine],
    permissions: vec![],
    metadata: Default::default(),
};

assert_eq!(manifest.name, "Demo Plugin");
```

## Scope

This crate defines shared host-side types. It does not load WASM modules by
itself.
