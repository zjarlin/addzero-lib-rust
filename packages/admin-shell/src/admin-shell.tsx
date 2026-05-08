"use client";

import { Bell, LogOut, MoonStar, Search, SunMedium } from "lucide-react";
import { useEffect, type ReactNode } from "react";
import {
    Badge,
    Button,
    ScrollArea,
    Separator,
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
    cn,
} from "@addzero/ui";

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
            <Button
                type="button"
                variant={active ? "secondary" : "ghost"}
                size="sm"
                onClick={() => onNavigate(menu.href)}
                className={cn(
                    "h-auto w-full justify-between whitespace-normal px-2 py-2 text-left text-xs sm:px-3 sm:text-sm",
                    depth > 0 && "ml-3 w-[calc(100%-0.75rem)] sm:ml-4 sm:w-[calc(100%-1rem)]",
                    active && "shadow-sm",
                )}
            >
                <span>{menu.label}</span>
                {menu.children?.length ? (
                    <Badge
                        variant="secondary"
                        className="min-w-5 justify-center px-1.5 text-[10px]"
                    >
                        {menu.children.length}
                    </Badge>
                ) : null}
            </Button>
            {menu.children?.length
                ? menu.children.map((child) =>
                      renderMenu(child, currentPath, onNavigate, depth + 1),
                  )
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
            <header className="flex items-center justify-between px-1">
                <h2 className="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground">
                    {section.label}
                </h2>
            </header>
            <div className="space-y-2">
                {section.menus.map((menu) =>
                    renderMenu(menu, currentPath, onNavigate),
                )}
            </div>
        </section>
    );
}

function renderDomain(
    domain: DomainNode,
    currentPath: string,
    onNavigate: (href: string) => void,
) {
    const activePatterns = domain.activePatterns ?? [domain.href];
    const active = activePatterns.some((pattern) => {
        if (pattern === "/" || pattern.includes(":")) {
            return matchPattern(currentPath, pattern);
        }
        const cleanPath = currentPath.split("?")[0].replace(/\/+$/, "") || "/";
        const cleanPattern = pattern.replace(/\/+$/, "") || "/";
        return (
            matchPattern(cleanPath, cleanPattern) ||
            cleanPath.startsWith(`${cleanPattern}/`)
        );
    });
    return (
        <Button
            key={domain.id}
            type="button"
            onClick={() => onNavigate(domain.href)}
            size="sm"
            variant={active ? "default" : "outline"}
            className="rounded-full"
        >
            {domain.label}
        </Button>
    );
}

function handlerForAction(
    action: TopbarAction,
    context: AdminShellContext,
) {
    if (action.onSelect) {
        return () => action.onSelect?.();
    }

    switch (action.id) {
        case "theme-toggle":
            return () => context.onToggleTheme();
        case "focus-search":
            return () => context.onFocusSearch();
        case "logout":
            return () => {
                void context.onLogout();
            };
        default:
            return undefined;
    }
}

function renderAction(
    action: TopbarAction,
    isDark: boolean,
    context: AdminShellContext,
) {
    const variant =
        action.tone === "danger"
            ? "destructive"
            : action.tone === "accent"
              ? "default"
              : "outline";
    const label = action.title ?? action.label;
    const onClick = handlerForAction(action, context);

    return (
        <Tooltip key={action.id}>
            <TooltipTrigger asChild>
                <Button
                    type="button"
                    size="sm"
                    variant={variant}
                    aria-label={label}
                    disabled={action.disabled}
                    onClick={onClick}
                    className="gap-2"
                >
                    {iconForAction(action, isDark)}
                    <span className="hidden xl:inline">{action.label}</span>
                </Button>
            </TooltipTrigger>
            <TooltipContent>{label}</TooltipContent>
        </Tooltip>
    );
}

function renderRightPanel(rightPanel: RightPanelSlot | null | undefined) {
    if (!rightPanel) {
        return null;
    }

    return (
        <aside className="hidden min-h-0 w-80 shrink-0 flex-col border-l bg-card xl:flex">
            <div className="border-b px-4 py-3">
                <h2 className="text-sm font-semibold">{rightPanel.title}</h2>
            </div>
            <ScrollArea className="min-h-0 flex-1 px-4 py-4">
                <div className="text-sm text-muted-foreground">
                    {rightPanel.content}
                </div>
            </ScrollArea>
        </aside>
    );
}

export interface AdminWorkbenchProps {
    provider: AdminProvider;
    context: AdminShellContext;
    children: ReactNode;
}

export function AdminWorkbench({
    provider,
    context,
    children,
}: AdminWorkbenchProps) {
    useEffect(() => {
        const handler = (event: KeyboardEvent) => {
            if (
                (event.metaKey || event.ctrlKey) &&
                event.key.toLowerCase() === "k"
            ) {
                event.preventDefault();
                context.onFocusSearch();
            }
        };

        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    }, [context]);

    const shell: AdminShellState = provider.getShellState(context);
    const hasRightPanel = Boolean(shell.rightPanel);

    return (
        <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
            <header className="flex min-h-14 shrink-0 items-center justify-between gap-2 border-b bg-card px-3 py-2 sm:px-4">
                <div className="flex min-w-0 flex-1 flex-wrap gap-2">
                    {shell.domains.map((domain) =>
                        renderDomain(
                            domain,
                            context.currentPath,
                            context.onNavigate,
                        ),
                    )}
                </div>
                <TooltipProvider delayDuration={120}>
                    <div className="flex shrink-0 flex-wrap justify-end gap-2">
                        {shell.topbarContentEnd ? (
                            <div className="flex shrink-0 items-center">
                                {shell.topbarContentEnd}
                            </div>
                        ) : null}
                        {shell.topbarActions.map((action) =>
                            renderAction(action, context.isDark, context),
                        )}
                    </div>
                </TooltipProvider>
            </header>

            <div
                className={cn(
                    "grid min-h-0 flex-1 grid-cols-[11rem_minmax(0,1fr)] sm:grid-cols-[12.5rem_minmax(0,1fr)] md:grid-cols-[15rem_minmax(0,1fr)]",
                    hasRightPanel
                        ? "lg:grid-cols-[18rem_minmax(0,1fr)] xl:grid-cols-[18rem_minmax(0,1fr)_20rem]"
                        : "lg:grid-cols-[18rem_minmax(0,1fr)]",
                )}
            >
                <aside className="flex min-h-0 flex-col border-r bg-card">
                    <div className="border-b px-3 py-3 sm:px-4">
                        <h2 className="text-sm font-semibold">
                            {shell.navigationTitle ?? "Navigation"}
                        </h2>
                        <p className="mt-1 text-xs text-muted-foreground">
                            {shell.navigationDetail ?? "双轴上下文树"}
                        </p>
                    </div>
                    <ScrollArea className="min-h-0 flex-1 px-3 py-3 sm:px-4 sm:py-4">
                        <div className="space-y-4 sm:space-y-5">
                            {shell.sections.map((section, index) => (
                                <div key={section.id} className="space-y-5">
                                    {index > 0 ? <Separator /> : null}
                                    {renderSection(
                                        section,
                                        context.currentPath,
                                        context.onNavigate,
                                    )}
                                </div>
                            ))}
                        </div>
                    </ScrollArea>
                </aside>

                <main className="min-h-0 min-w-0 overflow-y-auto">
                    {children}
                </main>
                {renderRightPanel(shell.rightPanel)}
            </div>
        </div>
    );
}
