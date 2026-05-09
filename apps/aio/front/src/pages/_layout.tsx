import { useEffect, useMemo, useState } from "react";
import { Outlet, useLocation, Navigate } from "react-router-dom";
import {
  getApiBaseUrl,
  isDesktopRuntime,
  type BootstrapStatusDto,
} from "@az/api-client";
import { useAuthStore } from "../stores/auth";
import AdminLayout from "../layouts/AdminLayout";

export default function RootLayout() {
  const { username, loading, checkSession } = useAuthStore();
  const location = useLocation();
  const baseUrl = useMemo(() => getApiBaseUrl(), []);
  const desktopMode = useMemo(() => isDesktopRuntime(), []);
  const [bootstrapLoading, setBootstrapLoading] = useState(desktopMode);
  const [bootstrapStatus, setBootstrapStatus] = useState<BootstrapStatusDto | null>(null);

  useEffect(() => {
    let cancelled = false;

    void checkSession();

    if (!desktopMode) {
      setBootstrapLoading(false);
      return () => {
        cancelled = true;
      };
    }

    async function loadBootstrapStatus() {
      setBootstrapLoading(true);
      try {
        const response = await fetch(`${baseUrl}/api/bootstrap/status`);
        if (!response.ok) {
          const text = await response.text();
          throw new Error(text || `HTTP ${response.status}`);
        }
        const payload = (await response.json()) as BootstrapStatusDto;
        if (!cancelled) {
          setBootstrapStatus(payload);
        }
      } catch {
        if (!cancelled) {
          setBootstrapStatus({
            desktop_mode: true,
            setup_required: true,
            database_configured: false,
            database_reachable: false,
            config_path: "~/.config/aio/aio.env",
            message: "读取桌面初始化状态失败，请先配置 PostgreSQL。",
          });
        }
      } finally {
        if (!cancelled) {
          setBootstrapLoading(false);
        }
      }
    }

    void loadBootstrapStatus();
    return () => {
      cancelled = true;
    };
  }, [baseUrl, checkSession, desktopMode]);

  if (loading || bootstrapLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background text-muted-foreground">
        <p className="animate-pulse">Loading…</p>
      </div>
    );
  }

  const setupRequired = desktopMode && (bootstrapStatus?.setup_required ?? false);

  if (location.pathname === "/setup") {
    if (!desktopMode) {
      return <Navigate to="/login" replace />;
    }
    if (!setupRequired) {
      return <Navigate to={username ? "/" : "/login"} replace />;
    }
    return <Outlet />;
  }

  if (setupRequired) {
    return <Navigate to="/setup" replace />;
  }

  // Login page — no auth required
  if (location.pathname === "/login") {
    return <Outlet />;
  }

  // Protected routes
  if (!username) {
    return <Navigate to="/login" replace />;
  }

  return (
    <AdminLayout>
      <Outlet />
    </AdminLayout>
  );
}
