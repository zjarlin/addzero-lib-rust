# AIO Drive Finder Sync

This directory contains the native macOS Finder Sync integration for AIO Drive.

The extension injects Finder context-menu items and delegates actual work to the
existing `az-drive-app` CLI:

- `AIO Drive 托管`
- `AIO Drive 取消托管`

The extension watches the user's home directory and `/Volumes`, which makes the
menu available for normal user files and external volumes. Runtime logs are
written to `~/Library/Logs/az-drive-finder-sync.log`.

Install from the repository root:

```sh
apps/drive/macos/finder-sync/install.sh
```
