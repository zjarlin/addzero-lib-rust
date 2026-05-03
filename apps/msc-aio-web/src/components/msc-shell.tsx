"use client";

import { AdminWorkbench } from "@addzero/admin-shell";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import { api } from "@/lib/api";
import { adminProvider } from "@/lib/navigation";
import { Input, Button, Callout } from "./ui";

function focusSearchInput() {
  const target = document.querySelector<HTMLInputElement | HTMLTextAreaElement>(
    "[data-command-search='true']",
  );
  if (target) {
    target.focus();
    if ("select" in target && typeof target.select === "function") {
      target.select();
    }
  }
}

export function useThemePreference() {
  const [isDark, setIsDark] = useState(true);

  useEffect(() => {
    const stored = window.localStorage.getItem("msc-aio-theme");
    if (stored === "light") {
      setIsDark(false);
    }
  }, []);

  useEffect(() => {
    window.localStorage.setItem("msc-aio-theme", isDark ? "dark" : "light");
    document.documentElement.dataset.theme = isDark ? "dark" : "light";
  }, [isDark]);

  return {
    isDark,
    toggle: () => setIsDark((value) => !value),
  };
}

export function useSessionState() {
  const [ready, setReady] = useState(false);
  const [username, setUsername] = useState("");
  const [authenticated, setAuthenticated] = useState(false);
  const [permissions, setPermissions] = useState<string[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setError(null);
    try {
      const session = await api.getSession();
      setAuthenticated(session.authenticated);
      setUsername(session.username ?? "");
      if (session.authenticated) {
        const codes = await api.getPermissions();
        setPermissions(codes.length ? codes : null);
      } else {
        setPermissions(null);
      }
    } catch (nextError) {
      setAuthenticated(false);
      setUsername("");
      setPermissions(null);
      setError(nextError instanceof Error ? nextError.message : "会话恢复失败");
    } finally {
      setReady(true);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return {
    ready,
    username,
    authenticated,
    permissions,
    error,
    refresh,
    async login(username: string, password: string) {
      await api.login({ username, password });
      await refresh();
    },
    async logout() {
      await api.logout();
      await refresh();
    },
  };
}

export function MscProtectedShell({ children }: { children: ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const theme = useThemePreference();
  const session = useSessionState();

  useEffect(() => {
    if (session.ready && !session.authenticated) {
      router.replace("/login");
    }
  }, [router, session.authenticated, session.ready]);

  const context = useMemo(
    () => ({
      currentPath: pathname,
      isDark: theme.isDark,
      username: session.username,
      permissions: session.permissions,
      onNavigate: (href: string) => router.push(href),
      onLogout: () => {
        void session.logout().then(() => router.replace("/login"));
      },
      onToggleTheme: theme.toggle,
      onFocusSearch: focusSearchInput,
    }),
    [pathname, router, session, theme],
  );

  if (!session.ready) {
    return <div className="flex min-h-screen items-center justify-center text-zinc-400">正在恢复登录态…</div>;
  }

  if (!session.authenticated) {
    return <div className="flex min-h-screen items-center justify-center text-zinc-400">正在跳转登录页…</div>;
  }

  return <AdminWorkbench provider={adminProvider} context={context}>{children}</AdminWorkbench>;
}

export function LoginPage() {
  const router = useRouter();
  const session = useSessionState();
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("admin");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (session.ready && session.authenticated) {
      router.replace("/dashboard");
    }
  }, [router, session.authenticated, session.ready]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-[radial-gradient(circle_at_top_left,_rgba(16,185,129,0.16),_transparent_32%),radial-gradient(circle_at_bottom_right,_rgba(56,189,248,0.14),_transparent_28%),#050816] p-6">
      <div className="w-full max-w-md rounded-2xl border border-white/10 bg-zinc-950/85 p-8 shadow-2xl shadow-black/50">
        <p className="text-xs uppercase tracking-[0.32em] text-emerald-300/70">MSC_AIO</p>
        <h1 className="mt-4 text-3xl font-semibold text-white">登录工作台</h1>
        <p className="mt-3 text-sm text-zinc-400">
          浏览器端只承担管理面，认证仍由现有 Axum session cookie 负责。
        </p>

        {error ? <Callout tone="danger">{error}</Callout> : null}
        {session.error ? <Callout tone="danger">{session.error}</Callout> : null}

        <form
          className="mt-6 space-y-4"
          onSubmit={async (event) => {
            event.preventDefault();
            setPending(true);
            setError(null);
            try {
              await session.login(username, password);
              router.replace("/dashboard");
            } catch (nextError) {
              setError(nextError instanceof Error ? nextError.message : "登录失败");
            } finally {
              setPending(false);
            }
          }}
        >
          <div className="space-y-2">
            <label className="text-xs uppercase tracking-[0.18em] text-zinc-500">用户名</label>
            <Input value={username} onChange={(event) => setUsername(event.target.value)} />
          </div>
          <div className="space-y-2">
            <label className="text-xs uppercase tracking-[0.18em] text-zinc-500">密码</label>
            <Input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </div>
          <Button tone="accent" type="submit" disabled={pending} className="w-full justify-center">
            {pending ? "登录中…" : "进入工作台"}
          </Button>
        </form>
      </div>
    </div>
  );
}
