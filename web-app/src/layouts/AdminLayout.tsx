import { useEffect, useState } from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { AdminWorkbench } from "@addzero/admin-shell";
import type { AdminShellContext } from "@addzero/admin-shell";
import { useAdminProvider } from "../hooks/useAdminProvider";
import { useAuthStore } from "../stores/auth";
import { useThemeStore } from "../stores/theme";

export default function AdminLayout() {
    const location = useLocation();
    const navigate = useNavigate();
    const { provider, loading } = useAdminProvider();
    const username = useAuthStore((s) => s.username) ?? "";
    const logout = useAuthStore((s) => s.logout);
    const { theme, toggle: toggleTheme } = useThemeStore();
    const isDark = theme === "dark";
    const [searchOpen, setSearchOpen] = useState(false);

    // Ctrl+K global shortcut
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
            <div className="flex min-h-screen items-center justify-center bg-zinc-950 text-zinc-400">
                <p className="animate-pulse">Loading admin shell…</p>
            </div>
        );
    }

    return (
        <>
            <AdminWorkbench provider={provider} context={context}>
                <Outlet />
            </AdminWorkbench>

            {/* Command Palette */}
            {searchOpen && (
                <div
                    className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
                    onClick={() => setSearchOpen(false)}
                >
                    <div
                        className="w-full max-w-lg rounded-xl border border-white/10 bg-zinc-900/95 shadow-2xl shadow-black/50 backdrop-blur"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="border-b border-white/5 px-4 py-3">
                            <input
                                type="text"
                                placeholder="输入命令搜索..."
                                autoFocus
                                className="w-full bg-transparent text-sm text-white placeholder-zinc-500 outline-none"
                                onKeyDown={(e) => {
                                    if (e.key === "Escape")
                                        setSearchOpen(false);
                                }}
                            />
                        </div>
                        <div className="p-2 text-sm">
                            {[
                                { label: "Dashboard", href: "/" },
                                { label: "脚本控制台", href: "/console" },
                            ].map((item) => (
                                <button
                                    key={item.href}
                                    type="button"
                                    className="flex w-full items-center rounded-lg px-3 py-2 text-left text-zinc-300 transition hover:bg-white/5"
                                    onClick={() => {
                                        navigate(item.href);
                                        setSearchOpen(false);
                                    }}
                                >
                                    {item.label}
                                </button>
                            ))}
                        </div>
                        <div className="border-t border-white/5 px-4 py-2 text-xs text-zinc-600">
                            <kbd className="rounded bg-white/5 px-1.5 py-0.5">
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
