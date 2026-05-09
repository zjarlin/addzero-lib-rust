# addzero-wasm-plugin-host

Minimal Wasmtime-backed host for `addzero-wasm-plugin-api` plugins.

## What It Provides

- `RuntimePluginRegistry`

## Example

```rust
use addzero_wasm_plugin_host::RuntimePluginRegistry;

let registry = RuntimePluginRegistry::new();
assert!(registry.list().is_empty());
```

## Scope

This crate manages plugin lifecycle and WASM instantiation. It is intentionally
small and does not define marketplace, UI, or business plugin models.
