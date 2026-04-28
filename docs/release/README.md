# Monorepo release strategy

This repository should not use one shared desktop-app release tag for every package.

For the remote desktop desktop app, release with an app-scoped tag:

```bash
scripts/release/remote-desktop-desktop.sh 0.1.0
```

That creates and pushes:

```text
remote-desktop-desktop-v0.1.0
```

The GitHub Actions workflow `.github/workflows/release-remote-desktop.yml` will then:

1. create or update a GitHub Release for that tag
2. build the macOS `.dmg` on `macos-latest`
3. build the Windows `.msi` on `windows-latest`
4. upload both installers to the Release

Current constraints:

- installers are unsigned
- workflow only releases `apps/remote-desktop-desktop`
- macOS and Windows are built natively on their own runners; no cross-bundling
