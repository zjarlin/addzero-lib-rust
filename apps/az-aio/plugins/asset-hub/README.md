# Asset Hub

Native plugin for asset workflows.

## Runtime

- Dioxus UI contract page: `asset-hub.page`
- Route: `/assets`
- Axum APIs: `/api/asset-hub/status`, `/api/asset-hub/skills`, `/api/asset-hub/assets`, `/api/asset-hub/asset`
- Toasty table prefix: `biz_asset_hub_`
- Rudi context: `store::build_asset_hub_context`

## Domain

The plugin keeps skill scanning logic in `skill_scanner.rs` and exposes it through the native API and page renderer.
