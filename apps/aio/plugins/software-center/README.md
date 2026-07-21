# Software Center

Native plugin for installer scanning and software package workflows.

## Runtime

- Dioxus UI contract page: `software-center.page`
- Route: `/software`
- Axum APIs: `/api/software-center/status`, `/api/software-center/installers`, `/api/software-center/organize`, `/api/software-center/packages`, `/api/software-center/package`
- Toasty table prefix: `biz_software_center_`
- Rudi context: `store::build_software_center_context`

## Domain

Installer scanning, archive path resolution, and catalog name matching remain in domain-first files under this plugin crate.
