use az_wasm_plugin_api::{ExtensionPoint, PluginManifest, PluginRegistry, PluginState};
use az_wasm_plugin_host::RuntimePluginRegistry;
use wasmtime::Engine;

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
fn with_engine_should_accept_injected_wasmtime_engine() {
    let registry = RuntimePluginRegistry::with_engine(Engine::default());

    let handle = registry
        .load(manifest("com.example.injected", "builtin:test"), Vec::new())
        .expect("builtin plugin should load with injected engine");

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
