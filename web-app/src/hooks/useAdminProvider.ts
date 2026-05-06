import { useCallback, useEffect, useState } from "react";
import type {
  AdminProvider,
  AdminShellState,
  DomainNode,
  MenuNode,
  SectionNode,
} from "@addzero/admin-shell";
import { getApiBaseUrl } from "@addzero/api-client";
import { createMenuTreeApi, type MenuTreeNodeDto } from "@addzero/api-client/menu-tree";

// ─── Default domains (topbar) ──────────────────────────────────────

const DEFAULT_DOMAINS: DomainNode[] = [
  { id: "dashboard", label: "Dashboard", href: "/", order: 0 },
  { id: "knowledge", label: "知识库", href: "/knowledge", order: 1 },
  { id: "storage", label: "存储", href: "/storage", order: 2 },
  { id: "system", label: "系统", href: "/system", order: 3 },
];

// ─── Mapping helpers ───────────────────────────────────────────────

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
      id: "main",
      label: "导航",
      menus: DEFAULT_DOMAINS.map((d) => ({
        id: d.id,
        label: d.label,
        href: d.href,
        activePatterns: [d.href],
      })),
    },
  ];
}

// ─── Hook ───────────────────────────────────────────────────────────

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
      } catch (err) {
        console.error("Admin shell: failed to load menu tree:", err);
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
      brandDetail: "脚本化运行平台",
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

  return {
    provider: { getShellState },
    loading,
    username,
  };
}
