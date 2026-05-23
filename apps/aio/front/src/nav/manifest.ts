import type {AioNavManifest} from "../types";

export const aioNavManifest: AioNavManifest = {
  domains: [
    {id: "operations", label: "Operations", icon: "LayoutDashboard", order: 10, defaultRoute: "/drive"},
    {id: "knowledge", label: "Knowledge", icon: "Boxes", order: 20, defaultRoute: "/assets"},
    {id: "environment", label: "Environment", icon: "Wrench", order: 30, defaultRoute: "/config"}
  ],
  branches: [
    {id: "operations-storage", domainId: "operations", label: "Storage", icon: "HardDrive", order: 10},
    {id: "operations-edge", domainId: "operations", label: "Network", icon: "Network", order: 20},
    {id: "knowledge-assets", domainId: "knowledge", label: "Assets", icon: "FolderTree", order: 10},
    {id: "knowledge-software", domainId: "knowledge", label: "Software", icon: "PackageSearch", order: 20},
    {id: "environment-machine", domainId: "environment", label: "Machine", icon: "Database", order: 10}
  ],
  pages: [
    {
      id: "drive-center",
      domainId: "operations",
      branchId: "operations-storage",
      title: "Drive Center",
      subtitle: "Host, sync, inspect queue and conflicts.",
      route: "/drive",
      order: 10,
      icon: "HardDrive",
      pinned: true,
      toolbarActions: [
        {id: "drive.refresh", label: "Refresh", icon: "RefreshCw", tooltip: "Reload drive snapshot", order: 10},
        {id: "drive.sync", label: "Sync", icon: "FolderSync", tooltip: "Run one sync cycle", order: 20, primary: true},
        {id: "drive.retry-queue", label: "Retry Queue", icon: "RotateCcw", tooltip: "Retry queued sync items", order: 30},
        {id: "drive.host-skills", label: "Host Skills", icon: "CloudUpload", tooltip: "Host ~/.agents/skills", order: 40},
        {id: "drive.unhost-skills", label: "Unhost Skills", icon: "Archive", tooltip: "Unhost ~/.agents/skills", order: 50}
      ]
    },
    {
      id: "edge-gateway",
      domainId: "operations",
      branchId: "operations-edge",
      title: "Edge Gateway",
      subtitle: "Gateway flow editor, plan generation, runtime execution, and helper references.",
      route: "/gateway",
      order: 20,
      icon: "Network",
      toolbarActions: [
        {id: "edge-gateway.refresh", label: "Refresh", icon: "RefreshCw", tooltip: "Refresh gateway panel state", order: 10},
        {id: "edge-gateway.load-example", label: "Load Example", icon: "Copy", tooltip: "Load a reference gateway plan", order: 20},
        {id: "edge-gateway.run-example", label: "Run Example", icon: "Play", tooltip: "Execute the loaded example gateway plan", order: 30, primary: true}
      ]
    },
    {
      id: "asset-hub",
      domainId: "knowledge",
      branchId: "knowledge-assets",
      title: "Asset Hub",
      subtitle: "Asset editor/feed, skill scan, compose assets, tag filters, and detail surfaces.",
      route: "/assets",
      order: 10,
      icon: "Boxes",
      toolbarActions: [
        {id: "asset-hub.refresh", label: "Refresh", icon: "RefreshCw", tooltip: "Reload asset graph", order: 10},
        {id: "asset-hub.sync", label: "Sync", icon: "FolderSync", tooltip: "Sync asset graph", order: 20, primary: true}
      ]
    },
    {
      id: "software-center",
      domainId: "knowledge",
      branchId: "knowledge-software",
      title: "Software Center",
      subtitle: "Installer scan, organize/archive, and catalog-linked package detail surfaces.",
      route: "/software",
      order: 20,
      icon: "PackageSearch",
      toolbarActions: [
        {id: "software-center.refresh", label: "Refresh", icon: "RefreshCw", tooltip: "Reload catalog and installer scan", order: 10},
        {id: "software-center.scan-installers", label: "Scan", icon: "Search", tooltip: "Scan Downloads and Desktop for installers", order: 20, primary: true},
        {id: "software-center.organize-installers", label: "Organize", icon: "Archive", tooltip: "Archive detected installers", order: 30}
      ]
    },
    {
      id: "config-center",
      domainId: "environment",
      branchId: "environment-machine",
      title: "Config Center",
      subtitle: "Dotfiles monitor, pairing identity, XDG paths, and model provider configuration.",
      route: "/config",
      order: 10,
      icon: "Database",
      toolbarActions: [
        {id: "config-center.refresh", label: "Refresh", icon: "RefreshCw", tooltip: "Reload local config status", order: 10},
        {id: "config-center.import-env-providers", label: "Import Env", icon: "Upload", tooltip: "Import provider secrets from environment", order: 20, primary: true},
        {id: "config-center.test-openai", label: "Test OpenAI", icon: "Zap", tooltip: "Test OpenAI provider connectivity", order: 30},
        {id: "config-center.test-anthropic", label: "Test Anthropic", icon: "Zap", tooltip: "Test Anthropic provider connectivity", order: 40},
        {id: "config-center.test-gemini", label: "Test Gemini", icon: "Zap", tooltip: "Test Gemini provider connectivity", order: 50}
      ]
    }
  ],
  summaryCards: [
    {id: "drive-center-summary", title: "Drive Center", summary: "Realtime drive operations, queue, conflicts, pool surfaces, and hosting status.", route: "/drive", order: 10},
    {id: "asset-hub-summary", title: "Asset Hub", summary: "Asset editor/feed, skill scan, compose assets, tag filters, and detail surfaces.", route: "/assets", order: 20},
    {id: "software-center-summary", title: "Software Center", summary: "Installer scan, archive flow, and catalog linkage.", route: "/software", order: 30},
    {id: "edge-gateway-summary", title: "Edge Gateway", summary: "Flow runtime reference with HTTP chain execution.", route: "/gateway", order: 50},
    {id: "config-center-summary", title: "Config Center", summary: "Dotfiles conflict audit, pairing identity, XDG/backend panels, and provider testing.", route: "/config", order: 60}
  ]
};

export const sortedPages = [...aioNavManifest.pages].sort((left, right) => left.order - right.order);

export function pageForRoute(route: string) {
  return sortedPages.find(page => page.route === route) ?? sortedPages[0];
}
