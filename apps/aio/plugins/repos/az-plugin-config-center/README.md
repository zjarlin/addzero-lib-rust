# Config Center Plugin

This AIO business plugin provides an Infisical-like configuration center surface
without hardcoding business pages into the host application.

## Scope

- Uses `biz_dotfiles.xlsx` as the legacy metadata seed for shell exports,
  aliases, suffix aliases, functions, variables, and shell snippets.
- Models the target shell sync root as `~/.config/shell`.
- Keeps secret-like values out of `plugin.toml`; the page only declares
  metadata, redaction policy, and target sync boundaries.
- Declares future host capabilities for audited config reads, shell sync
  previews, and OS environment writes.

## Boundaries

The plugin can describe pages and lifecycle hooks today. Actual writes to
operating-system environment locations or `~/.config/shell` must be host
capabilities with preview, approval, audit, and conflict handling. Rust libraries
that need runtime configuration should read through the config center contract
instead of scraping shell files directly.

## Build Package

```bash
python3 apps/aio/plugins/repos/az-plugin-config-center/scripts/package.py
```

The package is written to:

```text
apps/aio/plugins/catalog/config-center.azplugin
```

If `wasm32-unknown-unknown` is installed, the script builds the Rust plugin
runtime. Otherwise it falls back to a lifecycle-only WASM module that exports
the host-required lifecycle functions.

## Local Validation

```bash
cargo test --manifest-path apps/aio/plugins/repos/az-plugin-config-center/Cargo.toml
python3 apps/aio/plugins/repos/az-plugin-config-center/scripts/package.py --skip-cargo
```
