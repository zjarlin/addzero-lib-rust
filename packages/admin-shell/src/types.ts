import type { ReactNode } from "react";

export type AdminActionId =
  | "theme-toggle"
  | "focus-search"
  | "logout"
  | "notifications"
  | (string & {});

export interface TopbarAction {
  id: AdminActionId;
  label: string;
  icon?: ReactNode;
  title?: string;
  tone?: "default" | "accent" | "danger";
  disabled?: boolean;
  onSelect?: () => void;
}

export interface MenuNode {
  id: string;
  label: string;
  href: string;
  activePatterns: string[];
  permissionsAnyOf?: string[];
  children?: MenuNode[];
}

export interface SectionNode {
  id: string;
  label: string;
  menus: MenuNode[];
}

export interface DomainNode {
  id: string;
  label: string;
  href: string;
  order: number;
}

export interface RightPanelSlot {
  title: string;
  content: ReactNode;
}

export interface AdminShellState {
  brandTitle: string;
  brandDetail: string;
  topbarActions: TopbarAction[];
  domains: DomainNode[];
  sections: SectionNode[];
  rightPanel?: RightPanelSlot | null;
}

export interface AdminShellContext {
  currentPath: string;
  isDark: boolean;
  username: string;
  permissions: string[] | null;
  onNavigate: (href: string) => void;
  onLogout: () => void;
  onToggleTheme: () => void;
  onFocusSearch: () => void;
}

export interface AdminProvider {
  getShellState(context: AdminShellContext): AdminShellState;
}
