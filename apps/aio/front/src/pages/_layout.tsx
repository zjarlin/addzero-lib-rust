import { useEffect } from "react";
import { Outlet, useLocation, Navigate } from "react-router-dom";
import { useAuthStore } from "../stores/auth";
import AdminLayout from "../layouts/AdminLayout";

export default function RootLayout() {
  const { username, loading, checkSession } = useAuthStore();
  const location = useLocation();

  useEffect(() => {
    checkSession();
  }, [checkSession]);

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background text-muted-foreground">
        <p className="animate-pulse">Loading…</p>
      </div>
    );
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
