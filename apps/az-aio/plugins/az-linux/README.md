# AZ Linux

Native plugin for Linux server onboarding from the client side.

## Runtime

- Dioxus UI contract page: `linux.page`
- Route: `/linux`
- Axum APIs: `/api/linux/status`, `/api/linux/profiles`, `/api/linux/bootstrap-plan`, `/api/linux/bootstrap-script`
- Persistence: no formal business data yet; generated plans are transient client contracts
- Target profile: Ubuntu first, via `LinuxEnvironmentAdapter`

## Domain

The plugin keeps client-side Linux onboarding contracts, Ubuntu environment planning, SSH snippet generation, and manual curl bootstrap script rendering in domain-first files. The future server CLI should reuse these contract shapes instead of creating a second drifting interface surface.
