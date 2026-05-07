import { useEffect, useMemo, useState } from "react";
import {
    Blocks,
    Box,
    Cable,
    CheckCircle2,
    CircleOff,
    Loader2,
    PackageOpen,
    Puzzle,
    ShieldCheck,
    ToggleLeft,
    Trash2,
    WandSparkles,
} from "lucide-react";
import {
    getApiBaseUrl,
} from "@addzero/api-client";
import { Badge, Button, Input, Textarea } from "@addzero/ui";

interface PluginDescriptorDto {
    runtime_id: string | null;
    manifest_id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    min_platform_version: string;
    entry: string;
    extension_points: string[];
    permissions: string[];
    state: string;
    builtin: boolean;
}

const foundationPillars = [
    {
        icon: <PackageOpen className="h-4 w-4" />,
        title: ".aio-plugin 包",
        detail: "统一承载 manifest、wasm 二进制和安装元数据。",
    },
    {
        icon: <Blocks className="h-4 w-4" />,
        title: "WASM Runtime",
        detail: "通过进程内 Wasmtime / WASI 装载、启停和隔离插件实例，不为每个插件单独启动监听端口。",
    },
    {
        icon: <Cable className="h-4 w-4" />,
        title: "WIT 契约",
        detail: "统一宿主与插件之间的扩展点和生命周期调用。",
    },
];

const hostTopology = [
    {
        title: "前端入口",
        value: "1",
        detail: "开发态由 Vite 提供一个页面入口；桌面态可以继续收敛到内嵌前端资源。",
    },
    {
        title: "后端宿主",
        value: "1",
        detail: "Axum 宿主统一承载 API、插件注册、生命周期调度和扩展点装配。",
    },
    {
        title: "WASM 插件",
        value: "0 额外端口",
        detail: "插件作为宿主进程内实例运行，默认不派生独立服务，也不额外监听端口。",
    },
];

const extensionPoints = [
    "ScriptEngine",
    "AiProvider",
    "UiContribution",
    "TaskNode",
    "CliCommand",
    "TemplateGenerator",
];

const lifecycle = [
    {
        title: "发现与导入",
        detail: "从本地包、内置目录或远端注册表读入 `.aio-plugin`。",
    },
    {
        title: "校验与安装",
        detail: "检查 manifest、权限、平台版本和 wasm 入口。",
    },
    {
        title: "启用与挂载",
        detail: "把扩展点装配到脚本引擎、AI、UI、任务流和 CLI。",
    },
    {
        title: "禁用与卸载",
        detail: "单个插件可独立停用，不影响宿主和其他插件。",
    },
];

const pluginExamples = [
    {
        title: "Rhai Engine",
        type: "ScriptEngine",
        status: "Host builtin",
        detail: "现阶段虽未作为独立 wasm 插件交付，但应该朝插件化引擎靠拢。",
    },
    {
        title: "OpenAI Provider",
        type: "AiProvider",
        status: "Planned",
        detail: "把模型提供方从宿主中抽出，作为可切换能力插件。",
    },
    {
        title: "CLI Generator",
        type: "TemplateGenerator",
        status: "Planned",
        detail: "负责把工作台里的脚本和模板产出为 CLI 工程。",
    },
];

