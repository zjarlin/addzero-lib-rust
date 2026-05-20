# AIO Front

React + TypeScript + Vite frontend for AIO.

This package is the only AIO web UI and is also embedded by the Tauri desktop shell in `apps/aio/desktop`.

```bash
npm install
npm run typecheck
npm run build
```

The frontend talks to `apps/aio/backend` over HTTP. In Tauri mode, the desktop shell injects the backend base URL and desktop session token through the `runtime_info` command.
