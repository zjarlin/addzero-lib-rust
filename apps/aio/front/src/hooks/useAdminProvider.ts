import { createElement, useCallback, useEffect, useState } from "react";
import { Bot } from "lucide-react";
import type {
    AdminProvider,
    AdminShellContext,
    AdminShellState,
    DomainNode,
    MenuNode,
    SectionNode,
} from "@az/admin-shell";
import { Button } from "@az/ui";
import {
    getApiBaseUrl,
    createMenuTreeApi,
    type MenuTreeNodeDto,
} from "@az/api-client";
import {
    fetchWasmPluginOverview,
    type WasmPluginNavigationSection,
} from "../lib/wasm-plugin-runtime";
import { emitAiWorkspacePanel } from "../lib/ai-workspace";

const DEFAULT_DOMAINS: DomainNode[] = [
    {
        id: "assets",
        label: "资产",
        href: "/assets/notes",
        activePatterns: [
            "/assets",
            "/assets/notes",
            "/assets/packages",
            "/assets/dotfiles",
            "/assets/agents",
            "/skills",
            "/",
        ],
        order: 0,
    },
    {
        id: "runtime",
        label: "运行",
        href: "/console",
        activePatterns: ["/console", "/env"],
        order: 2,
    },
    {
        id: "plugins",
        label: "插件",
        href: "/market",
        activePatterns: ["/market", "/apps"],
        order: 3,
    },
    {
        id: "system",
        label: "系统",
        href: "/system",
        activePatterns: ["/system"],
        order: 4,
    },
];

const FALLBACK_SCENES: Record<string, SectionNode[]> = {
    assets: [
        {
            id: "personal-assets",
            label: "个人资产",
            menus: [
                {
                    id: "asset-notes",
                    label: "笔记",
                    href: "/assets/notes",
                    activePatterns: ["/assets/notes"],
                },
                {
                    id: "asset-packages",
                    label: "安装包",
                    href: "/assets/packages",
                    activePatterns: ["/assets/packages"],
                },
                {
                    id: "asset-dotfiles",
                    label: "dotfiles",
                    href: "/assets/dotfiles",
                    activePatterns: ["/assets/dotfiles"],
                },
            ],
        },
        {
            id: "agent-assets",
            label: "Agent资产",
            menus: [
                {
                    id: "agent-skills",
                    label: "Skill",
                    href: "/assets/agents/skills",
                    activePatterns: ["/assets/agents/skills", "/skills"],
                },
                {
                    id: "agent-cli",
                    label: "CLI",
                    href: "/assets/agents/cli",
                    activePatterns: ["/assets/agents/cli"],
                },
                {
                    id: "agent-mcp",
                    label: "MCP",
                    href: "/assets/agents/mcp",
                    activePatterns: ["/assets/agents/mcp"],
                },
            ],
        },
    ],
    runtime: [
        {
            id: "runtime",
            label: "运行时",
            menus: [
                {
                    id: "script-console",
                    label: "脚本控制台",
                    href: "/console",
                    activePatterns: ["/console"],
                },
                {
                    id: "runtime-config",
                    label: "环境与配置",
                    href: "/env",
                    activePatterns: ["/env"],
                },
            ],
        },
    ],
    plugins: [
        {
            id: "plugin-system",
            label: "插件系统",
            menus: [
                {
                    id: "wasm-market",
                    label: "WASM 插件市场",
                    href: "/market",
                    activePatterns: ["/market"],
                },
            ],
        },
    ],
    system: [
        {
            id: "system",
            label: "系统",
            menus: [
                {
                    id: "system-admin",
                    label: "系统管理",
                    href: "/system",
                    activePatterns: ["/system"],
                },
            ],
        },
    ],
};

const SCENE_META: Record<string, { title: string; detail: string }> = {
    assets: {
        title: "资产路由树",
        detail: "笔记、安装包、dotfiles",
    },
    runtime: {
        title: "运行路由树",
        detail: "脚本、环境、配置",
    },
    plugins: {
        title: "插件路由树",
        detail: "WASM 插件市场与扩展点",
    },
    system: {
        title: "系统路由树",
        detail: "权限、菜单、系统设置",
    },
};

function pathMatchesPattern(path: string, pattern: string) {
    const cleanPath = path.split("?")[0].replace(/\/+$/, "") || "/";
    const cleanPattern = pattern.replace(/\/+$/, "") || "/";
    if (cleanPattern === "/") {
        return cleanPath === "/";
    }
    return cleanPath === cleanPattern || cleanPath.startsWith(`${cleanPattern}/`);
}

function activeSceneId(currentPath: string) {
    const domain = [...DEFAULT_DOMAINS]
        .sort((a, b) => b.order - a.order)
        .find((item) =>
            (item.activePatterns ?? [item.href]).some((pattern) =>
                pathMatchesPattern(currentPath, pattern),
            ),
        );
    return domain?.id ?? "assets";
}

function isAuditLabel(label: string) {
    const normalized = label.trim().toLowerCase();
    return (
        normalized.includes("审计日志") ||
        normalized.includes("audit log") ||
        normalized === "审计" ||
        normalized === "audit"
    );
}

function mapTreeToMenuNodes(nodes: MenuTreeNodeDto[]): MenuNode[] {
    return nodes
        .filter((node) => !isAuditLabel(node.title))
        .map((node) => ({
            id: node.id,
            label: node.title,
            href: node.route_path,
            activePatterns: [node.route_path],
            children:
                node.children.length > 0
                    ? mapTreeToMenuNodes(node.children)
                    : undefined,
        }));
}

