# AIO Plugin Root

This directory is the only repo-local root for AIO plugins.

- `repos/`: source repositories for decoupled `addzero-plugin-*` projects. These can be mounted here as git submodules.
- `catalog/`: built `.azplugin` wasm packages that the host can install.
- `host/`: unpacked runtime host cache for installed wasm plugin packages.

The main program must not hardcode business pages or load business plugins from Rust crates. Business plugins should be packaged as wasm bundles and loaded from this root.
