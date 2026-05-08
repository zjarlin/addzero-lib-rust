import {
    useCallback,
    useDeferredValue,
    useEffect,
    useMemo,
    useState,
    type ReactNode,
} from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
    Blocks,
    Box,
    Cable,
    CheckCircle2,
    Loader2,
    PackageOpen,
    Puzzle,
    Search,
    ShieldCheck,
    WandSparkles,
} from "lucide-react";
import {
    Badge,
    Button,
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    Input,
    ScrollArea,
    cn,
} from "@addzero/ui";
import {
    fetchWasmPluginOverview,
    installCatalogWasmPlugin,
    registerDevWasmPlugin,
    type WasmPluginInstallResult,
    type WasmPluginMarketplaceEntry,
    type WasmPluginRuntimeSnapshot,
} from "../lib/wasm-plugin-runtime";

const sceneCards = {
    cli: {
        eyebrow: "CLI Marketplace",
        title: "CLI 插件市场",
        detail:
            "面向 cli-hub 类对象，先关注 provider、安装命令、分类、文档链接和本地脚手架导出。",
        items: [
            {
                title: "Provider 清单",
                detail: "从 cli-hub 抽取 provider、分类和示例命令。",
                tag: "cli-hub",
            },
            {
                title: "安装命令",
                detail: "保留 brew、npm、cargo、pipx、curl 等安装入口。",
                tag: "installer",
            },
            {
                title: "本机部署",
                detail: "勾选后导出为本地 `.azplugin` 子目录，后续交给 WASM 宿主装载。",
                tag: "deploy",
            },
        ],
    },
    skill: {
        eyebrow: "Skill Marketplace",
        title: "Skill 技能市场",
        detail:
            "面向 skills.sh 类对象，管理 repo、skill 名称、安装命令、正文快照和本地 Skill 同步状态。",
        items: [
            {
                title: "skills.sh 快照",
                detail: "抓取官方 owner/repo/skill 页面，保留描述和安装命令。",
                tag: "skills.sh",
            },
            {
                title: "Skill Bundle",
                detail: "把远端技能导出成插件目录，同时可转入本地 Skill 管理台。",
                tag: "skill",
            },
            {
                title: "已安装对比",
                detail: "对照当前 `/api/skills`，避免重复安装同名 Skill。",
                tag: "sync",
            },
        ],
    },
} as const;

const builtinMarketEntries = [
    {
        id: "plugin",
        name: "插件",
        summary: "内置的插件工作台入口，负责浏览、导入、安装并实例化外部 WASM 插件。",
        description:
            "这是宿主自带的市场壳层，不是外部业务插件。它承担目录浏览、catalog 导入、实例化入口和状态呈现，右侧详情区应该像 VS Code 扩展市场一样解释插件能做什么，而不是堆宿主实现细节。",
        routeHref: "/market",
        routeLabel: "当前入口",
        badges: ["Builtin", "Marketplace shell"],
        capabilities: ["市场目录", "Catalog 导入", "安装实例", "状态追踪"],
        compatibility: ["web", "desktop"],
        featureBlocks: [
            {
                title: "外部插件目录",
                detail: "左侧列出外部 WASM 插件，筛选、查看状态并进入详情。",
            },
            {
                title: "安装与实例化",
                detail: "从 catalog 安装后立即创建业务实例，实例页再挂到 `/apps/*`。",
            },
            {
                title: "宿主边界",
                detail: "它只负责装配和呈现，不把业务页面继续硬编码回宿主骨架。",
            },
        ],
    },
    {
        id: "system",
        name: "系统",
        summary: "内置的系统治理入口，承载用户、组织、字典、审计等宿主能力。",
        description:
            "系统是另一个内置入口，用来承接宿主治理能力与系统级页面。它不是外部 WASM 业务插件市场的一部分，所以在市场语义里需要和外部插件明确分层。",
        routeHref: "/system",
        routeLabel: "打开系统",
        badges: ["Builtin", "Host governance"],
        capabilities: ["用户与组织", "字典与权限", "审计与仓库", "系统页挂载"],
        compatibility: ["web", "desktop"],
        featureBlocks: [
            {
                title: "治理能力",
                detail: "系统域管理用户、组织、字典、审计与包仓库等宿主资源。",
            },
            {
                title: "固定入口",
                detail: "它跟“插件”一样属于内置入口，不应该被伪装成外部 WASM 包。",
            },
            {
                title: "系统页承载",
                detail: "系统 starter 负责输出系统页，但不改变市场里外部插件的定义。",
            },
        ],
    },
] as const;

