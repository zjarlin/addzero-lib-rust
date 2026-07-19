# Drive Center

Native plugin for drive queue and hosting workflows.

## Runtime

- Dioxus UI contract page: `drive-center.page`
- Route: `/drive`
- Axum APIs: `/api/drive-center/status`, `/api/drive-center/tasks`, `/api/drive-center/task`
- Toasty table prefix: `biz_drive_center_`
- Rudi context: `store::build_drive_center_context`

## Domain

The plugin owns drive task models and routes directly under this crate instead of depending on desktop host services.
