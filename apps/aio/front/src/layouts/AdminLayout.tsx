import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { AdminWorkbench } from "@addzero/admin-shell";
import type { AdminShellContext } from "@addzero/admin-shell";
import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandShortcut,
} from "@addzero/ui";
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
    const { provider } = useAdminProvider();
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

    return (
        <>
            <AdminWorkbench provider={provider} context={context}>
                {children}
            </AdminWorkbench>

            <CommandDialog open={searchOpen} onOpenChange={setSearchOpen}>
                <CommandInput placeholder="输入命令或页面名称..." />
                <CommandList>
                    <CommandEmpty>没有匹配项</CommandEmpty>
                    <CommandGroup heading="场景">
                        {[
                            { label: "笔记", href: "/assets/notes", shortcut: "G N" },
                            { label: "安装包", href: "/assets/packages", shortcut: "G P" },
                            { label: "dotfiles", href: "/assets/dotfiles", shortcut: "G D" },
                            { label: "脚本控制台", href: "/console", shortcut: "G C" },
                            { label: "环境与配置", href: "/env", shortcut: "G E" },
                            { label: "WASM 插件市场", href: "/market", shortcut: "G W" },
                            { label: "系统管理", href: "/system", shortcut: "G Y" },
                        ].map((item) => (
                            <CommandItem
                                key={item.href}
                                value={`${item.label} ${item.href}`}
                                onSelect={() => {
                                    navigate(item.href);
                                    setSearchOpen(false);
                                }}
                            >
                                <span>{item.label}</span>
                                <CommandShortcut>{item.shortcut}</CommandShortcut>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                </CommandList>
            </CommandDialog>
        </>
    );
}
