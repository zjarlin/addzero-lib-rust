import { useEffect } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import AdminLayout from "./layouts/AdminLayout";
import Dashboard from "./pages/Dashboard";
import ScriptConsole from "./pages/ScriptConsole";
import Login from "./pages/Login";
import { useAuthStore } from "./stores/auth";

export default function App() {
    const { username, loading, checkSession } = useAuthStore();

    useEffect(() => {
        checkSession();
    }, [checkSession]);

    if (loading) {
        return (
            <div className="flex min-h-screen items-center justify-center bg-zinc-950 text-zinc-400">
                <p className="animate-pulse">Loading…</p>
            </div>
        );
    }

    return (
        <Routes>
            <Route
                path="/login"
                element={username ? <Navigate to="/" replace /> : <Login />}
            />
            <Route
                element={
                    username ? (
                        <AdminLayout />
                    ) : (
                        <Navigate to="/login" replace />
                    )
                }
            >
                <Route path="/" element={<Dashboard />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/console" element={<ScriptConsole />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
    );
}
