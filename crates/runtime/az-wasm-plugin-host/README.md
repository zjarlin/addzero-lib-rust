# az-wasm-plugin-host

基于 Wasmtime 的 `az-wasm-plugin-api` 插件最小化宿主。

## 提供内容

- `RuntimePluginRegistry`

## 示例

```rust
use az_wasm_plugin_api::api::PluginRegistry;
use az_wasm_plugin_host::api::RuntimePluginRegistry;

let registry = RuntimePluginRegistry::new();
assert!(registry.list().is_empty());
```

## 适用范围

此 crate 管理插件生命周期和 WASM 实例化。它刻意保持精简，不定义市场、UI 或业务插件模型。
