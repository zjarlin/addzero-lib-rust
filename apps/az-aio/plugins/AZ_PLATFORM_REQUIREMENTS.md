# AZ AIO Plugin Platform Requirements

## Platform Name

The plugin runtime platform is `az-platform`.

`az-platform` is the AZ AIO equivalent of an IDE platform: the desktop shell owns layout, runtime loading, sandboxing, shared UI components, and backend API routing, while plugins contribute UI and API surfaces.

## Plugin Naming

Plugins under `apps/az-aio/plugins` must be named by domain or capability group, not by implementation role or runtime origin.

Planned plugin groups:

- `navigation`
- `catalog`
- `settings`
- `search`
- `projects`
- `git`

Planned `git` subplugins:

- `git/skills`
- `git/clis`
- `git/envs`
- `git/notes`

`git/notes` is reserved for future note functionality and is not implemented yet.

Do not introduce `builtin/*`, `native/*`, or `external/*` plugin identities. A plugin shipped with the app and a plugin installed later must use the same packaged Wasm loading path.

## UI Contribution Contract

Every plugin frontend must declare where it contributes UI. The plugin API exposes `UiContribution` for this contract.

Required contribution slots:

- `app-sidebar`: app-level left rail and navigation entries.
- `app-topbar`: global toolbar or context controls.
- `content`: route-level content area.
- `settings-content`: settings panels and settings subroutes.
- `project-sidebar`: project tree/list/sidebar contributions.
- `project-content`: content shown when a project is selected.
- `sandbox-panel`: az-platform sandbox/debug panels.

Examples:

- `projects` contributes a project sidebar and the project detail content area.
- `settings` contributes settings content, including project defaults.
- `search` contributes a search route/content surface.
- `git/skills`, `git/clis`, `git/envs`, and future `git/notes` contribute git/workflow-specific UI under their feature routes and sandbox panels.

## Settings Requirements

The settings plugin must expose project defaults, including:

- default sync root: `az-sync/workspace`
- project binding defaults
- plugin/runtime configuration needed by `az-platform`

The default sync root is a structured frontend package contribution under the settings section:

```text
projects.default_sync_root = az-sync/workspace
```

Generated package and sandbox output must keep this value visible in `frontend/az-frontend.json` and in the `/az-platform` bundle contract view.

Settings data must follow the repository table naming rule: business tables use `biz_`, system tables use `sys_`, and table names must not include the app name such as `az_aio_`.

## Backend API Contribution Contract

Plugins that expose backend behavior must declare `BackendApiContribution` entries.

The `az-platform` sandbox must be able to inspect and debug:

- contributed UI slots
- backend API routes
- request/response examples once implemented
- plugin lifecycle status

The current sandbox contract exposes a structured `sandbox_debug` report in each generated `az-plugin.json`. It derives from the plugin's contribution set and currently includes:

- UI contribution debug rows with slot name, slot label, renderer id, route, and order.
- Backend API debug rows with method, path, label, description, and a request hint such as `GET /api/projects`.
- Settings default rows such as `projects.default_sync_root = az-sync/workspace`.

The host sandbox example also emits the same report in `--json` mode so packaged manifests and wasm components can be compared without reparsing table output.

## Wasm Runtime Direction

The target is runtime Wasm plugins.

Each plugin must eventually have:

- an independently compilable wasm component package
- an independently runnable `xtask` task
- frontend and backend code packaged as one independently runnable plugin artifact
- sandbox execution through `az-platform`

The main application must not link plugin crates directly. Shipped plugins are packaged Wasm plugins discovered through the same manifest/component loader as user-installed plugins.

## Xtask Requirements

Each plugin must have an xtask entry that can compile it independently.

The future command shape should be stable enough for automation, for example:

```shell
cargo xtask az-platform plugin build settings
cargo xtask az-platform plugin build all
cargo xtask az-platform plugin build projects
cargo xtask az-platform plugin build-wasm projects
cargo xtask az-platform plugin package projects
cargo xtask az-platform plugin sandbox projects
cargo xtask az-platform plugin sandbox all
cargo xtask az-platform plugin sandbox target/az-platform/plugins/projects/az-plugin.json
```

Exact task implementation can evolve, but each plugin must be addressable by feature-group name.

## Shared UI Component Boundary

Platform UI components belong in:

```text
crates/ui/az-dioxus-components
```

The former `crates/ui/az-table` table API is folded into `az-dioxus-components` as `az_table` primitives and `az_data_table` structured table components. New plugin and platform UI code must depend on `az-dioxus-components` directly instead of a separate table crate.

## Current First Slice

The first implementation slice adds explicit runtime contribution models:

- `UiContribution`
- `UiContributionSlot`
- `BackendApiContribution`

These models let runtime Wasm plugins declare frontend contribution positions and backend API surfaces before the full sandbox renderer is implemented.

The first sandbox slice also keeps every plugin's own `ContributionSet` in the host snapshot. This lets `az-platform` inspect a selected plugin without inferring ownership from contribution IDs.

Implemented sandbox entry points:

```shell
cargo xtask az-platform plugin package projects
cargo xtask az-platform plugin package all
cargo xtask az-platform plugin sandbox projects
cargo xtask az-platform plugin sandbox all
cargo xtask az-platform plugin sandbox target/az-platform/plugins/projects/az-plugin.json
cargo xtask az-platform plugin build-wasm projects
cargo xtask az-platform plugin build-wasm all
```

Desktop route:

```text
/az-platform
```

The desktop sandbox page lists plugins, UI contribution slots, backend APIs, capabilities, permissions, and lifecycle state.

The packaging slice writes an `az-plugin.json` bundle manifest under `target/az-platform/plugins/<plugin>/`. The manifest contains the actual plugin descriptor, contribution set, wasm component artifact metadata, and the sandbox command that can run the plugin through `az-platform`. The same sandbox command can also read a manifest path directly, so generated plugin bundles can be inspected without inferring contribution ownership from the host snapshot.

The package command also writes frontend and backend contract artifacts:

```text
target/az-platform/plugins/<plugin>/frontend/az-frontend.json
target/az-platform/plugins/<plugin>/backend/az-backend.json
```

The frontend artifact contains UI-facing contributions such as nav items, pages, UI slots, toolbar actions, catalog providers, and settings sections. The backend artifact contains backend API routes, shell entries, and generated-file contracts. These JSON artifacts are the current package boundary until real frontend bundles and backend binaries are introduced.

Running `cargo xtask az-platform plugin sandbox <plugin>` first packages that plugin, then runs the generated `target/az-platform/plugins/<plugin>/az-plugin.json` through the packaged sandbox path. Running `cargo xtask az-platform plugin sandbox all` repeats that packaged sandbox flow for every planned plugin. Running `cargo xtask az-platform plugin sandbox target/az-platform/plugins/<plugin>/az-plugin.json` inspects an existing package manifest, prints the frontend/backend bundle contracts, and loads the packaged `.component.wasm` through the `az-platform` host sandbox. This makes the generated plugin package independently runnable from either the plugin name or manifest path.

The same WIT `az-aio-plugin` component export applies to all planned plugins: `navigation`, `catalog`, `settings`, `search`, `projects`, `sync`, `git/skills`, `git/clis`, `git/envs`, and reserved `git/notes`. Each one can now be targeted through `cargo xtask az-platform plugin build-wasm <plugin>` and packaged into a manifest with a `.component.wasm` artifact.

Plugins with host filesystem behavior, such as `git/skills` and `git/clis`, export descriptor, UI contribution slots, and backend API contracts from Wasm; the backend implementation must be routed through `az-platform` host APIs instead of reading or writing host paths directly from Wasm.
