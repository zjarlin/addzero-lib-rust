export type AioDomainId = "operations" | "knowledge" | "environment";

export interface AioToolbarAction {
  id: string;
  label: string;
  tooltip: string;
  order: number;
  icon: string;
  primary?: boolean;
}

export interface AioDomain {
  id: AioDomainId;
  label: string;
  order: number;
  defaultRoute: string;
  icon: string;
}

export interface AioBranch {
  id: string;
  domainId: AioDomainId;
  parentId?: string;
  label: string;
  order: number;
  icon: string;
}

export interface AioPage {
  id: string;
  domainId: AioDomainId;
  branchId: string;
  title: string;
  subtitle: string;
  route: string;
  order: number;
  icon: string;
  pinned?: boolean;
  toolbarActions: AioToolbarAction[];
}

export interface AioSummaryCard {
  id: string;
  title: string;
  summary: string;
  route: string;
  order: number;
}

export interface AioNavManifest {
  domains: AioDomain[];
  branches: AioBranch[];
  pages: AioPage[];
  summaryCards: AioSummaryCard[];
}

export interface ActionResult {
  message: string;
}

export interface DesktopRuntimeInfo {
  baseUrl: string;
  desktopToken?: string;
}

export interface DriveSnapshot {
  roots: unknown[];
  hosted: unknown[];
  tracked: unknown[];
  conflicts: unknown[];
  queue: unknown[];
}

export interface GatewayRuntimeStep {
  id: string;
  kind: string;
  label: string;
  method: string;
  url: string;
  headers: Record<string, string>;
  bodyPreview: string;
  capturePath: string;
  dependsOn: string[];
  inputRefs: string[];
  notes: string;
}

export interface GatewayRunRequest {
  entryRoute: string;
  input: unknown;
  steps: GatewayRuntimeStep[];
}

export interface GatewayRunStepResult {
  id: string;
  label: string;
  ok: boolean;
  statusCode?: number;
  requestUrl: string;
  captured?: unknown;
  error?: string;
  durationMs: number;
}

export interface GatewayRunResult {
  entryRoute: string;
  finalResult?: unknown;
  message: string;
  ok: boolean;
  status: string;
  steps: GatewayRunStepResult[];
}

export interface AssetGraph {
  items?: unknown[];
  nodes?: unknown[];
  edges?: unknown[];
  tags?: unknown[];
}

export interface SoftwareCatalog {
  items: Array<{id?: string; slug: string; title: string; vendor?: string; tags?: string[]}>;
  hostPlatform?: string;
}

export interface InstallerPackage {
  id: string;
  fileName: string;
  sourcePath: string;
  version: string;
  platform: string;
  arch: string;
  targetPath: string;
  installStatus: string;
  status: string;
  md5: string;
}

export interface AiProviderConfig {
  provider: string;
  label: string;
  enabled: boolean;
  apiKeyConfigured: boolean;
  defaultModel: string;
  baseUrl?: string;
}

export interface ConfigLocalStatus {
  dotfiles: {
    root: string;
    watchedFiles: number;
    changedFiles: number;
    conflictFiles: number;
    devices: unknown[];
    pendingFiles: unknown[];
    conflicts: unknown[];
  };
  pairing: {
    deviceName: string;
    fingerprint: string;
    homePath: string;
    metadataPath: string;
  };
  xdgPaths: {
    dataDir: string;
    configDir: string;
    stateDir: string;
    cacheDir: string;
  };
  providers: AiProviderConfig[];
}
