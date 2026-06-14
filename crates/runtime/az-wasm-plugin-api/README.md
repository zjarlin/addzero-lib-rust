# az-wasm-plugin-api

用于加载和管理 WASM 插件的宿主端契约类型。

## 提供内容

- `PluginManifest`
- `ExtensionPoint`
- `PluginState`
- `PluginHandle`
- `PluginRegistry`
- `PluginError`

## 示例

```rust
use az_wasm_plugin_api::api::{ExtensionPoint, PluginManifest};

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

## 适用范围

此 crate 定义共享的宿主端类型，本身不负责加载 WASM 模块。
