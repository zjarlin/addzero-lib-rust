# Asset Hub

Native AZ AIO plugin for asset workflows.

## Runtime

- Dioxus renderer: `asset-hub.page`
- Route: `/assets`
- Axum APIs: `/api/asset-hub/status`, `/api/asset-hub/skills`, `/api/asset-hub/assets`, `/api/asset-hub/asset`
- Toasty table prefix: `biz_asset_hub_`
- shaku module: `store::AssetHubModule`

## Domain

The plugin keeps skill scanning logic in `skill_scanner.rs` and exposes it through the native API and page renderer.