type MarketScene = "cli" | "skill" | "wasm";

type BuiltinMarketEntry = (typeof builtinMarketEntries)[number];

type MarketListItem =
    | {
          id: string;
          kind: "builtin";
          title: string;
          summary: string;
          badges: string[];
          searchableText: string;
          entry: BuiltinMarketEntry;
      }
    | {
          id: string;
          kind: "external";
          title: string;
          summary: string;
          badges: string[];
          searchableText: string;
          entry: WasmPluginMarketplaceEntry;
      };

export default function MarketPage() {
    const params = useParams<{ scene?: string }>();
    const scene: MarketScene =
        params.scene === "cli" || params.scene === "skill" ? params.scene : "wasm";

    if (scene !== "wasm") {
        return <SceneMarketPage scene={scene} />;
    }

    return <WasmMarketplacePage />;
}

function WasmMarketplacePage() {
    const navigate = useNavigate();
    const [snapshot, setSnapshot] = useState<WasmPluginRuntimeSnapshot | null>(null);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState<string | null>(null);
    const [actionError, setActionError] = useState<string | null>(null);
    const [actionMessage, setActionMessage] = useState<string | null>(null);
    const [search, setSearch] = useState("");
    const deferredSearch = useDeferredValue(search.trim().toLowerCase());
    const [selectedId, setSelectedId] = useState<string>("builtin:plugin");
    const [pendingPluginId, setPendingPluginId] = useState<string | null>(null);
    const [lastInstall, setLastInstall] = useState<WasmPluginInstallResult | null>(null);
    const [registering, setRegistering] = useState(false);
    const [registerForm, setRegisterForm] = useState({
        source_dir: "",
        package_name: "",
    });

    const loadSnapshot = useCallback(async () => {
        setLoading(true);
        setLoadError(null);
        try {
            setSnapshot(await fetchWasmPluginOverview());
        } catch (err) {
            setLoadError(err instanceof Error ? err.message : "加载 WASM 插件市场失败");
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        void loadSnapshot();
        const refresh = () => {
            void loadSnapshot();
        };
        window.addEventListener("aio:plugin-runtime-updated", refresh);
        return () => {
            window.removeEventListener("aio:plugin-runtime-updated", refresh);
        };
    }, [loadSnapshot]);

    const builtinItems = useMemo<MarketListItem[]>(
        () =>
            builtinMarketEntries.map((entry) => ({
                id: `builtin:${entry.id}`,
                kind: "builtin",
                title: entry.name,
                summary: entry.summary,
                badges: entry.badges.slice(),
                searchableText: [
                    entry.name,
                    entry.summary,
                    entry.description,
                    ...entry.capabilities,
                    ...entry.featureBlocks.map((item) => `${item.title} ${item.detail}`),
                ]
                    .join(" ")
                    .toLowerCase(),
                entry,
            })),
        [],
    );

    const externalItems = useMemo<MarketListItem[]>(() => {
        const entries = [...(snapshot?.marketplace.entries ?? [])]
            .filter((entry) => entry.kind === "Business")
            .sort((left, right) => {
                const leftScore =
                    left.status === "Installed" ? 0 : left.status === "Available" ? 1 : 2;
                const rightScore =
                    right.status === "Installed" ? 0 : right.status === "Available" ? 1 : 2;
                return (
                    leftScore - rightScore ||
                    right.instances - left.instances ||
                    left.name.localeCompare(right.name, "zh-Hans-CN")
                );
            });
        return entries.map((entry) => ({
            id: `external:${entry.plugin_id}`,
            kind: "external",
            title: entry.name,
            summary: entry.summary,
            badges: [entry.status, `v${entry.version}`],
            searchableText: [
                entry.name,
                entry.plugin_id,
                entry.summary,
                ...entry.tags,
                ...entry.compatibility,
                ...entry.capabilities.map((item) => String(item)),
            ]
                .join(" ")
                .toLowerCase(),
            entry,
        }));
    }, [snapshot]);

    const filteredBuiltinItems = useMemo(
        () =>
            builtinItems.filter(
                (item) =>
                    !deferredSearch ||
                    item.searchableText.includes(deferredSearch) ||
                    item.title.toLowerCase().includes(deferredSearch),
            ),
        [builtinItems, deferredSearch],
    );

    const filteredExternalItems = useMemo(
        () =>
            externalItems.filter(
                (item) =>
                    !deferredSearch ||
                    item.searchableText.includes(deferredSearch) ||
                    item.title.toLowerCase().includes(deferredSearch),
            ),
        [externalItems, deferredSearch],
    );

    const visibleItems = useMemo(
        () => [...filteredBuiltinItems, ...filteredExternalItems],
        [filteredBuiltinItems, filteredExternalItems],
    );

    useEffect(() => {
        if (visibleItems.length === 0) {
            return;
        }
        if (!visibleItems.some((item) => item.id === selectedId)) {
            setSelectedId(visibleItems[0].id);
        }
    }, [selectedId, visibleItems]);

    const selectedItem = useMemo(() => {
        return (
            visibleItems.find((item) => item.id === selectedId) ??
            builtinItems.find((item) => item.id === selectedId) ??
            externalItems.find((item) => item.id === selectedId) ??
            builtinItems[0] ??
            null
        );
    }, [builtinItems, externalItems, selectedId, visibleItems]);

    const externalPluginCount = externalItems.length;
    const runtimeCounts = snapshot?.runtime.counts;

    async function installExternalPlugin(entry: WasmPluginMarketplaceEntry) {
        setPendingPluginId(entry.plugin_id);
        setActionError(null);
        setActionMessage(null);
        try {
            const result = await installCatalogWasmPlugin({
                plugin_id: entry.plugin_id,
                instance_label: entry.name,
            });
            setLastInstall(result);
            setActionMessage(`已创建实例：${result.instance_label} (${result.instance_slug})`);
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await loadSnapshot();
        } catch (err) {
            setActionError(err instanceof Error ? err.message : "安装外部插件失败");
        } finally {
            setPendingPluginId(null);
        }
    }

    async function registerExternalPlugin() {
        setRegistering(true);
        setActionError(null);
        setActionMessage(null);
        try {
            const result = await registerDevWasmPlugin(registerForm);
            setActionMessage(`已导入 catalog：${result.plugin_name} (${result.plugin_id})`);
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await loadSnapshot();
        } catch (err) {
            setActionError(err instanceof Error ? err.message : "导入本地插件失败");
        } finally {
            setRegistering(false);
        }
    }

    return (
        <div className="space-y-6">
            <Card className="overflow-hidden">
                <CardHeader className="border-b bg-[#faf8f2]">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">
                        <Puzzle className="h-3.5 w-3.5" />
                        Plugin Marketplace
                    </div>
                    <CardTitle className="mt-3 text-3xl tracking-tight">
                        WASM 插件市场
                    </CardTitle>
                    <p className="mt-2 max-w-4xl text-sm text-muted-foreground">
                        参考 VS Code 扩展市场做法：左侧负责选择插件，右侧负责解释功能、状态与安装动作。
                        这里固定只有两个内置入口“插件”“系统”，除此之外全部按外部 WASM 插件处理。
                    </p>
                </CardHeader>
                <CardContent className="grid gap-0 p-0 md:grid-cols-3">
                    <RuntimeMetric
                        label="内置入口"
                        value="2"
                        detail="固定只有“插件”“系统”两个内置项。"
                    />
                    <RuntimeMetric
                        label="外部 WASM"
                        value={String(externalPluginCount)}
                        detail="catalog 中可浏览与安装的业务插件。"
                    />
                    <RuntimeMetric
                        label="业务实例"
                        value={String(runtimeCounts?.plugin_instances ?? 0)}
                        detail="安装后创建实例，实例页挂到 `/apps/*`。"
                    />
                </CardContent>
            </Card>

            {actionError ? (
                <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                    {actionError}
                </div>
            ) : null}
            {actionMessage ? (
                <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-700 dark:text-emerald-400">
                    {actionMessage}
                </div>
            ) : null}

            <section className="grid gap-6 xl:grid-cols-[360px_minmax(0,1fr)]">
                <Card className="overflow-hidden">
                    <CardHeader className="border-b">
                        <div className="flex items-start justify-between gap-3">
                            <div>
                                <CardTitle className="text-base">插件列表</CardTitle>
                                <p className="mt-1 text-sm text-muted-foreground">
                                    左侧只做选择；右侧再解释能力、边界和动作。
                                </p>
                            </div>
                            {loading ? (
                                <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                            ) : null}
                        </div>
                        <div className="relative mt-4">
                            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                            <Input
                                value={search}
                                onChange={(event) => setSearch(event.target.value)}
                                placeholder="搜索插件、能力或标签"
                                className="pl-9"
                            />
                        </div>
                    </CardHeader>
                    <CardContent className="p-0">
                        <ScrollArea className="h-[72vh] min-h-[520px] max-h-[760px]">
                            <MarketListSection
                                title="内置"
                                subtitle="只有两个固定入口"
                                items={filteredBuiltinItems}
                                selectedId={selectedId}
                                onSelect={setSelectedId}
                            />
                            <MarketListSection
                                title="外部 WASM 插件"
                                subtitle={
                                    loadError
                                        ? "运行时离线时只保留内置入口"
                                        : "其余条目都按外部业务插件展示"
                                }
                                items={filteredExternalItems}
                                selectedId={selectedId}
                                onSelect={setSelectedId}
                                emptyState={
                                    loadError
                                        ? `未能加载外部插件：${loadError}`
                                        : loading
                                          ? "正在读取 catalog..."
                                          : "当前还没有外部 WASM 插件。"
                                }
                            />
                        </ScrollArea>
                    </CardContent>
                </Card>

                <Card className="min-h-[520px] overflow-hidden">
                    {selectedItem ? (
                        selectedItem.kind === "builtin" ? (
                            <BuiltinMarketDetail
                                entry={selectedItem.entry}
                                onOpen={() => navigate(selectedItem.entry.routeHref)}
                            />
                        ) : (
                            <ExternalMarketDetail
                                entry={selectedItem.entry}
                                loading={pendingPluginId === selectedItem.entry.plugin_id}
                                latestInstall={
                                    lastInstall?.plugin_id === selectedItem.entry.plugin_id
                                        ? lastInstall
                                        : null
                                }
                                onInstall={() => void installExternalPlugin(selectedItem.entry)}
                                onOpenLatestInstance={(install) =>
                                    navigate(
                                        `/apps/${install.instance_slug}/${install.page_ids[0]}`,
                                    )
                                }
                            />
                        )
                    ) : (
                        <CardContent className="flex min-h-[520px] items-center justify-center p-8 text-sm text-muted-foreground">
                            没有匹配到可展示的插件条目。
                        </CardContent>
                    )}
                </Card>
            </section>

            <section className="grid gap-6 xl:grid-cols-[0.95fr_1.05fr]">
                <Card>
                    <CardHeader className="border-b">
                        <CardTitle className="text-base">运行时概览</CardTitle>
                        <p className="text-sm text-muted-foreground">
                            市场本身只是壳层，正式数据与安装行为来自 WASM runtime snapshot。
                        </p>
                    </CardHeader>
                    <CardContent className="space-y-4 p-5">
                        <div className="grid gap-3 sm:grid-cols-3">
                            <CompactMetric
                                label="系统插件"
                                value={String(runtimeCounts?.system_plugins ?? 0)}
                            />
                            <CompactMetric
                                label="业务插件"
                                value={String(runtimeCounts?.installed_business_plugins ?? 0)}
                            />
                            <CompactMetric
                                label="实例数"
                                value={String(runtimeCounts?.plugin_instances ?? 0)}
                            />
                        </div>
                        <div className="rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground">
                            <div>package_root: {snapshot?.runtime.package_root ?? "--"}</div>
                            <div className="mt-1">
                                auth mode: {snapshot?.runtime.dev_auth_mode ?? "--"}
                            </div>
                        </div>
                        <div className="flex flex-wrap gap-2">
                            <Button type="button" variant="outline" onClick={() => void loadSnapshot()}>
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
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader className="border-b">
                        <CardTitle className="text-base">导入外部插件</CardTitle>
                        <p className="text-sm text-muted-foreground">
                            把本地插件源码目录打进 catalog，市场左栏随后就会把它当外部 WASM 插件显示出来。
                        </p>
                    </CardHeader>
                    <CardContent className="grid gap-4 p-5 md:grid-cols-2">
                        <Field
                            label="源码目录"
                            value={registerForm.source_dir}
                            onChange={(value) =>
                                setRegisterForm((current) => ({
                                    ...current,
                                    source_dir: value,
                                }))
                            }
                            placeholder="/absolute/path/to/plugin-source"
                        />
                        <Field
                            label="包名（可选）"
                            value={registerForm.package_name}
                            onChange={(value) =>
                                setRegisterForm((current) => ({
                                    ...current,
                                    package_name: value,
                                }))
                            }
                            placeholder="memory-manager"
                        />
                        <div className="md:col-span-2 rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground">
                            目录内至少包含 `plugin.toml`、`backend/plugin.wasm`、`checksums.sha256`。
                        </div>
                        <div className="md:col-span-2 flex flex-wrap gap-2">
                            <Button
                                type="button"
                                onClick={() => void registerExternalPlugin()}
                                disabled={registering || !registerForm.source_dir.trim()}
                            >
                                {registering ? (
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                ) : (
                                    <PackageOpen className="h-4 w-4" />
                                )}
                                导入到 catalog
                            </Button>
                        </div>
                    </CardContent>
                </Card>
            </section>
        </div>
    );
}

function BuiltinMarketDetail({
    entry,
    onOpen,
}: {
    entry: BuiltinMarketEntry;
    onOpen: () => void;
}) {
    return (
        <>
            <CardHeader className="border-b bg-[#fbfaf5]">
                <div className="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">
                    <Puzzle className="h-3.5 w-3.5" />
                    Built-in Entry
                </div>
                <div className="mt-3 flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                    <div className="min-w-0">
                        <CardTitle className="text-3xl tracking-tight">{entry.name}</CardTitle>
                        <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                            {entry.summary}
                        </p>
                    </div>
                    <div className="flex shrink-0 flex-wrap gap-2">
                        <Button type="button" onClick={onOpen}>
                            {entry.routeLabel}
                        </Button>
                    </div>
                </div>
                <div className="mt-4 flex flex-wrap gap-2">
                    {entry.badges.map((badge) => (
                        <Badge key={badge} variant="secondary" className="text-[11px]">
                            {badge}
                        </Badge>
                    ))}
                    {entry.compatibility.map((item) => (
                        <Badge key={item} variant="outline" className="text-[11px]">
                            {item}
                        </Badge>
                    ))}
                </div>
            </CardHeader>
            <CardContent className="space-y-6 p-6">
                <section className="rounded-2xl border bg-muted/20 p-5">
                    <div className="text-sm font-medium">定位说明</div>
                    <p className="mt-2 text-sm leading-6 text-muted-foreground">
                        {entry.description}
                    </p>
                </section>

                <section className="grid gap-4 lg:grid-cols-3">
                    {entry.featureBlocks.map((item) => (
                        <div key={item.title} className="rounded-2xl border p-5">
                            <div className="text-sm font-medium">{item.title}</div>
                            <p className="mt-2 text-sm leading-6 text-muted-foreground">
                                {item.detail}
                            </p>
                        </div>
                    ))}
                </section>

                <section className="grid gap-4 lg:grid-cols-[0.85fr_1.15fr]">
                    <InfoPanel
                        title="能力边界"
                        items={entry.capabilities.map((item) => ({
                            label: item,
                            detail: "内置壳层负责市场行为与宿主管理，不把业务页回写到主应用。",
                        }))}
                    />
                    <InfoPanel
                        title="市场语义"
                        items={[
                            {
                                label: "只有两个内置项",
                                detail: "插件 / 系统 是固定入口，不随 catalog 内容变化。",
                            },
                            {
                                label: "其余全是外部 WASM",
                                detail: "外部条目来自 catalog snapshot，并按业务插件语义处理。",
                            },
                            {
                                label: "列表与详情分离",
                                detail: "左侧做选择，右侧解释功能与动作，避免再把宿主实现堆成说明墙。",
                            },
                        ]}
                    />
                </section>
            </CardContent>
        </>
    );
}

function ExternalMarketDetail({
    entry,
    loading,
    latestInstall,
    onInstall,
    onOpenLatestInstance,
}: {
    entry: WasmPluginMarketplaceEntry;
    loading: boolean;
    latestInstall: WasmPluginInstallResult | null;
    onInstall: () => void;
    onOpenLatestInstance: (install: WasmPluginInstallResult) => void;
}) {
    const primaryActionLabel =
        entry.status === "Available" ? "安装并创建实例" : "创建新实例";

    return (
        <>
            <CardHeader className="border-b bg-[#fcfbf8]">
                <div className="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">
                    <Blocks className="h-3.5 w-3.5" />
                    External WASM Plugin
                </div>
                <div className="mt-3 flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                    <div className="min-w-0">
                        <CardTitle className="text-3xl tracking-tight">{entry.name}</CardTitle>
                        <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                            {entry.summary}
                        </p>
                        <div className="mt-3 font-mono text-xs text-muted-foreground">
                            {entry.plugin_id} · v{entry.version}
                        </div>
                    </div>
                    <div className="flex shrink-0 flex-wrap gap-2">
                        <Button type="button" onClick={onInstall} disabled={loading}>
                            {loading ? (
                                <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                                <PackageOpen className="h-4 w-4" />
                            )}
                            {primaryActionLabel}
                        </Button>
                        {latestInstall?.page_ids[0] ? (
                            <Button
                                type="button"
                                variant="secondary"
                                onClick={() => onOpenLatestInstance(latestInstall)}
                            >
                                打开最新实例
                            </Button>
                        ) : null}
                    </div>
                </div>
                <div className="mt-4 flex flex-wrap gap-2">
                    <Badge variant="secondary" className="text-[11px]">
                        {entry.status}
                    </Badge>
                    <Badge variant="outline" className="text-[11px]">
                        {entry.instances} 个实例
                    </Badge>
                    {entry.tags.map((tag) => (
                        <Badge key={tag} variant="outline" className="text-[11px]">
                            {tag}
                        </Badge>
                    ))}
                </div>
            </CardHeader>
            <CardContent className="space-y-6 p-6">
                <section className="grid gap-4 lg:grid-cols-3">
                    <FeatureCard
                        icon={<WandSparkles className="h-4 w-4" />}
                        title="功能定位"
                        detail="这是外部业务 WASM 插件，不属于宿主内置入口。安装后由宿主创建实例页。"
                    />
                    <FeatureCard
                        icon={<Cable className="h-4 w-4" />}
                        title="接入方式"
                        detail="通过 catalog 包接入宿主，在同一宿主进程内运行，不额外长出新端口。"
                    />
                    <FeatureCard
                        icon={<CheckCircle2 className="h-4 w-4" />}
                        title="当前状态"
                        detail={`状态 ${entry.status}，当前已有 ${entry.instances} 个业务实例。`}
                    />
                </section>

                <section className="grid gap-4 lg:grid-cols-[0.95fr_1.05fr]">
                    <InfoPanel
                        title="能力贡献"
                        items={
                            entry.capabilities.length > 0
                                ? entry.capabilities.map((item) => ({
                                      label: String(item),
                                      detail: "由插件 descriptor 声明，宿主按能力边界装配。",
                                  }))
                                : [
                                      {
                                          label: "未声明 capability",
                                          detail: "当前 descriptor 还没有暴露额外宿主能力。",
                                      },
                                  ]
                        }
                    />
                    <InfoPanel
                        title="兼容性与标签"
                        items={[
                            ...(entry.compatibility.length > 0
                                ? entry.compatibility.map((item) => ({
                                      label: item,
                                      detail: "声明的宿主兼容面。",
                                  }))
                                : [
                                      {
                                          label: "未声明兼容性",
                                          detail: "当前插件没有附带 compatibility 元数据。",
                                      },
                                  ]),
                            ...(entry.tags.length > 0
                                ? entry.tags.map((item) => ({
                                      label: `#${item}`,
                                      detail: "用于列表分类与检索。",
                                  }))
                                : []),
                        ]}
                    />
                </section>

                <section className="rounded-2xl border bg-muted/20 p-5">
                    <div className="text-sm font-medium">实例化说明</div>
                    <p className="mt-2 text-sm leading-6 text-muted-foreground">
                        外部插件安装后不是直接把页面硬编码进主应用，而是先进入宿主 registry，
                        再由宿主生成实例并把实例页挂到 `/apps/*`。这跟 VS Code 扩展先安装、再激活、再贡献页面的语义更接近。
                    </p>
                </section>
            </CardContent>
        </>
    );
}

function MarketListSection({
    title,
    subtitle,
    items,
    selectedId,
    onSelect,
    emptyState,
}: {
    title: string;
    subtitle: string;
    items: MarketListItem[];
    selectedId: string;
    onSelect: (id: string) => void;
    emptyState?: string;
}) {
    return (
        <section className="border-b last:border-b-0">
            <div className="border-b px-4 py-3">
                <div className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                    {title}
                </div>
                <div className="mt-1 text-sm text-muted-foreground">{subtitle}</div>
            </div>
            {items.length === 0 ? (
                <div className="px-4 py-5 text-sm text-muted-foreground">
                    {emptyState ?? "暂无条目。"}
                </div>
            ) : (
                <div className="space-y-0">
                    {items.map((item) => (
                        <button
                            key={item.id}
                            type="button"
                            onClick={() => onSelect(item.id)}
                            className={cn(
                                "w-full border-b px-4 py-4 text-left transition last:border-b-0 hover:bg-muted/40",
                                selectedId === item.id &&
                                    "bg-[#f5f1e8] shadow-[inset_2px_0_0_0_rgba(24,24,27,0.92)]",
                            )}
                        >
                            <div className="flex items-start justify-between gap-3">
                                <div className="min-w-0">
                                    <div className="truncate text-sm font-medium">
                                        {item.title}
                                    </div>
                                    <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                                        {item.summary}
                                    </p>
                                </div>
                                <Badge
                                    variant={item.kind === "builtin" ? "secondary" : "outline"}
                                    className="shrink-0 text-[11px]"
                                >
                                    {item.kind === "builtin" ? "内置" : "外部"}
                                </Badge>
                            </div>
                            <div className="mt-3 flex flex-wrap gap-2">
                                {item.badges.slice(0, 3).map((badge) => (
                                    <Badge
                                        key={`${item.id}:${badge}`}
                                        variant="outline"
                                        className="text-[11px]"
                                    >
                                        {badge}
                                    </Badge>
                                ))}
                            </div>
                        </button>
                    ))}
                </div>
            )}
        </section>
    );
}

function SceneMarketPage({ scene }: { scene: Exclude<MarketScene, "wasm"> }) {
    const content = sceneCards[scene];

    return (
        <div className="space-y-6">
            <section className="rounded-lg border bg-card">
                <div className="border-b px-5 py-4">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <Puzzle className="h-3.5 w-3.5" />
                        {content.eyebrow}
                    </div>
                    <h1 className="mt-3 text-3xl font-semibold tracking-tight">
                        {content.title}
                    </h1>
                    <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                        {content.detail}
                    </p>
                </div>
                <div className="grid gap-0 md:grid-cols-3">
                    {content.items.map((item, index) => (
                        <div
                            key={item.title}
                            className={`px-5 py-4 ${
                                index > 0 ? "border-t md:border-l md:border-t-0" : ""
                            }`}
                        >
                            <div className="flex items-center justify-between gap-3">
                                <div className="text-sm font-medium">{item.title}</div>
                                <Badge variant="outline" className="text-[11px]">
                                    {item.tag}
                                </Badge>
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
                    <h2 className="text-base font-semibold">市场目录</h2>
                    <p className="mt-1 text-sm text-muted-foreground">
                        这里下一步接 `market_hub` 的真实抓取结果，并支持勾选部署到本机目录。
                    </p>
                </div>
                <div className="grid gap-0 md:grid-cols-2">
                    <MarketCell
                        icon={<Box className="h-4 w-4" />}
                        title="对象采集"
                        detail={
                            scene === "cli"
                                ? "来源优先对齐 cli-hub provider/schema。"
                                : "来源优先对齐 skills.sh official/detail 页面。"
                        }
                    />
                    <MarketCell
                        icon={<PackageOpen className="h-4 w-4" />}
                        title="部署目录"
                        detail="每个市场对象导出到独立子文件夹，并生成 `.azplugin` 包。"
                    />
                    <MarketCell
                        icon={<ShieldCheck className="h-4 w-4" />}
                        title="安装边界"
                        detail="先写入本机目录，安装动作再交给 WASM 插件运行时统一处理。"
                    />
                    <MarketCell
                        icon={<CheckCircle2 className="h-4 w-4" />}
                        title="状态回填"
                        detail="记录已部署、已安装、冲突和源不可用状态。"
                    />
                </div>
            </section>
        </div>
    );
}

function FeatureCard({
    icon,
    title,
    detail,
}: {
    icon: ReactNode;
    title: string;
    detail: string;
}) {
    return (
        <div className="rounded-2xl border p-5">
            <div className="flex items-center gap-2 text-sm font-medium">
                <span className="text-muted-foreground">{icon}</span>
                {title}
            </div>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">{detail}</p>
        </div>
    );
}

function InfoPanel({
    title,
    items,
}: {
    title: string;
    items: { label: string; detail: string }[];
}) {
    return (
        <div className="rounded-2xl border">
            <div className="border-b px-5 py-4">
                <div className="text-sm font-medium">{title}</div>
            </div>
            <div className="space-y-0">
                {items.map((item, index) => (
                    <div
                        key={`${item.label}:${index}`}
                        className={cn("px-5 py-4", index > 0 && "border-t")}
                    >
                        <div className="text-sm font-medium">{item.label}</div>
                        <p className="mt-2 text-sm leading-6 text-muted-foreground">
                            {item.detail}
                        </p>
                    </div>
                ))}
            </div>
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
                onChange={(event) => onChange(event.target.value)}
                placeholder={placeholder}
            />
        </label>
    );
}

function RuntimeMetric({
    label,
    value,
    detail,
}: {
    label: string;
    value: string;
    detail: string;
}) {
    return (
        <div className="px-5 py-4 md:border-l first:md:border-l-0">
            <div className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {label}
            </div>
            <div className="mt-3 text-2xl font-semibold tracking-tight">{value}</div>
            <p className="mt-2 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}

function CompactMetric({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-lg border bg-muted/30 px-4 py-3">
            <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                {label}
            </div>
            <div className="mt-2 text-2xl font-semibold">{value}</div>
        </div>
    );
}

function MarketCell({
    icon,
    title,
    detail,
}: {
    icon: ReactNode;
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
