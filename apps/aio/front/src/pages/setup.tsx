import { useEffect, useMemo, useState } from "react";
import { Database, HardDrive, Loader2, Save, ShieldCheck } from "lucide-react";
import {
    getApiBaseUrl,
    isDesktopRuntime,
    type BootstrapDatabaseSaveResultDto,
    type BootstrapStatusDto,
} from "@addzero/api-client";
import {
    Button,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
    Input,
} from "@addzero/ui";

export default function SetupPage() {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const desktopMode = useMemo(() => isDesktopRuntime(), []);
    const [status, setStatus] = useState<BootstrapStatusDto | null>(null);
    const [databaseUrl, setDatabaseUrl] = useState("");
    const [loading, setLoading] = useState(true);
    const [savingMode, setSavingMode] = useState<"postgres" | "sqlite" | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const saving = savingMode !== null;

    useEffect(() => {
        if (!desktopMode) {
            window.location.replace("/login");
            return;
        }

        let cancelled = false;

        async function loadStatus() {
            setLoading(true);
            setError(null);
            try {
                const response = await fetch(`${baseUrl}/api/bootstrap/status`);
                if (!response.ok) {
                    const text = await response.text();
                    throw new Error(text || `HTTP ${response.status}`);
                }
                const payload = (await response.json()) as BootstrapStatusDto;
                if (cancelled) {
                    return;
                }
                setStatus(payload);
                setMessage(payload.message);
                if (!payload.setup_required) {
                    window.location.replace("/login");
                }
            } catch (loadError) {
                if (!cancelled) {
                    setError(
                        loadError instanceof Error
                            ? loadError.message
                            : "读取桌面初始化状态失败。",
                    );
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void loadStatus();
        return () => {
            cancelled = true;
        };
    }, [baseUrl, desktopMode]);

    async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
        event.preventDefault();
        setSavingMode("postgres");
        setError(null);
        setMessage(null);
        try {
            const response = await fetch(`${baseUrl}/api/bootstrap/database`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    database_url: databaseUrl.trim(),
                }),
            });
            if (!response.ok) {
                const text = await response.text();
                throw new Error(text || `HTTP ${response.status}`);
            }
            const payload = (await response.json()) as BootstrapDatabaseSaveResultDto;
            setMessage(payload.message);
            window.location.replace("/login");
        } catch (submitError) {
            setError(
                submitError instanceof Error
                    ? submitError.message
                    : "保存 PostgreSQL 地址失败。",
            );
        } finally {
            setSavingMode(null);
        }
    }

    async function handleUseLocalSqlite() {
        setSavingMode("sqlite");
        setError(null);
        setMessage(null);
        try {
            const response = await fetch(`${baseUrl}/api/bootstrap/database/sqlite-local`, {
                method: "POST",
            });
            if (!response.ok) {
                const text = await response.text();
                throw new Error(text || `HTTP ${response.status}`);
            }
            const payload = (await response.json()) as BootstrapDatabaseSaveResultDto;
            setMessage(payload.message);
            window.location.replace("/login");
        } catch (submitError) {
            setError(
                submitError instanceof Error
                    ? submitError.message
                    : "切换本机 SQLite 失败。",
            );
        } finally {
            setSavingMode(null);
        }
    }

    return (
        <div className="grid min-h-screen bg-[#f7f4ec] lg:grid-cols-[1.15fr_0.85fr]">
            <section className="hidden border-r bg-[#171a17] text-stone-100 lg:flex lg:flex-col lg:justify-between lg:p-10">
                <div>
                    <div className="text-xs font-semibold uppercase tracking-[0.22em] text-stone-400">
                        AIO Desktop
                    </div>
                    <h1 className="mt-4 max-w-xl text-4xl font-semibold tracking-tight">
                        纯桌面本地工作台，首次启动先接 PostgreSQL，或者直接落本机 SQLite
                    </h1>
                    <p className="mt-4 max-w-lg text-sm leading-7 text-stone-300">
                        桌面壳子、前端资源和本地 API 都在本机运行。
                        如果你已经有正式库，就先接 PostgreSQL；如果现在只想先跑起来，
                        可以直接跳过，系统会把数据落到本机内嵌 SQLite，后续再切回 PostgreSQL。
                    </p>
                </div>

                <div className="grid gap-3">
                    <DesktopSignal
                        label="启动方式"
                        value="Tauri + 本地 Axum"
                    />
                    <DesktopSignal
                        label="配置落点"
                        value={status?.config_path || "~/.config/aio/aio.env"}
                    />
                    <DesktopSignal
                        label="持久化策略"
                        value="PostgreSQL 优先 / SQLite 首启兜底"
                    />
                </div>
            </section>

            <section className="flex items-center justify-center p-6 sm:p-10">
                <form
                    onSubmit={handleSubmit}
                    className="w-full max-w-xl"
                >
                    <Card className="border-stone-300 shadow-sm">
                        <CardHeader className="space-y-3">
                            <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                                <Database className="h-3.5 w-3.5" />
                                Desktop Bootstrap
                            </div>
                            <CardTitle className="text-2xl tracking-tight">
                                配置数据库
                            </CardTitle>
                            <CardDescription className="text-sm leading-6">
                                你可以先接 PostgreSQL；如果这一步跳过，系统会直接启用本机内嵌 SQLite。
                                两种方式都会把地址写进本机配置文件，后续桌面端直接复用。
                            </CardDescription>
                        </CardHeader>

                        <CardContent className="space-y-5">
                            {loading ? (
                                <div className="flex items-center justify-center py-12 text-muted-foreground">
                                    <Loader2 className="mr-2 h-5 w-5 animate-spin" />
                                    正在检查本机初始化状态…
                                </div>
                            ) : (
                                <>
                                    <label className="block">
                                        <span className="mb-2 flex items-center gap-2 text-sm text-muted-foreground">
                                            <Database className="h-4 w-4" />
                                            PostgreSQL URL
                                        </span>
                                        <Input
                                            value={databaseUrl}
                                            onChange={(event) => setDatabaseUrl(event.target.value)}
                                            placeholder="postgresql://user:password@host:5432/database?sslmode=require"
                                            autoFocus
                                        />
                                    </label>

                                    <div className="rounded-2xl border border-dashed border-stone-300 bg-[#fcfaf4] px-4 py-3 text-sm text-muted-foreground">
                                        <div className="font-medium text-foreground">保存位置</div>
                                        <div className="mt-1 break-all">
                                            {status?.config_path || "~/.config/aio/aio.env"}
                                        </div>
                                        <div className="mt-1 text-xs text-muted-foreground">
                                            跳过 PostgreSQL 时会自动创建 `~/.config/aio/aio.sqlite3`
                                        </div>
                                        <div className="mt-3 flex items-start gap-2">
                                            <ShieldCheck className="mt-0.5 h-4 w-4 text-emerald-600" />
                                            <span>
                                                先测试连接，再落本机配置；不把数据库地址写进页面状态或业务表。
                                            </span>
                                        </div>
                                    </div>

                                    {message ? (
                                        <div className="rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                                            {message}
                                        </div>
                                    ) : null}

                                    {error ? (
                                        <div className="rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
                                            {error}
                                        </div>
                                    ) : null}

                                    <div className="grid gap-3 sm:grid-cols-2">
                                        <Button
                                            type="submit"
                                            className="w-full rounded-2xl"
                                            disabled={saving || !databaseUrl.trim()}
                                        >
                                            {savingMode === "postgres" ? (
                                                <>
                                                    <Loader2 className="h-4 w-4 animate-spin" />
                                                    测试连接并保存
                                                </>
                                            ) : (
                                                <>
                                                    <Save className="h-4 w-4" />
                                                    测试连接并保存
                                                </>
                                            )}
                                        </Button>

                                        <Button
                                            type="button"
                                            variant="outline"
                                            className="w-full rounded-2xl border-stone-300 bg-[#fcfaf4]"
                                            disabled={saving}
                                            onClick={() => void handleUseLocalSqlite()}
                                        >
                                            {savingMode === "sqlite" ? (
                                                <>
                                                    <Loader2 className="h-4 w-4 animate-spin" />
                                                    正在启用本机 SQLite
                                                </>
                                            ) : (
                                                <>
                                                    <HardDrive className="h-4 w-4" />
                                                    跳过并使用本机 SQLite
                                                </>
                                            )}
                                        </Button>
                                    </div>
                                </>
                            )}
                        </CardContent>
                    </Card>
                </form>
            </section>
        </div>
    );
}

function DesktopSignal({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-2xl border border-stone-800 bg-stone-900/60 px-4 py-3">
            <div className="text-xs uppercase tracking-[0.18em] text-stone-400">
                {label}
            </div>
            <div className="mt-1 text-sm font-medium text-stone-100">
                {value}
            </div>
        </div>
    );
}
