import { getApiBaseUrl } from "@az/api-client";

export interface WasmPluginNavigationItem {
    label: string;
    href: string;
    plugin_id: string | null;
    page_id: string | null;
    badge: string | null;
    kind: "Fixed" | "SystemPage" | "BusinessInstance";
}

export interface WasmPluginNavigationSection {
    label: string;
    items: WasmPluginNavigationItem[];
}

export interface WasmPluginMarketplaceEntry {
    plugin_id: string;
    name: string;
    version: string;
    kind: "System" | "Business";
    summary: string;
    tags: string[];
    icon: string | null;
    compatibility: string[];
    capabilities: string[];
    status: "Available" | "Installed" | "Disabled";
    instances: number;
}

export interface WasmPluginRuntimeOverview {
    counts: {
        system_plugins: number;
        installed_business_plugins: number;
        plugin_instances: number;
    };
    package_root: string;
    dev_auth_mode: string;
}

export interface WasmPluginShellSnapshot {
    actor: {
        username: string;
        display_name: string;
        roles: string[];
    };
    nav_sections: WasmPluginNavigationSection[];
    counts: {
        system_plugins: number;
        installed_business_plugins: number;
        plugin_instances: number;
    };
    dev_auth_mode: string;
}

export interface WasmPluginRuntimeSnapshot {
    shell: WasmPluginShellSnapshot;
    marketplace: {
        entries: WasmPluginMarketplaceEntry[];
        tags: string[];
    };
    runtime: WasmPluginRuntimeOverview;
}

export interface WasmPluginUploadRequest {
    file_name: string;
    bytes: number[];
}

export interface WasmPluginUploadResult {
    package_path: string;
    plugin_id: string;
    plugin_name: string;
    version: string;
    validated: boolean;
}

export interface WasmPluginInstallRequest {
    plugin_id: string;
    instance_label?: string | null;
}

export interface WasmPluginInstallResult {
    plugin_id: string;
    plugin_name: string;
    version: string;
    instance_slug: string;
    instance_label: string;
    page_ids: string[];
}

export interface WasmPluginResolvedPage {
    scope: "Fixed" | "System" | "Instance";
    plugin_id: string;
    plugin_name: string;
    page_id: string;
    title: string;
    subtitle: string;
    breadcrumbs: string[];
    schema: WasmPluginPageSchema;
}

export type WasmPluginPageSchema =
    | {
          kind: "markdown";
          body: string;
      }
    | {
          kind: "table";
          columns: string[];
          rows: { cells: string[] }[];
          empty_message: string;
      }
    | {
          kind: "board";
          metrics: { label: string; value: string; detail: string }[];
          groups: {
              title: string;
              items: { title: string; detail: string; meta: string }[];
          }[];
      }
    | {
          kind: "detail";
          summary: string;
          fields: { label: string; value: string; readonly: boolean }[];
          timeline: { title: string; detail: string; meta: string }[];
      }
    | {
          kind: "form";
          fields: { label: string; value: string; readonly: boolean }[];
      }
    | {
          kind: "graph";
          nodes: {
              id: string;
              label: string;
              category: string;
              description: string;
              details: string;
          }[];
          edges: {
              source: string;
              target: string;
              kind: string;
              label?: string | null;
          }[];
      };

async function requestJson<T>(
    path: string,
    init?: RequestInit,
    baseUrl = getApiBaseUrl(),
): Promise<T> {
    const response = await fetch(`${baseUrl}${path}`, {
        credentials: "include",
        ...init,
        headers: {
            "Content-Type": "application/json",
            ...(init?.headers ?? {}),
        },
    });
    if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
    }
    return (await response.json()) as T;
}

export function fetchWasmPluginOverview(baseUrl = getApiBaseUrl()) {
    return requestJson<WasmPluginRuntimeSnapshot>(
        "/api/wasm/plugins/overview",
        undefined,
        baseUrl,
    );
}

export function uploadWasmPlugin(
    input: WasmPluginUploadRequest,
    baseUrl = getApiBaseUrl(),
) {
    return requestJson<WasmPluginUploadResult>(
        "/api/wasm/plugins/upload",
        {
            method: "POST",
            body: JSON.stringify(input),
        },
        baseUrl,
    );
}

export function installCatalogWasmPlugin(
    input: WasmPluginInstallRequest,
    baseUrl = getApiBaseUrl(),
) {
    return requestJson<WasmPluginInstallResult>(
        "/api/wasm/plugins/install-catalog",
        {
            method: "POST",
            body: JSON.stringify(input),
        },
        baseUrl,
    );
}

export function fetchSystemWasmPluginPage(
    pluginId: string,
    pageId: string,
    baseUrl = getApiBaseUrl(),
) {
    return requestJson<WasmPluginResolvedPage>(
        `/api/wasm/plugins/system/${encodeURIComponent(pluginId)}/${encodeURIComponent(pageId)}`,
        undefined,
        baseUrl,
    );
}

export function fetchInstanceWasmPluginPage(
    instanceSlug: string,
    pageId: string,
    baseUrl = getApiBaseUrl(),
) {
    return requestJson<WasmPluginResolvedPage>(
        `/api/wasm/plugins/apps/${encodeURIComponent(instanceSlug)}/${encodeURIComponent(pageId)}`,
        undefined,
        baseUrl,
    );
}
