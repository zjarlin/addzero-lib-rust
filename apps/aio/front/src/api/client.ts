import {invoke} from "@tauri-apps/api/core";
import type {
  ActionResult,
  AiProviderConfig,
  AssetGraph,
  ConfigLocalStatus,
  DesktopRuntimeInfo,
  DriveSnapshot,
  GatewayRunRequest,
  GatewayRunResult,
  InstallerPackage,
  SoftwareCatalog
} from "../types";

let runtimeInfo: DesktopRuntimeInfo | undefined;

export async function loadRuntimeInfo(): Promise<DesktopRuntimeInfo> {
  if (runtimeInfo) {
    return runtimeInfo;
  }

  if ("__TAURI_INTERNALS__" in window) {
    runtimeInfo = await invoke<DesktopRuntimeInfo>("runtimeInfo");
  } else {
    runtimeInfo = {
      baseUrl: import.meta.env.VITE_AIO_API_BASE_URL ?? "",
      desktopToken: import.meta.env.VITE_AIO_DESKTOP_TOKEN
    };
  }
  return runtimeInfo;
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const runtime = await loadRuntimeInfo();
  const headers = new Headers(init.headers);
  if (!headers.has("Content-Type") && init.body) {
    headers.set("Content-Type", "application/json");
  }
  if (runtime.desktopToken) {
    headers.set("x-aio-desktop-token", runtime.desktopToken);
  }

  const response = await fetch(`${runtime.baseUrl}${path}`, {
    ...init,
    credentials: "include",
    headers
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `HTTP ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: "POST",
    body: body === undefined ? undefined : JSON.stringify(body)
  });
}

export const api = {
  driveSnapshot: () => request<DriveSnapshot>("/api/aio/drive/snapshot"),
  driveHost: (path = "~/.agents/skills") => post<ActionResult>("/api/aio/drive/host", {path}),
  driveUnhost: (path = "~/.agents/skills") => post<ActionResult>("/api/aio/drive/unhost", {path}),
  driveSync: () => post<ActionResult>("/api/aio/drive/sync"),
  driveRetryQueue: () => post<ActionResult>("/api/aio/drive/retry-queue"),
  driveQueue: () => request<unknown[]>("/api/aio/drive/queue"),
  driveConflicts: () => request<unknown[]>("/api/aio/drive/conflicts"),
  driveTrackedRoots: () => request<unknown[]>("/api/aio/drive/tracked-roots"),
  gatewayExample: () => request<GatewayRunRequest>("/api/aio/gateway/example"),
  gatewayRun: (plan: GatewayRunRequest) => post<GatewayRunResult>("/api/aio/gateway/run", plan),
  assetGraph: () => request<AssetGraph>("/api/admin/assets/graph"),
  syncAssets: () => post<unknown>("/api/admin/assets/sync"),
  softwareCatalog: () => request<SoftwareCatalog>("/api/software-catalog"),
  softwareInstallers: () => request<InstallerPackage[]>("/api/aio/software/installers"),
  organizeInstallers: () => post<InstallerPackage[]>("/api/aio/software/installers/organize"),
  configLocalStatus: () => request<ConfigLocalStatus>("/api/aio/config-local/status"),
  importEnvProviders: () => post<ActionResult>("/api/aio/config-local/import-env-providers"),
  testProvider: (provider: AiProviderConfig["provider"]) =>
    post<ActionResult>("/api/aio/config-local/providers/test", {provider})
};
