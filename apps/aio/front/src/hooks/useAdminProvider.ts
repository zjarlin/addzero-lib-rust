import { useCallback, useEffect, useState } from "react";
import type {
    AdminProvider,
    AdminShellState,
    DomainNode,
    MenuNode as AdminMenuNode,
    SectionNode,
} from "@addzero/admin-shell";
import { getApiBaseUrl } from "@addzero/api-client";
import {
    fetchWasmPluginOverview,
    type WasmPluginNavigationSection,
} from "../lib/wasm-plugin-runtime";

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

function mapPluginNavigationItem(item: WasmPluginNavigationSection["items"][number]): AdminMenuNode {
    return {
        id: `${item.plugin_id ?? "fixed"}:${item.page_id ?? item.href}`,
        label: item.badge ? `${item.label}` : item.label,
        href: item.href,
        activePatterns: [item.href],
    };
}

export function useAdminProvider(): {
    provider: AdminProvider;
} {
    const [sections, setSections] = useState<SectionNode[]>(fallbackSections);

    useEffect(() => {
        const baseUrl = getApiBaseUrl();
        let cancelled = false;

        async function load() {
            try {
                const overview = await fetchWasmPluginOverview(baseUrl);
                if (cancelled) return;
                const dynamicSections = mapPluginNavigationSections(
                    overview.shell.nav_sections,
                );
                if (dynamicSections.length > 0) {
                    setSections([...fallbackSections(), ...dynamicSections]);
                }
            } catch {
                // keep fallback
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

    return { provider: { getShellState } };
}
