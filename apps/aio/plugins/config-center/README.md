# Config Center

Native plugin for machine configuration workflows.

## Runtime

- Dioxus UI contract page: `config-center.page`
- Route: `/config`
- Axum APIs: `/api/config-center/status`, `/api/config-center/dotfiles`, `/api/config-center/pairing`, `/api/config-center/entries`, `/api/config-center/entry`
- Toasty table prefix: `biz_config_center_`
- Rudi context: `store::build_config_center_context`

## Domain

The plugin keeps dotfiles monitoring, pairing identity, and XDG path resolution in domain-first files beside the native runtime files.
