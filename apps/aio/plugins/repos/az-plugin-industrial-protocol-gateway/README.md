# Industrial Protocol Gateway Plugin

This is an AIO business plugin for an industrial protocol conversion gateway.
It ships as a `.azplugin` package and contributes static gateway workbench pages
through `plugin.toml`.

## Scope

- Declares Modbus RTU, Modbus TCP, OPC UA, MQTT, and HTTP JSON endpoints.
- Declares conversion routes with explicit source, target, mapping, and cadence.
- Provides dashboard, route table, topology, endpoint policy, and runtime config
  pages for the current AIO plugin schema renderer.
- Keeps field-bus details out of the AIO shell. Real serial and Modbus I/O should
  remain in shared protocol crates before this plugin binds runtime operations.

## Build Package

```bash
python3 apps/aio/plugins/repos/az-plugin-industrial-protocol-gateway/scripts/package.py
```

The package is written to:

```text
apps/aio/plugins/catalog/industrial-protocol-gateway.azplugin
```

If `wasm32-unknown-unknown` is installed, the script builds the Rust plugin
runtime. Otherwise it falls back to a lifecycle-only WASM module that exposes the
host-required `aio_on_load`, `aio_on_enable`, `aio_on_disable`, and
`aio_on_unload` functions.

## Local Validation

```bash
cargo test --manifest-path apps/aio/plugins/repos/az-plugin-industrial-protocol-gateway/Cargo.toml
python3 apps/aio/plugins/repos/az-plugin-industrial-protocol-gateway/scripts/package.py --skip-cargo
cargo test -p az-plugin-runtime
```