function mergeRemoteMenuTree(
    sceneSections: Record<string, SectionNode[]>,
    tree: MenuTreeNodeDto[],
) {
    if (tree.length === 0) {
        return sceneSections;
    }

    return {
        ...sceneSections,
        system: [
            ...sceneSections.system,
            {
                id: "remote-menu-tree",
                label: "权限菜单树",
                menus: mapTreeToMenuNodes(tree),
            },
        ],
    };
}

function mapPluginNavigationSections(
    sections: WasmPluginNavigationSection[],
): SectionNode[] {
    return sections
        .map((section) => ({
            id: `plugin:${section.label}`,
            label: section.label,
            menus: section.items
                .filter(
                    (item) =>
                        (item.plugin_id || item.kind !== "Fixed") &&
                        !isAuditLabel(item.label),
                )
                .map(mapPluginNavigationItem),
        }))
        .filter((section) => section.menus.length > 0);
}

function mergePluginNavigationSections(
    sceneSections: Record<string, SectionNode[]>,
    sections: SectionNode[],
) {
    if (sections.length === 0) {
        return sceneSections;
    }

    return {
        ...sceneSections,
        plugins: [
            ...sceneSections.plugins,
            ...filterPluginSections(sections, "/market"),
            ...mergeAppSections(sections),
        ],
        system: [
            ...sceneSections.system,
            ...filterPluginSections(sections, "/system/"),
        ],
    };
}

function mergeAppSections(pluginSections: SectionNode[]) {
    const appPluginSections = filterPluginSections(pluginSections, "/apps/");
    return appPluginSections;
}

function filterPluginSections(
    sections: SectionNode[],
    hrefPrefix: string,
): SectionNode[] {
    return sections
        .map((section) => ({
            ...section,
            menus: section.menus.filter((menu) =>
                menu.href.startsWith(hrefPrefix),
            ),
        }))
        .filter((section) => section.menus.length > 0);
}

function mapPluginNavigationItem(item: WasmPluginNavigationSection["items"][number]): MenuNode {
    return {
        id: `${item.plugin_id ?? "fixed"}:${item.page_id ?? item.href}`,
        label: item.badge ? `${item.label}` : item.label,
        href: item.href,
        activePatterns: [item.href],
    };
}

export function useAdminProvider(): {
    provider: AdminProvider;
    loading: boolean;
    username: string;
} {
    const [sceneSections, setSceneSections] = useState(FALLBACK_SCENES);
    const [loading, setLoading] = useState(true);
    const [username, setUsername] = useState("");

    useEffect(() => {
        const baseUrl = getApiBaseUrl();
        const menuApi = createMenuTreeApi(baseUrl);
        let cancelled = false;

        async function load() {
            try {
                const [tree, session, overview] = await Promise.all([
                    menuApi.getMenuTree(),
                    fetch(`${baseUrl}/api/admin/session`, {
                        credentials: "include",
                    }).then((r) => r.json()),
                    fetchWasmPluginOverview(baseUrl),
                ]);
                if (cancelled) return;
                const dynamicSections = mapPluginNavigationSections(
                    overview.shell.nav_sections,
                );
                setUsername(session.username ?? "");
                setSceneSections(
                    mergePluginNavigationSections(
                        mergeRemoteMenuTree(FALLBACK_SCENES, tree ?? []),
                        dynamicSections,
                    ),
                );
            } catch {
                // keep static navigation available while backend or plugin runtime is offline
            } finally {
                if (!cancelled) setLoading(false);
            }
        }

        void load();
        const refresh = () => {
            void load();
        };
        window.addEventListener("aio:plugin-runtime-updated", refresh);
        return () => {
            cancelled = true;
            window.removeEventListener("aio:plugin-runtime-updated", refresh);
        };
    }, []);

    const getShellState = useCallback(
        (context: AdminShellContext): AdminShellState => {
            const sceneId = activeSceneId(context.currentPath);
            const meta = SCENE_META[sceneId] ?? SCENE_META.assets;

            return {
                brandTitle: "AIO Platform",
                brandDetail: "Script Runtime + Vibe Coding + Plugin Workbench",
                topbarActions: [
                    { id: "theme-toggle", label: "主题" },
                    { id: "focus-search", label: "搜索" },
                    { id: "logout", label: "登出" },
                ],
                topbarContentEnd: createElement(
                    Button,
                    {
                        type: "button",
                        variant: "ghost",
                        size: "icon",
                        title: "AI 整理",
                        "aria-label": "AI 整理",
                        className:
                            "h-10 w-10 rounded-full border border-stone-900/10 bg-[#171915] text-stone-50 shadow-[0_10px_24px_rgba(23,25,21,0.16)] hover:bg-[#11130f] hover:text-stone-50",
                        onClick: () => emitAiWorkspacePanel({ toggle: true }),
                    },
                    createElement(Bot, { className: "h-4 w-4" }),
                ),
                domains: DEFAULT_DOMAINS,
                sections: sceneSections[sceneId] ?? FALLBACK_SCENES.assets,
                navigationTitle: meta.title,
                navigationDetail: meta.detail,
                rightPanel: null,
            };
        },
        [sceneSections],
    );

    return { provider: { getShellState }, loading, username };
}
