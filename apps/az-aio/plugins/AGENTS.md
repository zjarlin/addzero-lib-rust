# AZ Platform Plugin Rules

## Naming

- The platform name is `az-platform`.
- Plugins are named by domain or capability group, not by frontend/backend, runtime origin, or implementation role.
- Planned plugin IDs are:
  - `navigation`
  - `catalog`
  - `settings`
  - `search`
  - `projects`
  - `git/skills`
  - `git/clis`
  - `git/envs`
  - `git/notes`
- `git/notes` is a reserved placeholder until note functionality is implemented.
- Do not introduce `builtin/*`, `native/*`, or `external/*` plugin identities. A plugin shipped with the app and a plugin installed later must use the same packaged Wasm loading path.

## Contribution Contracts

- Every plugin must declare frontend UI placement through `UiContribution`.
- Required slot semantics:
  - `navigation` contributes shell navigation and app-level route metadata.
  - `catalog` contributes plugin catalog content and catalog toolbar actions.
  - `settings` contributes `settings-content`.
  - `search` contributes a search route/content surface.
  - `projects` contributes both `project-sidebar` and `project-content`.
  - `git/*` plugins contribute workflow content or sandbox/debug panels under the plugin surface.
- Plugins exposing backend behavior must declare `BackendApiContribution`.
- Sandbox-visible debug data must flow through `PluginSandboxDebugReport` instead of ad hoc table-only output.

## Settings And Data

- The settings plugin owns project defaults.
- The default project sync root is `az-sync/workspace` and must remain visible in generated package and sandbox output as `projects.default_sync_root`.
- Database tables must follow the repo naming rule:
  - business tables use `biz_`
  - system tables use `sys_`
  - table names must not include app-name prefixes such as `az_aio_`

## Wasm And Xtask

- Every planned plugin must be independently addressable by `cargo xtask az-platform plugin <command> <plugin>`.
- Supported aggregate checks must include:
  - `cargo xtask az-platform plugin build all`
  - `cargo xtask az-platform plugin build-wasm all`
  - `cargo xtask az-platform plugin package all`
  - `cargo xtask az-platform plugin sandbox all`
- `sandbox <plugin>` must run the packaged manifest path, not an in-repo linked plugin path.
- Packaged manifests live under `target/az-platform/plugins/<plugin>/az-plugin.json`.
- Plugins with host filesystem behavior export descriptor, UI contributions, backend API contracts, and sandbox debug metadata from Wasm; actual filesystem work must be routed through platform host APIs.
- Wasm descriptors must not keep host-only dependencies that prevent single-plugin sandbox loading.

## Frontend And Backend Packaging

- A single plugin can contain frontend and backend contracts together.
- Generated packages must include:
  - `az-plugin.json`
  - `frontend/az-frontend.json`
  - `backend/az-backend.json`
  - the plugin `.component.wasm` when the plugin has a wasm target
- The package should remain inspectable from both `sandbox <plugin>` and `sandbox <az-plugin.json>`.

## Shared UI

- Shared Dioxus UI components belong in `crates/ui/az-dioxus-components`.
- Do not recreate a separate `az-table` crate; table primitives and data table components live inside `az-dioxus-components`.
