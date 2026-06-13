# Software Center

Native AZ AIO plugin for installer scanning and software package workflows.

## Runtime

- Dioxus renderer: `software-center.page`
- Route: `/software`
- Axum APIs: `/api/software-center/status`, `/api/software-center/installers`, `/api/software-center/organize`, `/api/software-center/packages`, `/api/software-center/package`
- Toasty table prefix: `biz_software_center_`
- shaku module: `store::SoftwareCenterModule`

## Domain

Installer scanning, archive path resolution, and catalog name matching remain in domain-first files under this plugin crate.
