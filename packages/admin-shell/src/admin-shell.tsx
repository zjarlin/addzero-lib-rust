"use client";

import clsx from "clsx";
import { Bell, LogOut, MoonStar, Search, SunMedium } from "lucide-react";
import { useEffect, type ReactNode } from "react";

import type {
  AdminProvider,
  AdminShellContext,
  AdminShellState,
  DomainNode,
  MenuNode,
  RightPanelSlot,
  SectionNode,
  TopbarAction,
} from "./types";

function matchPattern(path: string, pattern: string) {
  const cleanPath = path.split("?")[0].replace(/\/+$/, "") || "/";
  const cleanPattern = pattern.replace(/\/+$/, "") || "/";
  const pathParts = cleanPath === "/" ? [] : cleanPath.slice(1).split("/");
  const patternParts =
    cleanPattern === "/" ? [] : cleanPattern.slice(1).split("/");

  if (pathParts.length !== patternParts.length) {
    return false;
  }

  return patternParts.every((part, index) => {
    return part.startsWith(":") || part === pathParts[index];
  });
}

function isMenuActive(path: string, menu: MenuNode) {
  return menu.activePatterns.some((pattern) => matchPattern(path, pattern));
}

function iconForAction(action: TopbarAction, isDark: boolean) {
  if (action.icon) {
    return action.icon;
  }

  switch (action.id) {
    case "theme-toggle":
      return isDark ? <SunMedium size={16} /> : <MoonStar size={16} />;
    case "focus-search":
      return <Search size={16} />;
    case "notifications":
      return <Bell size={16} />;
    case "logout":
      return <LogOut size={16} />;
    default:
      return null;
  }
}

function renderMenu(
  menu: MenuNode,
  currentPath: string,
  onNavigate: (href: string) => void,
  depth = 0,
) {
  const active = isMenuActive(currentPath, menu);

  return (
    <div key={menu.id} className="flex flex-col gap-1">
      <button
        type="button"
        onClick={() => onNavigate(menu.href)}
        className={clsx(
          "flex w-full items-center justify-between rounded-md border px-3 py-2 text-left text-sm transition",
          depth > 0 && "ml-4 w-[calc(100%-1rem)]",
          active
            ? "border-emerald-400/60 bg-emerald-500/10 text-emerald-100"
            : "border-white/10 bg-white/5 text-zinc-300 hover:border-white/20 hover:bg-white/10 hover:text-white",
        )}
      >
        <span>{menu.label}</span>
        {menu.children?.length ? (
          <span className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
            {menu.children.length}
          </span>
        ) : null}
      </button>
      {menu.children?.length
        ? menu.children.map((child) => renderMenu(child, currentPath, onNavigate, depth + 1))
        : null}
    </div>
  );
}

function renderSection(
  section: SectionNode,
  currentPath: string,
  onNavigate: (href: string) => void,
) {
  return (
    <section key={section.id} className="space-y-3">
      <header className="flex items-center justify-between">
        <h2 className="text-xs font-semibold uppercase tracking-[0.24em] text-zinc-500">
          {section.label}
        </h2>
      </header>
      <div className="space-y-2">
        {section.menus.map((menu) => renderMenu(menu, currentPath, onNavigate))}
      </div>
    </section>
  );
}

function renderDomain(domain: DomainNode, currentPath: string, onNavigate: (href: string) => void) {
  const active = matchPattern(currentPath, domain.href) || currentPath.startsWith(`${domain.href}/`);
  return (
    <button
      key={domain.id}
      type="button"
      onClick={() => onNavigate(domain.href)}
      className={clsx(
        "rounded-md px-3 py-2 text-sm transition",
        active
          ? "bg-white text-zinc-950"
          : "bg-white/5 text-zinc-300 hover:bg-white/10 hover:text-white",
      )}
    >
      {domain.label}
    </button>
  );
}

function renderAction(action: TopbarAction, isDark: boolean) {
  return (
    <button
      key={action.id}
      type="button"
      title={action.title ?? action.label}
      disabled={action.disabled}
      onClick={() => action.onSelect?.()}
      className={clsx(
        "inline-flex items-center gap-2 rounded-md border px-3 py-2 text-sm transition",
        action.tone === "accent" && "border-emerald-400/60 bg-emerald-500/10 text-emerald-100",
        action.tone === "danger" && "border-red-400/40 bg-red-500/10 text-red-100",
        !action.tone &&
          "border-white/10 bg-white/5 text-zinc-200 hover:border-white/20 hover:bg-white/10",
        action.disabled && "cursor-not-allowed opacity-50",
      )}
    >
      {iconForAction(action, isDark)}
      <span className="hidden lg:inline">{action.label}</span>
    </button>
  );
}

function renderRightPanel(rightPanel: RightPanelSlot | null | undefined) {
  if (!rightPanel) {
    return null;
  }

  return (
    <aside className="hidden min-w-[18rem] max-w-[20rem] flex-col rounded-xl border border-white/10 bg-zinc-950/80 p-4 xl:flex">
      <h2 className="text-sm font-semibold text-white">{rightPanel.title}</h2>
      <div className="mt-4 text-sm text-zinc-300">{rightPanel.content}</div>
    </aside>
  );
}

export interface AdminWorkbenchProps {
  provider: AdminProvider;
  context: AdminShellContext;
  children: ReactNode;
}

export function AdminWorkbench({ provider, context, children }: AdminWorkbenchProps) {
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        context.onFocusSearch();
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [context]);

  const shell: AdminShellState = provider.getShellState(context);

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto flex min-h-screen max-w-[1800px] flex-col gap-4 px-4 py-4 sm:px-6">
        <header className="rounded-xl border border-white/10 bg-zinc-900/90 px-4 py-4 shadow-2xl shadow-black/30">
          <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
            <div className="flex flex-col gap-4">
              <div>
                <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">Addzero Admin</p>
                <div className="mt-2">
                  <h1 className="text-xl font-semibold text-white">{shell.brandTitle}</h1>
                  <p className="text-sm text-zinc-400">{shell.brandDetail}</p>
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                {shell.domains.map((domain) =>
                  renderDomain(domain, context.currentPath, context.onNavigate),
                )}
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              {shell.topbarActions.map((action) => renderAction(action, context.isDark))}
            </div>
          </div>
        </header>

        <div className="grid flex-1 gap-4 xl:grid-cols-[18rem_minmax(0,1fr)_20rem]">
          <aside className="space-y-4 rounded-xl border border-white/10 bg-zinc-900/80 p-4">
            {shell.sections.map((section) =>
              renderSection(section, context.currentPath, context.onNavigate),
            )}
          </aside>
          <main className="min-w-0 rounded-xl border border-white/10 bg-zinc-900/80 p-4 md:p-6">
            {children}
          </main>
          {renderRightPanel(shell.rightPanel)}
        </div>
      </div>
    </div>
  );
}
