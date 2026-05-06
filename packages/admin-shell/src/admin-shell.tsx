"use client";

import { Bell, LogOut, MoonStar, Search, SunMedium } from "lucide-react";
import { useEffect, type ReactNode } from "react";
import {
    Badge,
    Button,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
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
                    "h-auto w-full justify-between whitespace-normal px-3 py-2 text-left",
                    depth > 0 && "ml-4 w-[calc(100%-1rem)]",
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
    const active =
        matchPattern(currentPath, domain.href) ||
        currentPath.startsWith(`${domain.href}/`);
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
        <Card className="hidden min-w-[18rem] max-w-[20rem] overflow-hidden xl:flex xl:flex-col">
            <CardHeader>
                <CardTitle className="text-sm">{rightPanel.title}</CardTitle>
            </CardHeader>
            <Separator />
            <CardContent className="p-0">
                <ScrollArea className="h-[calc(100vh-16rem)] px-4 py-4">
                    <div className="text-sm text-muted-foreground">
                        {rightPanel.content}
                    </div>
                </ScrollArea>
            </CardContent>
        </Card>
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

    return (
        <div className="min-h-screen bg-background text-foreground">
            <div className="mx-auto flex min-h-screen max-w-[1800px] flex-col gap-4 px-4 py-4 sm:px-6">
                <Card className="shadow-sm">
                    <CardContent className="space-y-4 p-4">
                        <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
                            <div className="min-w-0">
                                <div className="text-xs font-medium uppercase tracking-[0.24em] text-muted-foreground">
                                    {shell.brandTitle}
                                </div>
                                <CardDescription className="mt-1 max-w-3xl text-sm">
                                    {shell.brandDetail}
                                </CardDescription>
                            </div>
                            <TooltipProvider delayDuration={120}>
                                <div className="flex flex-wrap gap-2">
                                    {shell.topbarActions.map((action) =>
                                        renderAction(action, context.isDark, context),
                                    )}
                                </div>
                            </TooltipProvider>
                        </div>
                        <Separator />
                        <div className="flex flex-wrap gap-2">
                            {shell.domains.map((domain) =>
                                renderDomain(
                                    domain,
                                    context.currentPath,
                                    context.onNavigate,
                                ),
                            )}
                        </div>
                    </CardContent>
                </Card>

                <div className="grid flex-1 gap-4 xl:grid-cols-[18rem_minmax(0,1fr)_20rem]">
                    <Card className="overflow-hidden">
                        <CardHeader className="pb-3">
                            <CardTitle className="text-sm">Navigation</CardTitle>
                            <CardDescription className="text-xs">
                                双轴上下文树
                            </CardDescription>
                        </CardHeader>
                        <Separator />
                        <CardContent className="p-0">
                            <ScrollArea className="h-[calc(100vh-16rem)] px-4 py-4">
                                <div className="space-y-5">
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
                        </CardContent>
                    </Card>
                    <Card className="min-w-0">
                        <CardContent className="p-4 md:p-6">
                            {children}
                        </CardContent>
                    </Card>
                    {renderRightPanel(shell.rightPanel)}
                </div>
            </div>
        </div>
    );
}
