import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { AdminWorkbench } from "@addzero/admin-shell";
import type { AdminShellContext } from "@addzero/admin-shell";
import { useAdminProvider } from "../hooks/useAdminProvider";
import { useAuthStore } from "../stores/auth";
import { useThemeStore } from "../stores/theme";

export default function AdminLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    const location = useLocation();
    const navigate = useNavigate();
    const { provider, loading } = useAdminProvider();
    const username = useAuthStore((s) => s.username) ?? "";
    const logout = useAuthStore((s) => s.logout);
    const { theme, toggle: toggleTheme } = useThemeStore();
    const isDark = theme === "dark";
    const [searchOpen, setSearchOpen] = useState(false);

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
                e.preventDefault();
                setSearchOpen((v) => !v);
            }
        };
        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    }, []);

    const context: AdminShellContext = {
        currentPath: location.pathname,
        isDark,
        username,
        permissions: null,
        onNavigate: (href: string) => {
            navigate(href);
            setSearchOpen(false);
        },
        onLogout: async () => {
            await logout();
            navigate("/login");
        },
        onToggleTheme: toggleTheme,
        onFocusSearch: () => setSearchOpen(true),
    };

    if (loading) {
        return (
            <div className="flex min-h-screen items-center justify-center bg-background text-muted-foreground">
                <p className="animate-pulse">Loading admin shell…</p>
            </div>
        );
    }

    return (
        <>
            <AdminWorkbench provider={provider} context={context}>
                {children}
            </AdminWorkbench>

            {searchOpen && (
                <div
                    className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
                    onClick={() => setSearchOpen(false)}
                >
                    <div
                        className="w-full max-w-lg rounded-xl border bg-popover shadow-2xl"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="border-b px-4 py-3">
                            <input
                                type="text"
                                placeholder="输入命令搜索..."
                                autoFocus
                                className="w-full bg-transparent text-sm outline-none"
                                onKeyDown={(e) => {
                                    if (e.key === "Escape")
                                        setSearchOpen(false);
                                }}
                            />
                        </div>
                        <div className="p-2 text-sm">
                            {[
                                { label: "平台总览", href: "/" },
                                { label: "脚本控制台", href: "/console" },
                                { label: "环境与配置", href: "/env" },
                                { label: "插件与技能", href: "/skills" },
                                { label: "知识与记忆", href: "/knowledge" },
                                { label: "存储与资源", href: "/storage" },
                                { label: "WASM 插件市场", href: "/market" },
                                { label: "系统管理", href: "/system" },
                            ].map((item) => (
                                <button
                                    key={item.href}
                                    type="button"
                                    className="flex w-full items-center rounded-lg px-3 py-2 text-left transition hover:bg-accent"
                                    onClick={() => {
                                        navigate(item.href);
                                        setSearchOpen(false);
                                    }}
                                >
                                    {item.label}
                                </button>
                            ))}
                        </div>
                        <div className="border-t px-4 py-2 text-xs text-muted-foreground">
                            <kbd className="rounded bg-muted px-1.5 py-0.5">
                                Esc
                            </kbd>{" "}
                            关闭
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