export default function MarketPage() {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const [builtinPlugins, setBuiltinPlugins] = useState<PluginDescriptorDto[]>([]);
    const [loadedPlugins, setLoadedPlugins] = useState<PluginDescriptorDto[]>([]);
    const [loadingBuiltin, setLoadingBuiltin] = useState(true);
    const [loadingLoaded, setLoadingLoaded] = useState(true);
    const [builtinError, setBuiltinError] = useState<string | null>(null);
    const [loadedError, setLoadedError] = useState<string | null>(null);
    const [actionError, setActionError] = useState<string | null>(null);
    const [actionMessage, setActionMessage] = useState<string | null>(null);
    const [pendingRuntimeId, setPendingRuntimeId] = useState<string | null>(null);
    const [installing, setInstalling] = useState(false);
    const [installForm, setInstallForm] = useState({
        id: "",
        name: "",
        version: "0.1.0",
        description: "",
        author: "addzero",
        min_platform_version: "0.1.0",
        entry: "plugin.wasm",
        extension_points: "UiContribution",
        permissions: "",
    });

    const loadBuiltinPlugins = useMemo(
        () => async () => {
            setLoadingBuiltin(true);
            setBuiltinError(null);
            try {
                const res = await fetch(`${baseUrl}/api/plugins/builtin`, {
                    credentials: "include",
                });
                if (!res.ok) {
                    throw new Error(`HTTP ${res.status}`);
                }
                setBuiltinPlugins(await res.json());
            } catch (err) {
                setBuiltinError(
                    err instanceof Error ? err.message : "加载内置插件失败",
                );
            } finally {
                setLoadingBuiltin(false);
            }
        },
        [baseUrl],
    );

    const loadLoadedPlugins = useMemo(
        () => async () => {
            setLoadingLoaded(true);
            setLoadedError(null);
            try {
                const res = await fetch(`${baseUrl}/api/plugins`, {
                    credentials: "include",
                });
                if (!res.ok) {
                    throw new Error(`HTTP ${res.status}`);
                }
                setLoadedPlugins(await res.json());
            } catch (err) {
                setLoadedError(
                    err instanceof Error ? err.message : "加载插件实例失败",
                );
            } finally {
                setLoadingLoaded(false);
            }
        },
        [baseUrl],
    );

    useEffect(() => {
        void loadBuiltinPlugins();
        void loadLoadedPlugins();
    }, [loadBuiltinPlugins, loadLoadedPlugins]);

    async function runPluginAction(
        runtimeId: string | null,
        action: "enable" | "disable" | "uninstall",
    ) {
        if (!runtimeId) {
            setActionError("缺少 runtime id，当前插件无法执行该操作");
            return;
        }
        setPendingRuntimeId(runtimeId);
        setActionError(null);
        setActionMessage(null);
        try {
            const endpoint =
                action === "enable"
                    ? `${baseUrl}/api/plugins/${runtimeId}/enable`
                    : action === "disable"
                      ? `${baseUrl}/api/plugins/${runtimeId}/disable`
                      : `${baseUrl}/api/plugins/${runtimeId}`;
            const res = await fetch(endpoint, {
                method: action === "uninstall" ? "DELETE" : "POST",
                credentials: "include",
                headers:
                    action === "uninstall"
                        ? undefined
                        : { "Content-Type": "application/json" },
                body: action === "uninstall" ? undefined : JSON.stringify({}),
            });
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            setActionMessage(
                action === "enable"
                    ? "插件已启用"
                    : action === "disable"
                      ? "插件已禁用"
                      : "插件已卸载",
            );
            await loadBuiltinPlugins();
            await loadLoadedPlugins();
        } catch (err) {
            setActionError(
                err instanceof Error ? err.message : "插件操作失败",
            );
        } finally {
            setPendingRuntimeId(null);
        }
    }

    async function installPlugin() {
        setInstalling(true);
        setActionError(null);
        setActionMessage(null);
        try {
            const res = await fetch(`${baseUrl}/api/plugins/install`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    manifest: {
                        ...installForm,
                        extension_points: installForm.extension_points
                            .split(",")
                            .map((item) => item.trim())
                            .filter(Boolean),
                        permissions: installForm.permissions
                            .split(",")
                            .map((item) => item.trim())
                            .filter(Boolean),
                    },
                    wasm_bytes: [],
                }),
            });
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            setActionMessage("插件清单已安装到运行时注册表");
            await loadLoadedPlugins();
        } catch (err) {
            setActionError(
                err instanceof Error ? err.message : "安装插件失败",
            );
        } finally {
            setInstalling(false);
        }
    }

    return (
        <div className="space-y-8">
            <section className="rounded-lg border bg-card">
                <div className="border-b px-5 py-4">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <Puzzle className="h-3.5 w-3.5" />
                        Plugin Marketplace
                    </div>
                    <h1 className="mt-3 text-3xl font-semibold tracking-tight">
                        WASM 插件市场工作台
                    </h1>
                    <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                        这个应用的基础骨架不是一组内建页面，而是一层能装配 WASM 插件的宿主。
                        市场页要围绕插件包、运行时、契约和生命周期来设计。当前目标拓扑也明确固定为
                        “一个前端入口 + 一个后端宿主”，WASM 插件在宿主进程内运行，不因为多装一个插件就多出新的监听端口。
                    </p>
                </div>

                <div className="grid gap-0 md:grid-cols-3">
                    {foundationPillars.map((item, index) => (
                        <div
                            key={item.title}
                            className={`px-5 py-4 ${
                                index > 0 ? "border-t md:border-l md:border-t-0" : ""
                            }`}
                        >
                            <div className="flex items-center gap-2 text-sm font-medium">
                                <span className="text-muted-foreground">{item.icon}</span>
                                {item.title}
                            </div>
                            <p className="mt-2 text-sm text-muted-foreground">
                                {item.detail}
                            </p>
                        </div>
                    ))}
                </div>
            </section>

            <section className="rounded-lg border bg-card">
                <div className="border-b px-5 py-4">
                    <h2 className="text-base font-semibold">宿主拓扑</h2>
                    <p className="mt-1 text-sm text-muted-foreground">
                        插件系统是进程内扩展模型，不是“每个插件再起一个服务”的微服务拼装模型。
                    </p>
                </div>
                <div className="grid gap-0 md:grid-cols-3">
                    {hostTopology.map((item, index) => (
                        <div
                            key={item.title}
                            className={`px-5 py-4 ${
                                index > 0 ? "border-t md:border-l md:border-t-0" : ""
                            }`}
                        >
                            <div className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                                {item.title}
                            </div>
                            <div className="mt-3 text-2xl font-semibold tracking-tight">
                                {item.value}
                            </div>
                            <p className="mt-2 text-sm text-muted-foreground">
                                {item.detail}
                            </p>
                        </div>
                    ))}
                </div>
            </section>

            <section className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
                <div className="rounded-lg border bg-card">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">插件生命周期</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            先把宿主平台的标准流程固定下来，确保安装、启停、卸载都围绕同一个宿主完成。
                        </p>
                    </div>
                    <div className="space-y-0">
                        {lifecycle.map((item, index) => (
                            <div
                                key={item.title}
                                className={`px-5 py-4 ${index > 0 ? "border-t" : ""}`}
                            >
                                <div className="text-sm font-medium">{item.title}</div>
                                <p className="mt-2 text-sm text-muted-foreground">
                                    {item.detail}
                                </p>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="rounded-lg border bg-card p-5">
                    <h2 className="text-base font-semibold">扩展点契约</h2>
                    <p className="mt-1 text-sm text-muted-foreground">
                        这些项已经在 `aio-plugin-api` 里定义，前台语义要和宿主契约保持一致。
                    </p>
                    <div className="mt-4 flex flex-wrap gap-2">
                        {extensionPoints.map((point) => (
                            <span
                                key={point}
                                className="rounded-md border bg-muted/40 px-2.5 py-1 text-xs font-medium"
                            >
                                {point}
                            </span>
                        ))}
                    </div>
                </div>
            </section>

            <section className="grid gap-6 xl:grid-cols-[1fr_1fr]">
                <div className="rounded-lg border bg-card">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">内置插件</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            壳子跑起来后，首先应该能看到宿主自带的基础能力，而不是一片空白。
                        </p>
                    </div>
                    {loadingBuiltin ? (
                        <div className="flex items-center justify-center px-5 py-10">
                            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                        </div>
                    ) : builtinError ? (
                        <div className="px-5 py-4 text-sm text-destructive">
                            {builtinError}
                        </div>
                    ) : (
                        <div className="space-y-0">
                            {builtinPlugins.map((plugin, index) => (
                                <div
                                    key={plugin.manifest_id}
                                    className={`px-5 py-4 ${index > 0 ? "border-t" : ""}`}
                                >
                                    <div className="flex items-center justify-between gap-4">
                                        <div>
                                            <div className="text-sm font-medium">
                                                {plugin.name}
                                            </div>
                                            <div className="mt-1 font-mono text-xs text-muted-foreground">
                                                {plugin.manifest_id}
                                            </div>
                                        </div>
                                        <Badge variant="secondary" className="text-[11px]">
                                            {plugin.state}
                                        </Badge>
                                    </div>
                                    <div className="mt-2 flex flex-wrap gap-2">
                                        {plugin.extension_points.map((point: string) => (
                                            <Badge
                                                key={point}
                                                variant="outline"
                                                className="bg-muted/30 text-[11px]"
                                            >
                                                {point}
                                            </Badge>
                                        ))}
                                    </div>
                                    <p className="mt-2 text-sm text-muted-foreground">
                                        {plugin.description}
                                    </p>
                                </div>
                            ))}
                        </div>
                    )}
                </div>

                <div className="rounded-lg border bg-card">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">市场对象</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            插件市场不只是“下载”，而是整个宿主装配面。
                        </p>
                    </div>
                    <div className="grid gap-0 sm:grid-cols-2">
                        <MarketCell
                            icon={<Box className="h-4 w-4" />}
                            title="插件包"
                            detail="导入、校验、安装、升级、回滚"
                        />
                        <MarketCell
                            icon={<ShieldCheck className="h-4 w-4" />}
                            title="权限"
                            detail="展示 manifest 申请的 capability 和风险边界"
                        />
                        <MarketCell
                            icon={<WandSparkles className="h-4 w-4" />}
                            title="能力"
                            detail="展示插件贡献了哪些引擎、节点、命令和页面"
                        />
                        <MarketCell
                            icon={<CheckCircle2 className="h-4 w-4" />}
                            title="状态"
                            detail="Installed / Active / Disabled / Error"
                        />
                    </div>
                </div>

                <div className="rounded-lg border bg-card xl:col-span-2">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">运行时插件实例</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            这里是宿主当前真正装载的插件实例，可以直接启用、禁用和卸载。
                        </p>
                    </div>
                    {actionError ? (
                        <div className="border-b px-5 py-3 text-sm text-destructive">
                            {actionError}
                        </div>
                    ) : null}
                    {actionMessage ? (
                        <div className="border-b px-5 py-3 text-sm text-emerald-600 dark:text-emerald-400">
                            {actionMessage}
                        </div>
                    ) : null}
                    {loadingLoaded ? (
                        <div className="flex items-center justify-center px-5 py-10">
                            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                        </div>
                    ) : loadedError ? (
                        <div className="px-5 py-4 text-sm text-destructive">
                            {loadedError}
                        </div>
                    ) : (
                        <div className="space-y-0">
                            {loadedPlugins.map((plugin, index) => (
                                <div
                                    key={`${plugin.runtime_id ?? plugin.manifest_id}-${index}`}
                                    className={`px-5 py-4 ${index > 0 ? "border-t" : ""}`}
                                >
                                    <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                                        <div className="min-w-0">
                                            <div className="flex flex-wrap items-center gap-2">
                                                <div className="text-sm font-medium">
                                                    {plugin.name}
                                                </div>
                                                <Badge variant="secondary" className="text-[11px]">
                                                    {plugin.state}
                                                </Badge>
                                                {plugin.builtin ? (
                                                    <Badge variant="outline" className="text-[11px]">
                                                        builtin
                                                    </Badge>
                                                ) : null}
                                            </div>
                                            <div className="mt-1 font-mono text-xs text-muted-foreground">
                                                {plugin.manifest_id}
                                                {plugin.runtime_id
                                                    ? ` · ${plugin.runtime_id}`
                                                    : ""}
                                            </div>
                                            <p className="mt-2 text-sm text-muted-foreground">
                                                {plugin.description}
                                            </p>
                                            <div className="mt-2 flex flex-wrap gap-2">
                                                {plugin.extension_points.map((point) => (
                                                    <Badge
                                                        key={point}
                                                        variant="outline"
                                                        className="bg-muted/30 text-[11px]"
                                                    >
                                                        {point}
                                                    </Badge>
                                                ))}
                                            </div>
                                        </div>
                                        <div className="flex shrink-0 flex-wrap gap-2">
                                            <Button
                                                type="button"
                                                size="sm"
                                                variant="outline"
                                                disabled={
                                                    pendingRuntimeId === plugin.runtime_id ||
                                                    plugin.state === "Active"
                                                }
                                                onClick={() =>
                                                    void runPluginAction(
                                                        plugin.runtime_id,
                                                        "enable",
                                                    )
                                                }
                                            >
                                                <ToggleLeft className="h-3.5 w-3.5" />
                                                启用
                                            </Button>
                                            <Button
                                                type="button"
                                                size="sm"
                                                variant="outline"
                                                disabled={
                                                    pendingRuntimeId === plugin.runtime_id ||
                                                    plugin.state === "Disabled"
                                                }
                                                onClick={() =>
                                                    void runPluginAction(
                                                        plugin.runtime_id,
                                                        "disable",
                                                    )
                                                }
                                            >
                                                <CircleOff className="h-3.5 w-3.5" />
                                                禁用
                                            </Button>
                                            <Button
                                                type="button"
                                                size="sm"
                                                variant="destructive"
                                                disabled={
                                                    pendingRuntimeId === plugin.runtime_id ||
                                                    plugin.builtin
                                                }
                                                onClick={() =>
                                                    void runPluginAction(
                                                        plugin.runtime_id,
                                                        "uninstall",
                                                    )
                                                }
                                            >
                                                <Trash2 className="h-3.5 w-3.5" />
                                                卸载
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            ))}
                            {loadedPlugins.length === 0 ? (
                                <div className="px-5 py-8 text-sm text-muted-foreground">
                                    当前没有已装载插件。
                                </div>
                            ) : null}
                        </div>
                    )}
                </div>

                <div className="rounded-lg border bg-card xl:col-span-2">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">目标插件画像</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            先用宿主契约定义插件画像，后续接真实 registry 和实例管理。
                        </p>
                    </div>
                    <div className="space-y-0">
                        {pluginExamples.map((plugin, index) => (
                            <div
                                key={plugin.title}
                                className={`px-5 py-4 ${index > 0 ? "border-t" : ""}`}
                            >
                                <div className="flex items-center justify-between gap-4">
                                    <div>
                                        <div className="text-sm font-medium">
                                            {plugin.title}
                                        </div>
                                        <div className="mt-1 text-xs text-muted-foreground">
                                            {plugin.type}
                                        </div>
                                    </div>
                                    <Badge variant="secondary" className="text-[11px]">
                                        {plugin.status}
                                    </Badge>
                                </div>
                                <p className="mt-2 text-sm text-muted-foreground">
                                    {plugin.detail}
                                </p>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="rounded-lg border bg-card xl:col-span-2">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">安装清单</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            这里先保留最小清单录入面，真实外部插件仍应以包含 wasm 二进制的 `.aio-plugin` 包导入。
                        </p>
                    </div>
                    <div className="grid gap-4 px-5 py-4 md:grid-cols-2">
                        <Field
                            label="插件 ID"
                            value={installForm.id}
                            onChange={(value) =>
                                setInstallForm((prev) => ({ ...prev, id: value }))
                            }
                            placeholder="com.example.demo"
                        />
                        <Field
                            label="名称"
                            value={installForm.name}
                            onChange={(value) =>
                                setInstallForm((prev) => ({ ...prev, name: value }))
                            }
                            placeholder="Demo Plugin"
                        />
                        <Field
                            label="版本"
                            value={installForm.version}
                            onChange={(value) =>
                                setInstallForm((prev) => ({ ...prev, version: value }))
                            }
                            placeholder="0.1.0"
                        />
                        <Field
                            label="作者"
                            value={installForm.author}
                            onChange={(value) =>
                                setInstallForm((prev) => ({ ...prev, author: value }))
                            }
                            placeholder="addzero"
                        />
                        <Field
                            label="最低平台版本"
                            value={installForm.min_platform_version}
                            onChange={(value) =>
                                setInstallForm((prev) => ({
                                    ...prev,
                                    min_platform_version: value,
                                }))
                            }
                            placeholder="0.1.0"
                        />
                        <Field
                            label="入口"
                            value={installForm.entry}
                            onChange={(value) =>
                                setInstallForm((prev) => ({ ...prev, entry: value }))
                            }
                            placeholder="plugin.wasm"
                        />
                        <Field
                            label="扩展点"
                            value={installForm.extension_points}
                            onChange={(value) =>
                                setInstallForm((prev) => ({
                                    ...prev,
                                    extension_points: value,
                                }))
                            }
                            placeholder="UiContribution,TemplateGenerator"
                        />
                        <Field
                            label="权限"
                            value={installForm.permissions}
                            onChange={(value) =>
                                setInstallForm((prev) => ({
                                    ...prev,
                                    permissions: value,
                                }))
                            }
                            placeholder="filesystem.write,network.outbound"
                        />
                        <div className="md:col-span-2">
                            <label className="mb-2 block text-sm font-medium">
                                描述
                            </label>
                            <Textarea
                                value={installForm.description}
                                onChange={(e) =>
                                    setInstallForm((prev) => ({
                                        ...prev,
                                        description: e.target.value,
                                    }))
                                }
                                placeholder="插件描述"
                                className="min-h-24"
                            />
                        </div>
                        <div className="md:col-span-2">
                            <Button
                                type="button"
                                onClick={() => void installPlugin()}
                                disabled={
                                    installing ||
                                    !installForm.id.trim() ||
                                    !installForm.name.trim()
                                }
                            >
                                {installing ? (
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                ) : (
                                    <PackageOpen className="h-4 w-4" />
                                )}
                                安装到运行时
                            </Button>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    );
}

function Field({
    label,
    value,
    onChange,
    placeholder,
}: {
    label: string;
    value: string;
    onChange: (value: string) => void;
    placeholder: string;
}) {
    return (
        <label className="block">
            <span className="mb-2 block text-sm font-medium">{label}</span>
            <Input
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder}
            />
        </label>
    );
}

function MarketCell({
    icon,
    title,
    detail,
}: {
    icon: React.ReactNode;
    title: string;
    detail: string;
}) {
    return (
        <div className="border-t px-5 py-4 first:border-t-0 odd:sm:border-r sm:[&:nth-child(2)]:border-t-0 sm:[&:nth-child(1)]:border-t-0">
            <div className="flex items-center gap-2 text-sm font-medium">
                <span className="text-muted-foreground">{icon}</span>
                {title}
            </div>
            <p className="mt-2 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}
