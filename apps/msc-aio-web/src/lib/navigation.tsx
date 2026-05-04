"use client";

import type {
  AdminProvider,
  AdminShellContext,
  AdminShellState,
  DomainNode,
  MenuNode,
  SectionNode,
} from "@addzero/admin-shell";

const domains: DomainNode[] = [
  { id: "overview", label: "总览", href: "/dashboard", order: 10 },
  { id: "agents", label: "Agent资产", href: "/agents", order: 20 },
  { id: "chat", label: "AI聊天", href: "/chat", order: 30 },
  { id: "knowledge", label: "知识库", href: "/knowledge/notes", order: 40 },
  { id: "system", label: "系统管理", href: "/system/users", order: 50 },
  { id: "audit", label: "审计日志", href: "/audit", order: 60 },
];

const sectionsByDomain: Record<string, SectionNode[]> = {
  overview: [
    {
      id: "overview",
      label: "总览",
      menus: [
        {
          id: "dashboard",
          label: "笔记工作台",
          href: "/dashboard",
          activePatterns: ["/", "/dashboard"],
          permissionsAnyOf: ["overview"],
        },
      ],
    },
  ],
  agents: [
    {
      id: "agents",
      label: "Agent资产",
      menus: [
        {
          id: "skills",
          label: "Skill 资产",
          href: "/agents",
          activePatterns: ["/agents", "/agents/editor"],
          permissionsAnyOf: ["knowledge:skill"],
        },
      ],
    },
  ],
  chat: [
    {
      id: "chat",
      label: "聊天",
      menus: [
        {
          id: "chat-workbench",
          label: "聊天工作台",
          href: "/chat",
          activePatterns: ["/chat"],
        },
      ],
    },
  ],
  knowledge: [
    {
      id: "knowledge",
      label: "知识库",
      menus: [
        {
          id: "notes",
          label: "笔记",
          href: "/knowledge/notes",
          activePatterns: ["/knowledge/notes"],
          permissionsAnyOf: ["knowledge:note"],
        },
        {
          id: "packages",
          label: "下载与安装",
          href: "/knowledge/packages",
          activePatterns: ["/knowledge/packages"],
          permissionsAnyOf: ["knowledge:pkg", "knowledge:dl"],
        },
        {
          id: "cli-market",
          label: "CLI 市场",
          href: "/knowledge/cli-market",
          activePatterns: [
            "/knowledge/cli-market",
            "/knowledge/cli-market/imports",
            "/knowledge/cli-market/docs",
          ],
          permissionsAnyOf: ["knowledge:cli"],
          children: [
            {
              id: "cli-market-registry",
              label: "注册表",
              href: "/knowledge/cli-market",
              activePatterns: ["/knowledge/cli-market"],
              permissionsAnyOf: ["knowledge:cli"],
            },
            {
              id: "cli-market-imports",
              label: "导入任务",
              href: "/knowledge/cli-market/imports",
              activePatterns: ["/knowledge/cli-market/imports"],
              permissionsAnyOf: ["knowledge:cli"],
            },
            {
              id: "cli-market-docs",
              label: "CLI 文档",
              href: "/knowledge/cli-market/docs",
              activePatterns: ["/knowledge/cli-market/docs"],
              permissionsAnyOf: ["knowledge:cli"],
            },
          ],
        },
        {
          id: "download-station",
          label: "下载站",
          href: "/download-station",
          activePatterns: ["/download-station", "/files"],
          permissionsAnyOf: ["knowledge:dl"],
        },
      ],
    },
  ],
  system: [
    {
      id: "system",
      label: "系统管理",
      menus: [
        {
          id: "users",
          label: "用户",
          href: "/system/users",
          activePatterns: ["/system/users"],
          permissionsAnyOf: ["system:user"],
        },
        {
          id: "menus",
          label: "菜单",
          href: "/system/menus",
          activePatterns: ["/system/menus"],
          permissionsAnyOf: ["system:menu"],
        },
        {
          id: "roles",
          label: "角色",
          href: "/system/roles",
          activePatterns: ["/system/roles"],
          permissionsAnyOf: ["system:role"],
        },
        {
          id: "departments",
          label: "部门",
          href: "/system/departments",
          activePatterns: ["/system/departments"],
          permissionsAnyOf: ["system:dept"],
        },
        {
          id: "dictionaries",
          label: "字典管理",
          href: "/system/dictionaries",
          activePatterns: ["/system/dictionaries"],
          permissionsAnyOf: ["system:dict"],
        },
        {
          id: "settings",
          label: "系统设置",
          href: "/system/settings",
          activePatterns: ["/system/settings"],
          permissionsAnyOf: ["system:setting"],
        },
      ],
    },
  ],
  audit: [
    {
      id: "audit",
      label: "审计日志",
      menus: [
        {
          id: "audit-log",
          label: "审计日志",
          href: "/audit",
          activePatterns: ["/audit"],
          permissionsAnyOf: ["audit"],
        },
      ],
    },
  ],
};

function activeDomain(path: string) {
  if (path.startsWith("/agents")) return "agents";
  if (path.startsWith("/chat")) return "chat";
  if (path.startsWith("/knowledge") || path.startsWith("/download-station") || path.startsWith("/files")) {
    return "knowledge";
  }
  if (path.startsWith("/system")) return "system";
  if (path.startsWith("/audit")) return "audit";
  return "overview";
}

function isAllowed(menu: MenuNode, permissions: string[] | null) {
  if (!menu.permissionsAnyOf?.length || permissions === null) {
    return true;
  }
  return menu.permissionsAnyOf.some((permission) => permissions.includes(permission));
}

function filterMenus(menus: MenuNode[], permissions: string[] | null): MenuNode[] {
  return menus
    .filter((menu) => isAllowed(menu, permissions))
    .map((menu) => ({
      ...menu,
      children: menu.children
        ? filterMenus(menu.children, permissions).filter((child) => isAllowed(child, permissions))
        : undefined,
    }));
}

export const adminProvider: AdminProvider = {
  getShellState(context: AdminShellContext): AdminShellState {
    const currentDomain = activeDomain(context.currentPath);
    const rawSections = sectionsByDomain[currentDomain] ?? sectionsByDomain.overview;
    const sections = rawSections.map((section) => ({
      ...section,
      menus: filterMenus(section.menus, context.permissions),
    }));

    return {
      brandTitle: "MSC_AIO",
      brandDetail: context.username
        ? `${context.username} · 双轴上下文工作台`
        : "双轴上下文工作台",
      domains,
      sections,
      topbarActions: [
        {
          id: "theme-toggle",
          label: context.isDark ? "切到浅色" : "切到深色",
          onSelect: context.onToggleTheme,
        },
        {
          id: "focus-search",
          label: "搜索",
          onSelect: context.onFocusSearch,
        },
        {
          id: "notifications",
          label: "通知",
          disabled: true,
        },
        {
          id: "logout",
          label: "退出登录",
          tone: "danger",
          onSelect: context.onLogout,
        },
      ],
      rightPanel: {
        title: "二维上下文",
        content: (
          <div className="space-y-3">
            <p>当前壳子维持主轴 domain 与侧轴 section 的双轴约定。</p>
            <p className="text-zinc-500">骨架层不写死业务按钮、路由分支或菜单字符串。</p>
          </div>
        ),
      },
    };
  },
};
