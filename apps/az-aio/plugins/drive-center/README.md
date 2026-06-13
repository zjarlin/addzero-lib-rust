# Drive Center

Native AZ AIO plugin for drive queue and hosting workflows.

## Runtime

- Dioxus renderer: `drive-center.page`
- Route: `/drive`
- Axum APIs: `/api/drive-center/status`, `/api/drive-center/tasks`, `/api/drive-center/task`
- Toasty table prefix: `biz_drive_center_`
- shaku module: `store::DriveCenterModule`

## Domain

The plugin owns drive task models and routes directly under this crate instead of depending on desktop host services.
