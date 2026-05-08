import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { getApiBaseUrl } from "@addzero/api-client";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Input } from "@addzero/ui";
import {
    fetchWasmPluginOverview,
    installCatalogWasmPlugin,
    type WasmPluginInstallResult,
    type WasmPluginMarketplaceEntry,
    type WasmPluginRuntimeSnapshot,
    uploadWasmPlugin,
} from "../lib/wasm-plugin-runtime";

export default function WasmPluginRuntimePanel() {
    const navigate = useNavigate();
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const [snapshot, setSnapshot] = useState<WasmPluginRuntimeSnapshot | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const [uploading, setUploading] = useState(false);
    const [pendingPluginId, setPendingPluginId] = useState<string | null>(null);
    const [lastInstall, setLastInstall] = useState<WasmPluginInstallResult | null>(null);
    const [uploadFile, setUploadFile] = useState<File | null>(null);

    async function load() {
        setLoading(true);
        setError(null);
        try {
            setSnapshot(await fetchWasmPluginOverview(baseUrl));
        } catch (err) {
            setError(err instanceof Error ? err.message : "加载 WASM 插件运行时失败");
        } finally {
            setLoading(false);
        }
    }

    useEffect(() => {
        void load();
    }, [baseUrl]);

    async function uploadPluginPackage() {
        if (!uploadFile) {
            setError("请先选择一个 `.azplugin` 插件包");
            return;
        }
        setUploading(true);
        setError(null);
        setMessage(null);
        try {
            const bytes = Array.from(new Uint8Array(await uploadFile.arrayBuffer()));
            const result = await uploadWasmPlugin(
                {
                    file_name: uploadFile.name,
                    bytes,
                },
                baseUrl,
            );
            setMessage(`已校验并导入: ${result.plugin_name} ${result.version} (${result.plugin_id})`);
            setUploadFile(null);
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await load();
        } catch (err) {
            setError(err instanceof Error ? err.message : "上传插件包失败");
        } finally {
            setUploading(false);
        }
    }

    async function installPlugin(entry: WasmPluginMarketplaceEntry) {
        setPendingPluginId(entry.plugin_id);
        setError(null);
        setMessage(null);
        try {
            const result = await installCatalogWasmPlugin(
                {
                    plugin_id: entry.plugin_id,
                    instance_label: entry.name,
                },
                baseUrl,
            );
            setLastInstall(result);
            setMessage(`实例已创建: ${result.instance_label} (${result.instance_slug})`);
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await load();
        } catch (err) {
            setError(err instanceof Error ? err.message : "安装 catalog 插件失败");
        } finally {
            setPendingPluginId(null);
        }
    }

    const entries = snapshot?.marketplace.entries ?? [];

    return (
        <section className="grid gap-6 xl:grid-cols-[0.95fr_1.05fr]">
            <Card>
                <CardHeader>
                    <CardTitle className="text-base">WASM 插件运行时</CardTitle>
                    <p className="text-sm text-muted-foreground">
                        这里接的是正式 `.azplugin` catalog + instance 链路，不再走旧的内存注册表。
                    </p>
                </CardHeader>
                <CardContent className="space-y-4">
                    {error ? (
                        <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                            {error}
                        </div>
                    ) : null}
                    {message ? (
                        <div className="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-700 dark:text-emerald-400">
                            {message}
                        </div>
                    ) : null}
                    {snapshot ? (
                        <div className="grid gap-3 sm:grid-cols-3">
                            <RuntimeMetric
                                label="系统插件"
                                value={snapshot.runtime.counts.system_plugins}
                            />
                            <RuntimeMetric
                                label="业务插件"
                                value={snapshot.runtime.counts.installed_business_plugins}
                            />
                            <RuntimeMetric
                                label="实例数"
                                value={snapshot.runtime.counts.plugin_instances}
                            />
                        </div>
                    ) : null}
                    {snapshot ? (
                        <div className="rounded-md border bg-muted/30 px-3 py-3 text-xs text-muted-foreground">
                            <div>package_root: {snapshot.runtime.package_root}</div>
                            <div className="mt-1">auth: {snapshot.runtime.dev_auth_mode}</div>
                        </div>
                    ) : null}
                    <div className="space-y-3 rounded-lg border p-4">
                        <div>
                            <div className="text-sm font-medium">上传用户插件包</div>
                            <p className="mt-1 text-sm text-muted-foreground">
                                只允许用户上传 `.azplugin` 包，服务端会先做完整校验，再放入 catalog。
                            </p>
                        </div>
                        <label className="block">
                            <span className="mb-2 block text-sm font-medium">插件包</span>
                            <Input
                                type="file"
                                accept=".azplugin,application/zip"
                                onChange={(event) =>
                                    setUploadFile(event.target.files?.[0] ?? null)
                                }
                            />
                        </label>
                        <div className="rounded-md border bg-muted/30 px-3 py-3 text-sm text-muted-foreground">
                            当前选择：{uploadFile ? uploadFile.name : "未选择文件"}
                        </div>
                        <div className="flex flex-wrap gap-2">
                            <Button
                                type="button"
                                onClick={() => void uploadPluginPackage()}
                                disabled={uploading || !uploadFile}
                            >
                                上传并校验
                            </Button>
                            <Button type="button" variant="outline" onClick={() => void load()}>
                                刷新运行时
                            </Button>
                            {lastInstall?.page_ids[0] ? (
                                <Button
                                    type="button"
                                    variant="secondary"
                                    onClick={() =>
                                        navigate(
                                            `/apps/${lastInstall.instance_slug}/${lastInstall.page_ids[0]}`,
                                        )
                                    }
                                >
                                    打开最新实例
                                </Button>
                            ) : null}
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle className="text-base">Catalog / 已可挂载插件</CardTitle>
                    <p className="text-sm text-muted-foreground">
                        业务插件从这里安装成实例，实例页会自动出现在左侧导航。
                    </p>
                </CardHeader>
                <CardContent className="space-y-3">
                    {loading ? (
                        <div className="text-sm text-muted-foreground">正在加载运行时…</div>
                    ) : entries.length === 0 ? (
                        <div className="text-sm text-muted-foreground">
                            当前 catalog 里还没有业务插件包。
                        </div>
                    ) : (
                        entries.map((entry) => (
                            <div
                                key={`${entry.plugin_id}:${entry.version}`}
                                className="rounded-lg border p-4"
                            >
                                <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                                    <div className="min-w-0">
                                        <div className="flex flex-wrap items-center gap-2">
                                            <div className="text-sm font-medium">{entry.name}</div>
                                            <Badge variant="secondary" className="text-[11px]">
                                                {entry.kind}
                                            </Badge>
                                            <Badge variant="outline" className="text-[11px]">
                                                {entry.status}
                                            </Badge>
                                        </div>
                                        <div className="mt-1 font-mono text-xs text-muted-foreground">
                                            {entry.plugin_id} · {entry.version}
                                        </div>
                                        <p className="mt-2 text-sm text-muted-foreground">
                                            {entry.summary}
                                        </p>
                                        <div className="mt-2 flex flex-wrap gap-2">
                                            {entry.tags.map((tag) => (
                                                <Badge
                                                    key={`${entry.plugin_id}:${tag}`}
                                                    variant="outline"
                                                    className="text-[11px]"
                                                >
                                                    {tag}
                                                </Badge>
                                            ))}
                                        </div>
                                    </div>
                                    {entry.kind === "Business" ? (
                                        <div className="flex shrink-0 gap-2">
                                            <Button
                                                type="button"
                                                size="sm"
                                                disabled={pendingPluginId === entry.plugin_id}
                                                onClick={() => void installPlugin(entry)}
                                            >
                                                安装实例
                                            </Button>
                                        </div>
                                    ) : null}
                                </div>
                            </div>
                        ))
                    )}
                </CardContent>
            </Card>
        </section>
    );
}

function RuntimeMetric({ label, value }: { label: string; value: number }) {
    return (
        <div className="rounded-lg border bg-muted/30 px-3 py-3">
            <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                {label}
            </div>
            <div className="mt-2 text-2xl font-semibold">{value}</div>
        </div>
    );
}
