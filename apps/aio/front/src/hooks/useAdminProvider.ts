import { useCallback, useEffect, useState } from "react";
import type {
    AdminProvider,
    AdminShellContext,
    AdminShellState,
    DomainNode,
    MenuNode,
    SectionNode,
} from "@addzero/admin-shell";
import { getApiBaseUrl } from "@addzero/api-client";
import {
    createMenuTreeApi,
    type MenuTreeNodeDto,
} from "@addzero/api-client/menu-tree";
import {
    fetchWasmPluginOverview,
    type WasmPluginNavigationSection,
} from "../lib/wasm-plugin-runtime";

const DEFAULT_DOMAINS: DomainNode[] = [
    {
        id: "workbench",
        label: "工作台",
        href: "/",
        activePatterns: ["/"],
        order: 0,
    },
    {
        id: "assets",
        label: "资产",
        href: "/assets/files",
        activePatterns: [
            "/assets",
            "/assets/files",
            "/assets/notes",
            "/assets/packages",
            "/assets/dotfiles",
            "/assets/agents",
            "/assets/agents/skills",
            "/assets/agents/cli",
            "/assets/agents/mcp",
            "/storage",
            "/knowledge",
            "/skills",
        ],
        order: 1,
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
        activePatterns: ["/market"],
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
    workbench: [
        {
            id: "overview",
            label: "平台工作台",
            menus: [
                {
                    id: "platform-overview",
                    label: "平台总览",
                    href: "/",
                    activePatterns: ["/"],
                },
            ],
        },
    ],
    assets: [
        {
            id: "personal-assets",
            label: "个人资产",
            menus: [
                {
                    id: "asset-files",
                    label: "资产文件",
                    href: "/assets/files",
                    activePatterns: ["/assets/files", "/storage"],
                },
                {
                    id: "asset-notes",
                    label: "笔记",
                    href: "/assets/notes",
                    activePatterns: ["/assets/notes", "/knowledge"],
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
            label: "Agent 资产",
            menus: [
                {
                    id: "agent-assets-root",
                    label: "Agent 资产总览",
                    href: "/assets/agents",
                    activePatterns: ["/assets/agents"],
                    children: [
                        {
                            id: "agent-skills",
                            label: "Skills",
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
    workbench: {
        title: "工作台路由树",
        detail: "平台总览与全局状态",
    },
    assets: {
        title: "资产路由树",
        detail: "文件、笔记、安装包、dotfiles、Agent 资产",
    },
    runtime: {
        title: "运行路由树",
        detail: "脚本、环境、配置",
    },
    plugins: {
        title: "插件路由树",
        detail: "WASM 插件与扩展点",
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
    return domain?.id ?? "workbench";
}

function mapTreeToMenuNodes(nodes: MenuTreeNodeDto[]): MenuNode[] {
    return nodes.map((node) => ({
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
        .filter((section) =>
            section.items.some((item) => item.plugin_id || item.kind !== "Fixed"),
        )
        .map((section) => ({
            id: `plugin:${section.label}`,
            label: section.label,
            menus: section.items.map(mapPluginNavigationItem),
        }));
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
        plugins: [...sceneSections.plugins, ...sections],
    };
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
                // keep fallback
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
            const meta = SCENE_META[sceneId] ?? SCENE_META.workbench;

            return {
                brandTitle: "AIO Platform",
                brandDetail: "Script Runtime + Vibe Coding + Plugin Workbench",
                topbarActions: [
                    { id: "theme-toggle", label: "主题" },
                    { id: "focus-search", label: "搜索" },
                    { id: "logout", label: "登出" },
                ],
                domains: DEFAULT_DOMAINS,
                sections: sceneSections[sceneId] ?? FALLBACK_SCENES.workbench,
                navigationTitle: meta.title,
                navigationDetail: meta.detail,
                rightPanel: null,
            };
        },
        [sceneSections],
    );

    return { provider: { getShellState }, loading, username };
}
