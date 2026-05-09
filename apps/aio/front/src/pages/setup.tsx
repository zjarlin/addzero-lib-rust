import { useEffect, useMemo, useState } from "react";
import { Database, HardDrive, Loader2, Save, ShieldCheck } from "lucide-react";
import {
    getApiBaseUrl,
    isDesktopRuntime,
    type BootstrapDatabaseSaveResultDto,
    type BootstrapPlatformSaveResultDto,
    type BootstrapStatusDto,
} from "@az/api-client";
import {
    Button,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
    Input,
} from "@az/ui";

export default function SetupPage() {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const desktopMode = useMemo(() => isDesktopRuntime(), []);
    const [status, setStatus] = useState<BootstrapStatusDto | null>(null);
    const [databaseUrl, setDatabaseUrl] = useState(
        "postgresql://postgres:postgres@host.docker.internal:5432/aio",
    );
    const [minioEndpoint, setMinioEndpoint] = useState("http://host.docker.internal:9000");
    const [minioAccessKey, setMinioAccessKey] = useState("minioadmin");
    const [minioSecretKey, setMinioSecretKey] = useState("minioadmin");
    const [minioRegion, setMinioRegion] = useState("us-east-1");
    const [enableMinio, setEnableMinio] = useState(true);
    const [loading, setLoading] = useState(true);
    const [savingMode, setSavingMode] = useState<"platform" | "postgres" | "sqlite" | null>(null);
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
        await savePlatformConfig();
    }

    async function savePlatformConfig() {
        setSavingMode("platform");
        setError(null);
        setMessage(null);
        try {
            const response = await fetch(`${baseUrl}/api/bootstrap/platform-config`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    postgres: databaseUrl.trim()
                        ? {
                              database_url: databaseUrl.trim(),
                          }
                        : null,
                    minio: enableMinio
                        ? {
                              endpoint: minioEndpoint.trim(),
                              access_key: minioAccessKey.trim(),
                              secret_key: minioSecretKey.trim(),
                              region: minioRegion.trim() || "us-east-1",
                          }
                        : null,
                }),
            });
            if (!response.ok) {
                const text = await response.text();
                throw new Error(text || `HTTP ${response.status}`);
            }
            const payload = (await response.json()) as BootstrapPlatformSaveResultDto;
            setMessage(payload.message);
            if (!payload.setup_required) {
                window.location.replace("/login");
            }
        } catch (submitError) {
            setError(
                submitError instanceof Error
                    ? submitError.message
                    : "保存本机平台配置失败。",
            );
        } finally {
            setSavingMode(null);
        }
    }

    async function savePostgresOnly() {
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
                        首次启动先生成本机 aio.env，再进入工作台
                    </h1>
                    <p className="mt-4 max-w-lg text-sm leading-7 text-stone-300">
                        桌面壳子、前端资源和本地 API 都在本机运行。
                        如果你的 PostgreSQL 或 MinIO 跑在 Docker Desktop 里，可以直接使用
                        host.docker.internal 示例；如果现在只想先跑起来，可以直接落本机 SQLite。
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
                        value="本机 aio.env / SQLite 首启兜底"
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
                                配置本机平台依赖
                            </CardTitle>
                            <CardDescription className="text-sm leading-6">
                                AIO 会把 PostgreSQL 和 MinIO 写进 ~/.config/aio/aio.env。
                                不需要仓库内 .env；进程环境变量只作为高级覆盖。
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
                                            placeholder="postgresql://postgres:postgres@host.docker.internal:5432/aio"
                                            autoFocus
                                        />
                                        <div className="mt-2 text-xs text-muted-foreground">
                                            Docker Desktop 示例：postgresql://postgres:postgres@host.docker.internal:5432/aio
                                        </div>
                                    </label>

                                    <div className="rounded-2xl border border-stone-300 bg-[#fcfaf4] px-4 py-4">
                                        <label className="flex items-center justify-between gap-3">
                                            <span className="flex items-center gap-2 text-sm font-medium">
                                                <HardDrive className="h-4 w-4 text-muted-foreground" />
                                                同时配置 MinIO
                                            </span>
                                            <input
                                                type="checkbox"
                                                checked={enableMinio}
                                                onChange={(event) => setEnableMinio(event.target.checked)}
                                            />
                                        </label>

                                        {enableMinio ? (
                                            <div className="mt-4 grid gap-3">
                                                <label className="block">
                                                    <span className="mb-2 block text-sm text-muted-foreground">
                                                        MinIO Endpoint
                                                    </span>
                                                    <Input
                                                        value={minioEndpoint}
                                                        onChange={(event) =>
                                                            setMinioEndpoint(event.target.value)
                                                        }
                                                        placeholder="http://host.docker.internal:9000"
                                                    />
                                                </label>
                                                <div className="grid gap-3 sm:grid-cols-2">
                                                    <label className="block">
                                                        <span className="mb-2 block text-sm text-muted-foreground">
                                                            Access Key
                                                        </span>
                                                        <Input
                                                            value={minioAccessKey}
                                                            onChange={(event) =>
                                                                setMinioAccessKey(event.target.value)
                                                            }
                                                        />
                                                    </label>
                                                    <label className="block">
                                                        <span className="mb-2 block text-sm text-muted-foreground">
                                                            Secret Key
                                                        </span>
                                                        <Input
                                                            type="password"
                                                            value={minioSecretKey}
                                                            onChange={(event) =>
                                                                setMinioSecretKey(event.target.value)
                                                            }
                                                        />
                                                    </label>
                                                </div>
                                                <label className="block">
                                                    <span className="mb-2 block text-sm text-muted-foreground">
                                                        Region
                                                    </span>
                                                    <Input
                                                        value={minioRegion}
                                                        onChange={(event) =>
                                                            setMinioRegion(event.target.value)
                                                        }
                                                        placeholder="us-east-1"
                                                    />
                                                </label>
                                                <div className="text-xs text-muted-foreground">
                                                    Docker Desktop 示例：endpoint 使用 http://host.docker.internal:9000，
                                                    默认账号 minioadmin / minioadmin，bucket 固定为 aio。
                                                </div>
                                            </div>
                                        ) : null}
                                    </div>

                                    <div className="rounded-2xl border border-dashed border-stone-300 bg-[#fcfaf4] px-4 py-3 text-sm text-muted-foreground">
                                        <div className="font-medium text-foreground">保存位置</div>
                                        <div className="mt-1 break-all">
                                            {status?.config_path || "~/.config/aio/aio.env"}
                                        </div>
                                        <div className="mt-1 text-xs text-muted-foreground">
                                            跳过 PostgreSQL 时会自动创建 `~/.config/aio/aio.sqlite3`。
                                            MinIO 配置也写在同一个文件里，不需要仓库内 .env。
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
                                            disabled={
                                                saving ||
                                                !databaseUrl.trim() ||
                                                (enableMinio &&
                                                    (!minioEndpoint.trim() ||
                                                        !minioAccessKey.trim() ||
                                                        !minioSecretKey.trim()))
                                            }
                                        >
                                            {savingMode === "platform" ? (
                                                <>
                                                    <Loader2 className="h-4 w-4 animate-spin" />
                                                    正在测试并保存
                                                </>
                                            ) : (
                                                <>
                                                    <Save className="h-4 w-4" />
                                                    测试并保存平台配置
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

                                    <Button
                                        type="button"
                                        variant="ghost"
                                        className="w-full rounded-2xl"
                                        disabled={saving || !databaseUrl.trim()}
                                        onClick={() => void savePostgresOnly()}
                                    >
                                        {savingMode === "postgres" ? (
                                            <>
                                                <Loader2 className="h-4 w-4 animate-spin" />
                                                只保存 PostgreSQL
                                            </>
                                        ) : (
                                            "只测试并保存 PostgreSQL"
                                        )}
                                    </Button>
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
