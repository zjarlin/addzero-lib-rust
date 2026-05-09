import { useState } from "react";
import { Navigate } from "react-router-dom";
import { Loader2, Lock, User } from "lucide-react";
import { Button, Card, CardContent, CardHeader, CardTitle, Input } from "@az/ui";
import { useAuthStore } from "../stores/auth";

export default function LoginPage() {
    const username = useAuthStore((s) => s.username);
    const login = useAuthStore((s) => s.login);
    const loading = useAuthStore((s) => s.loading);
    const [form, setForm] = useState({ username: "admin", password: "" });
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    if (loading) {
        return (
            <div className="flex min-h-screen items-center justify-center bg-background text-muted-foreground">
                <Loader2 className="h-5 w-5 animate-spin" />
            </div>
        );
    }

    if (username) {
        return <Navigate to="/" replace />;
    }

    async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
        event.preventDefault();
        setSubmitting(true);
        setError(null);
        try {
            await login(form.username, form.password);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Login failed");
        } finally {
            setSubmitting(false);
        }
    }

    return (
        <div className="grid min-h-screen bg-background lg:grid-cols-[1.3fr_0.9fr]">
            <section className="hidden border-r bg-muted/30 lg:flex lg:flex-col lg:justify-between lg:p-10">
                <div>
                    <div className="text-xs font-medium uppercase tracking-[0.22em] text-muted-foreground">
                        AIO Platform
                    </div>
                    <h1 className="mt-4 max-w-xl text-4xl font-semibold tracking-tight">
                        Unified shell for script runtime, AI orchestration, and plugin expansion
                    </h1>
                    <p className="mt-4 max-w-lg text-sm text-muted-foreground">
                        这一步先保证平台外壳、入口和工作台可用，后面再逐个落脚本引擎、
                        插件运行层、AI 编排和 CLI 产出能力。
                    </p>
                </div>

                <div className="grid gap-3">
                    <Signal label="Runtime" value="Rhai first, multi-engine later" />
                    <Signal label="Plugin model" value="WASM / WIT planned" />
                    <Signal label="Shell target" value="Web + Tauri shared workbench" />
                </div>
            </section>

            <section className="flex items-center justify-center p-6 sm:p-10">
                <form
                    onSubmit={handleSubmit}
                    className="w-full max-w-md"
                >
                    <Card className="shadow-sm">
                        <CardHeader>
                            <div className="text-sm font-medium text-muted-foreground">
                                登录 AIO 工作台
                            </div>
                            <CardTitle className="text-2xl">Admin session</CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <label className="block">
                                <span className="mb-2 flex items-center gap-2 text-sm text-muted-foreground">
                                    <User className="h-4 w-4" />
                                    Username
                                </span>
                                <Input
                                    value={form.username}
                                    onChange={(event) =>
                                        setForm((current) => ({
                                            ...current,
                                            username: event.target.value,
                                        }))
                                    }
                                    autoComplete="username"
                                />
                            </label>

                            <label className="block">
                                <span className="mb-2 flex items-center gap-2 text-sm text-muted-foreground">
                                    <Lock className="h-4 w-4" />
                                    Password
                                </span>
                                <Input
                                    type="password"
                                    value={form.password}
                                    onChange={(event) =>
                                        setForm((current) => ({
                                            ...current,
                                            password: event.target.value,
                                        }))
                                    }
                                    autoComplete="current-password"
                                />
                            </label>

                            {error ? (
                                <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                                    {error}
                                </div>
                            ) : null}

                            <Button type="submit" disabled={submitting} className="w-full">
                                {submitting ? (
                                    <>
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                        Signing in
                                    </>
                                ) : (
                                    "Sign in"
                                )}
                            </Button>
                        </CardContent>
                    </Card>
                </form>
            </section>
        </div>
    );
}

function Signal({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-md border bg-background px-4 py-3">
            <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                {label}
            </div>
            <div className="mt-1 text-sm font-medium">{value}</div>
        </div>
    );
}
