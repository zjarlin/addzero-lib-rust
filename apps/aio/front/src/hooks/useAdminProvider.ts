import { useCallback, useEffect, useState } from "react";
import type {
    AdminProvider,
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

const DEFAULT_DOMAINS: DomainNode[] = [
    { id: "workbench", label: "工作台", href: "/", order: 0 },
    { id: "scripts", label: "脚本引擎", href: "/console", order: 1 },
    { id: "orchestration", label: "编排", href: "/env", order: 2 },
    { id: "plugins", label: "插件", href: "/skills", order: 3 },
    { id: "knowledge", label: "知识", href: "/knowledge", order: 4 },
    { id: "assets", label: "资源", href: "/storage", order: 5 },
    { id: "market", label: "插件市场", href: "/market", order: 6 },
    { id: "system", label: "系统", href: "/system", order: 7 },
];

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

function fallbackSections(): SectionNode[] {
    return [
        {
            id: "platform",
            label: "平台工作台",
            menus: [
                {
                    id: "overview",
                    label: "平台总览",
                    href: "/",
                    activePatterns: ["/"],
                },
                {
                    id: "script-console",
                    label: "脚本控制台",
                    href: "/console",
                    activePatterns: ["/console"],
                },
                {
                    id: "env-lab",
                    label: "环境与配置",
                    href: "/env",
                    activePatterns: ["/env"],
                },
            ],
        },
        {
            id: "runtime",
            label: "运行与扩展",
            menus: [
                {
                    id: "skills",
                    label: "插件与技能",
                    href: "/skills",
                    activePatterns: ["/skills"],
                },
                {
                    id: "knowledge",
                    label: "知识与记忆",
                    href: "/knowledge",
                    activePatterns: ["/knowledge"],
                },
                {
                    id: "storage",
                    label: "存储与资源",
                    href: "/storage",
                    activePatterns: ["/storage"],
                },
                {
                    id: "market",
                    label: "WASM 插件市场",
                    href: "/market",
                    activePatterns: ["/market"],
                },
                {
                    id: "system",
                    label: "系统管理",
                    href: "/system",
                    activePatterns: ["/system"],
                },
            ],
        },
    ];
}

export function useAdminProvider(): {
    provider: AdminProvider;
    loading: boolean;
    username: string;
} {
    const [sections, setSections] = useState<SectionNode[]>(fallbackSections);
    const [loading, setLoading] = useState(true);
    const [username, setUsername] = useState("");

    useEffect(() => {
        const baseUrl = getApiBaseUrl();
        const menuApi = createMenuTreeApi(baseUrl);
        let cancelled = false;

        async function load() {
            try {
                const [tree, session] = await Promise.all([
                    menuApi.getMenuTree(),
                    fetch(`${baseUrl}/api/admin/session`, {
                        credentials: "include",
                    }).then((r) => r.json()),
                ]);
                if (cancelled) return;
                setUsername(session.username ?? "");
                if (tree && tree.length > 0) {
                    setSections([
                        {
                            id: "navigation",
                            label: "导航",
                            menus: mapTreeToMenuNodes(tree),
                        },
                    ]);
                }
            } catch {
                // keep fallback
            } finally {
                if (!cancelled) setLoading(false);
            }
        }

        load();
        return () => {
            cancelled = true;
        };
    }, []);

    const getShellState = useCallback(
        (): AdminShellState => ({
            brandTitle: "AIO Platform",
            brandDetail: "Script Runtime + Vibe Coding + Plugin Workbench",
            topbarActions: [
                { id: "theme-toggle", label: "主题" },
                { id: "focus-search", label: "搜索" },
                { id: "logout", label: "登出" },
            ],
            domains: DEFAULT_DOMAINS,
            sections,
            rightPanel: null,
        }),
        [sections],
    );

    return { provider: { getShellState }, loading, username };
}
