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
    ArrowUpRight,
    Blocks,
    CheckCircle2,
    ChevronRight,
    CircleDot,
    Filter,
    FolderArchive,
    Loader2,
    PackageOpen,
    PanelTop,
    Puzzle,
    RefreshCw,
    Search,
    Settings2,
    ShieldCheck,
    Sparkles,
    Upload,
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
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
    Textarea,
    cn,
} from "@az/ui";
import {
    fetchWasmPluginOverview,
    installCatalogWasmPlugin,
    type WasmPluginInstallResult,
    type WasmPluginMarketplaceEntry,
    type WasmPluginRuntimeSnapshot,
    uploadWasmPluginFirmware,
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

const builtinEntries = [
    {
        id: "plugin",
        name: "插件",
        lane: "builtin" as const,
        routeHref: "/market",
        statusTone: "Core shell",
        summary: "插件市场与安装壳层。",
        highlights: ["目录浏览", "上传校验", "安装实例"],
        chips: ["Builtin", "Marketplace"],
        health: "stable" as const,
    },
    {
        id: "system",
        name: "系统",
        lane: "builtin" as const,
        routeHref: "/system",
        statusTone: "Host governance",
        summary: "宿主治理与系统级能力入口。",
        highlights: ["用户组织", "字典审计", "系统页挂载"],
        chips: ["Builtin", "Governance"],
        health: "stable" as const,
    },
] as const;

type MarketScene = "cli" | "skill" | "wasm";
type BuiltinEntry = (typeof builtinEntries)[number];
type StatusFilter = "all" | "installed" | "available" | "disabled" | "builtin";
type LaneFilter = "all" | "builtin" | "external";
type SortMode = "featured" | "name" | "instances";
type DetailTab = "overview" | "package" | "activity";
type FirmwareKind = "System" | "Business";

type MarketEntryItem =
    | {
          id: string;
          lane: "builtin";
          name: string;
          summary: string;
          searchText: string;
          entry: BuiltinEntry;
          chips: string[];
          state: "Builtin";
          instanceCount: number;
      }
    | {
          id: string;
          lane: "external";
          name: string;
          summary: string;
          searchText: string;
          entry: WasmPluginMarketplaceEntry;
          chips: string[];
          state: WasmPluginMarketplaceEntry["status"];
          instanceCount: number;
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
    const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
    const [laneFilter, setLaneFilter] = useState<LaneFilter>("all");
    const [sortMode, setSortMode] = useState<SortMode>("featured");
    const [selectedId, setSelectedId] = useState<string>("builtin:plugin");
    const [detailTab, setDetailTab] = useState<DetailTab>("overview");
    const [pendingPluginId, setPendingPluginId] = useState<string | null>(null);
    const [lastInstall, setLastInstall] = useState<WasmPluginInstallResult | null>(null);
    const [uploading, setUploading] = useState(false);
    const [uploadFile, setUploadFile] = useState<File | null>(null);
    const [firmwareName, setFirmwareName] = useState("");
    const [firmwareDescription, setFirmwareDescription] = useState("");
    const [firmwareKind, setFirmwareKind] = useState<FirmwareKind>("Business");

    const loadSnapshot = useCallback(async () => {
        setLoading(true);
        setLoadError(null);
        try {
            setSnapshot(await fetchWasmPluginOverview());
        } catch (err) {
            setLoadError(err instanceof Error ? err.message : "加载插件市场失败");
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

    const builtinItems = useMemo<MarketEntryItem[]>(
        () =>
            builtinEntries.map((entry) => ({
                id: `builtin:${entry.id}`,
                lane: "builtin",
                name: entry.name,
                summary: entry.summary,
                searchText: [
                    entry.name,
                    entry.summary,
                    entry.statusTone,
                    ...entry.highlights,
                    ...entry.chips,
                ]
                    .join(" ")
                    .toLowerCase(),
                entry,
                chips: entry.chips.slice(),
                state: "Builtin",
                instanceCount: 0,
            })),
        [],
    );

    const externalItems = useMemo<MarketEntryItem[]>(() => {
        const items = snapshot?.marketplace.entries ?? [];
        return items
            .filter((entry) => entry.kind === "Business")
            .map((entry) => ({
                id: `external:${entry.plugin_id}`,
                lane: "external" as const,
                name: entry.name,
                summary: entry.summary,
                searchText: [
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
                chips: [
                    entry.status,
                    ...entry.tags.slice(0, 2),
                    ...entry.compatibility.slice(0, 1),
                ],
                state: entry.status,
                instanceCount: entry.instances,
            }));
    }, [snapshot]);

    const allItems = useMemo(
        () => [...builtinItems, ...externalItems],
        [builtinItems, externalItems],
    );

    const filteredItems = useMemo(() => {
        const matched = allItems.filter((item) => {
            if (laneFilter !== "all" && item.lane !== laneFilter) {
                return false;
            }
            if (statusFilter === "builtin" && item.lane !== "builtin") {
                return false;
            }
            if (statusFilter === "installed" && item.state !== "Installed") {
                return false;
            }
            if (statusFilter === "available" && item.state !== "Available") {
                return false;
            }
            if (statusFilter === "disabled" && item.state !== "Disabled") {
                return false;
            }
            if (
                deferredSearch &&
                !item.searchText.includes(deferredSearch) &&
                !item.name.toLowerCase().includes(deferredSearch)
            ) {
                return false;
            }
            return true;
        });

        const sorted = [...matched];
        sorted.sort((left, right) => {
            if (sortMode === "name") {
                return left.name.localeCompare(right.name, "zh-Hans-CN");
            }
            if (sortMode === "instances") {
                return (
                    right.instanceCount - left.instanceCount ||
                    left.name.localeCompare(right.name, "zh-Hans-CN")
                );
            }

            const laneScore = (item: MarketEntryItem) =>
                item.lane === "builtin"
                    ? 0
                    : item.state === "Installed"
                      ? 1
                      : item.state === "Available"
                        ? 2
                        : 3;
            return (
                laneScore(left) - laneScore(right) ||
                right.instanceCount - left.instanceCount ||
                left.name.localeCompare(right.name, "zh-Hans-CN")
            );
        });
        return sorted;
    }, [allItems, deferredSearch, laneFilter, sortMode, statusFilter]);

    useEffect(() => {
        if (filteredItems.length === 0) {
            return;
        }
        if (!filteredItems.some((item) => item.id === selectedId)) {
            setSelectedId(filteredItems[0].id);
        }
    }, [filteredItems, selectedId]);

    const selectedItem = useMemo(
        () =>
            filteredItems.find((item) => item.id === selectedId) ??
            allItems.find((item) => item.id === selectedId) ??
            builtinItems[0] ??
            null,
        [allItems, builtinItems, filteredItems, selectedId],
    );

    const runtimeCounts = snapshot?.runtime.counts;
    const installedExternalCount = externalItems.filter(
        (item) => item.state === "Installed",
    ).length;
    const availableExternalCount = externalItems.filter(
        (item) => item.state === "Available",
    ).length;
    const disabledExternalCount = externalItems.filter(
        (item) => item.state === "Disabled",
    ).length;
    const sortLabel =
        sortMode === "name" ? "名称" : sortMode === "instances" ? "实例" : "推荐";
    const railMode =
        statusFilter === "installed"
            ? "installed"
            : statusFilter === "available"
              ? "available"
              : laneFilter === "builtin" || statusFilter === "builtin"
                ? "builtin"
                : laneFilter === "external"
                  ? "external"
                  : "all";

    function handleUploadSelection(file: File | null) {
        setActionError(null);
        if (!file) {
            setUploadFile(null);
            return;
        }
        if (!file.name.toLowerCase().endsWith(".wasm")) {
            setUploadFile(null);
            setActionError("固件上传只接收 `.wasm` 二进制");
            return;
        }
        setUploadFile(file);
        if (!firmwareName.trim()) {
            setFirmwareName(file.name.replace(/\.wasm$/i, ""));
        }
    }

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
            setDetailTab("activity");
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await loadSnapshot();
        } catch (err) {
            setActionError(err instanceof Error ? err.message : "安装插件失败");
        } finally {
            setPendingPluginId(null);
        }
    }

    async function uploadFirmwarePlugin() {
        const name = firmwareName.trim();
        const description = firmwareDescription.trim();
        if (!name) {
            setActionError("请填写插件名");
            return;
        }
        if (!description) {
            setActionError("请填写描述");
            return;
        }
        if (!uploadFile) {
            setActionError("请先选择一个 `.wasm` 固件文件");
            return;
        }
        setUploading(true);
        setActionError(null);
        setActionMessage(null);
        try {
            const bytes = Array.from(new Uint8Array(await uploadFile.arrayBuffer()));
            const result = await uploadWasmPluginFirmware({
                name,
                description,
                firmware_kind: firmwareKind,
                file_name: uploadFile.name,
                bytes,
            });
            setActionMessage(
                `已写入 PG/MinIO：${result.plugin_name} ${result.version} (${result.plugin_id})`,
            );
            setUploadFile(null);
            setFirmwareName("");
            setFirmwareDescription("");
            setDetailTab("package");
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await loadSnapshot();
        } catch (err) {
            setActionError(err instanceof Error ? err.message : "上传固件失败");
        } finally {
            setUploading(false);
        }
    }

    return (
        <div className="space-y-6">
            <section className="overflow-hidden rounded-[32px] border border-stone-200/80 bg-[linear-gradient(180deg,#fdfaf2_0%,#f5eedd_40%,#fffdfa_100%)] shadow-[0_28px_90px_rgba(39,35,24,0.08)]">
                <div className="px-7 py-7">
                    <div className="flex flex-wrap items-center justify-between gap-4">
                        <div className="flex items-center gap-3 text-[11px] uppercase tracking-[0.28em] text-stone-500">
                            <PanelTop className="h-3.5 w-3.5" />
                            Plugin Marketplace
                        </div>
                        <div className="flex flex-wrap items-center gap-2">
                            <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                Built-in / 插件
                            </Badge>
                            <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                Built-in / 系统
                            </Badge>
                            <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                PG Metadata / MinIO WASM
                            </Badge>
                        </div>
                    </div>

                    <div className="mt-6 grid gap-5 xl:grid-cols-[minmax(0,1fr)_132px_minmax(0,1fr)]">
                        <div className="space-y-5">
                            <div className="space-y-3">
                                <h1 className="text-4xl font-semibold tracking-[-0.05em] text-stone-950 sm:text-5xl">
                                    WASM 插件市场
                                </h1>
                                <div className="flex flex-wrap gap-2">
                                    <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                        `.wasm` firmware
                                    </Badge>
                                    <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                        PG metadata
                                    </Badge>
                                    <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                        MinIO binary
                                    </Badge>
                                </div>
                            </div>

                            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                                <QuickMetricButton
                                    label="内置"
                                    value="2"
                                    detail="插件 / 系统"
                                    active={laneFilter === "builtin" || statusFilter === "builtin"}
                                    onClick={() => {
                                        setLaneFilter("builtin");
                                        setStatusFilter("builtin");
                                    }}
                                />
                                <QuickMetricButton
                                    label="外部"
                                    value={String(externalItems.length)}
                                    detail="全部外部 WASM"
                                    active={laneFilter === "external" && statusFilter === "all"}
                                    onClick={() => {
                                        setLaneFilter("external");
                                        setStatusFilter("all");
                                    }}
                                />
                                <QuickMetricButton
                                    label="已装"
                                    value={String(installedExternalCount)}
                                    detail="可继续建实例"
                                    active={statusFilter === "installed"}
                                    onClick={() => {
                                        setLaneFilter("external");
                                        setStatusFilter("installed");
                                    }}
                                />
                                <QuickMetricButton
                                    label="待装"
                                    value={String(availableExternalCount)}
                                    detail="已入 PG"
                                    active={statusFilter === "available"}
                                    onClick={() => {
                                        setLaneFilter("external");
                                        setStatusFilter("available");
                                    }}
                                />
                                <QuickMetricButton
                                    label="停用"
                                    value={String(disabledExternalCount)}
                                    detail="暂不可安装"
                                    active={statusFilter === "disabled"}
                                    onClick={() => {
                                        setLaneFilter("external");
                                        setStatusFilter("disabled");
                                    }}
                                />
                                <QuickMetricButton
                                    label="实例"
                                    value={String(runtimeCounts?.plugin_instances ?? 0)}
                                    detail="运行中的业务实例"
                                    active={false}
                                    onClick={() => {
                                        setLaneFilter("external");
                                        setStatusFilter("installed");
                                    }}
                                />
                            </div>

                            <div className="flex flex-wrap gap-3">
                                <Button
                                    type="button"
                                    className="rounded-full bg-stone-950 px-5"
                                    onClick={() => void uploadFirmwarePlugin()}
                                    disabled={
                                        uploading ||
                                        !uploadFile ||
                                        !firmwareName.trim() ||
                                        !firmwareDescription.trim()
                                    }
                                >
                                    {uploading ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <Upload className="h-4 w-4" />
                                    )}
                                    上传固件
                                </Button>
                                <Button
                                    type="button"
                                    variant="outline"
                                    className="rounded-full bg-white/80 px-5"
                                    onClick={() => void loadSnapshot()}
                                >
                                    <RefreshCw className="h-4 w-4" />
                                    刷新插件
                                </Button>
                            </div>
                        </div>

                        <CenterFilterSpine
                            active={railMode}
                            onSelect={(mode) => {
                                if (mode === "all") {
                                    setLaneFilter("all");
                                    setStatusFilter("all");
                                    return;
                                }
                                if (mode === "builtin") {
                                    setLaneFilter("builtin");
                                    setStatusFilter("builtin");
                                    return;
                                }
                                if (mode === "external") {
                                    setLaneFilter("external");
                                    setStatusFilter("all");
                                    return;
                                }
                                setLaneFilter("external");
                                setStatusFilter(mode as StatusFilter);
                            }}
                        />

                        <div className="grid gap-4">
                            <DockCard
                                title="固件入库"
                                eyebrow="Firmware Upload"
                                icon={<Upload className="h-4 w-4" />}
                            >
                                <div className="grid gap-3 sm:grid-cols-2">
                                    <label className="block">
                                        <span className="mb-2 block text-xs uppercase tracking-[0.18em] text-stone-500">
                                            插件名
                                        </span>
                                        <Input
                                            value={firmwareName}
                                            onChange={(event) => setFirmwareName(event.target.value)}
                                            placeholder="例如 工业协议转换网关"
                                            className="rounded-2xl border-stone-200 bg-white/80"
                                        />
                                    </label>
                                    <SegmentGroup
                                        label="分类"
                                        value={firmwareKind}
                                        options={[
                                            { id: "System", label: "系统固件" },
                                            { id: "Business", label: "业务固件" },
                                        ]}
                                        onChange={(value) => setFirmwareKind(value as FirmwareKind)}
                                    />
                                </div>

                                <label className="block">
                                    <span className="mb-2 block text-xs uppercase tracking-[0.18em] text-stone-500">
                                        描述
                                    </span>
                                    <Textarea
                                        value={firmwareDescription}
                                        onChange={(event) =>
                                            setFirmwareDescription(event.target.value)
                                        }
                                        placeholder="说明这个固件暴露的协议、能力或业务入口"
                                        className="min-h-[88px] rounded-2xl border-stone-200 bg-white/80"
                                    />
                                </label>

                                <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
                                    <label className="block">
                                        <span className="mb-2 block text-xs uppercase tracking-[0.18em] text-stone-500">
                                            固件文件
                                        </span>
                                        <Input
                                            type="file"
                                            accept=".wasm,application/wasm"
                                            onChange={(event) =>
                                                handleUploadSelection(
                                                    event.target.files?.[0] ?? null,
                                                )
                                            }
                                            className="rounded-2xl border-stone-200 bg-white/80"
                                        />
                                    </label>
                                    <Button
                                        type="button"
                                        className="rounded-full bg-stone-950 px-5"
                                        onClick={() => void uploadFirmwarePlugin()}
                                        disabled={
                                            uploading ||
                                            !uploadFile ||
                                            !firmwareName.trim() ||
                                            !firmwareDescription.trim()
                                        }
                                    >
                                        {uploading ? (
                                            <Loader2 className="h-4 w-4 animate-spin" />
                                        ) : (
                                            <Upload className="h-4 w-4" />
                                        )}
                                        入库
                                    </Button>
                                </div>

                                <div className="grid gap-2 sm:grid-cols-2">
                                    <CompactInfoTile
                                        label="文件"
                                        value={uploadFile?.name ?? "未选择"}
                                    />
                                    <CompactInfoTile
                                        label="分类"
                                        value={firmwareKind === "System" ? "系统固件" : "业务固件"}
                                    />
                                </div>

                                <div className="flex flex-wrap gap-2">
                                    <Badge variant="outline" className="rounded-full bg-white/80">
                                        `.wasm`
                                    </Badge>
                                    <Badge variant="outline" className="rounded-full bg-white/80">
                                        PostgreSQL
                                    </Badge>
                                    <Badge variant="outline" className="rounded-full bg-white/80">
                                        MinIO
                                    </Badge>
                                    <Badge variant="outline" className="rounded-full bg-white/80">
                                        server descriptor
                                    </Badge>
                                </div>
                            </DockCard>

                            <DockCard
                                title="检索轨道"
                                eyebrow="Control Rail"
                                icon={<Filter className="h-4 w-4" />}
                            >
                                <div className="relative">
                                    <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-stone-400" />
                                    <Input
                                        value={search}
                                        onChange={(event) => setSearch(event.target.value)}
                                        placeholder="搜索插件名、标签、能力"
                                        className="rounded-2xl border-stone-200 bg-white/80 pl-9"
                                    />
                                </div>

                                <div className="grid gap-2 sm:grid-cols-2">
                                    <SegmentGroup
                                        label="范围"
                                        value={laneFilter}
                                        options={[
                                            { id: "all", label: "全部" },
                                            { id: "builtin", label: "内置" },
                                            { id: "external", label: "外部" },
                                        ]}
                                        onChange={(value) => setLaneFilter(value as LaneFilter)}
                                    />
                                    <SegmentGroup
                                        label="状态"
                                        value={statusFilter}
                                        options={[
                                            { id: "all", label: "全部" },
                                            { id: "installed", label: "已装" },
                                            { id: "available", label: "待装" },
                                            { id: "disabled", label: "停用" },
                                            { id: "builtin", label: "内置" },
                                        ]}
                                        onChange={(value) => setStatusFilter(value as StatusFilter)}
                                    />
                                </div>

                                <SegmentGroup
                                    label="排序"
                                    value={sortMode}
                                    options={[
                                        { id: "featured", label: "推荐" },
                                        { id: "name", label: "名称" },
                                        { id: "instances", label: "实例" },
                                    ]}
                                    onChange={(value) => setSortMode(value as SortMode)}
                                />

                                <div className="grid gap-2 sm:grid-cols-3">
                                    <CompactInfoTile
                                        label="显示"
                                        value={String(filteredItems.length)}
                                    />
                                    <CompactInfoTile
                                        label="选中"
                                        value={selectedItem?.name ?? "--"}
                                    />
                                    <CompactInfoTile label="排序" value={sortLabel} />
                                </div>
                            </DockCard>
                        </div>
                    </div>
                </div>
            </section>

            {actionError ? (
                <ActionBanner tone="error" title="操作失败" detail={actionError} />
            ) : null}
            {actionMessage ? (
                <ActionBanner tone="success" title="操作完成" detail={actionMessage} />
            ) : null}

            <section className="grid gap-6 xl:grid-cols-[420px_minmax(0,1fr)]">
                <Card className="overflow-hidden rounded-[28px] border-stone-200/80 bg-white/95 shadow-[0_22px_70px_rgba(37,31,17,0.06)]">
                    <CardHeader className="border-b border-stone-200/80 bg-[#fbf9f4] pb-4">
                        <div className="flex items-center justify-between gap-3">
                            <div className="flex items-center gap-3">
                                <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-stone-200 bg-white text-stone-700">
                                    <PackageOpen className="h-4 w-4" />
                                </div>
                                <CardTitle className="text-lg tracking-[-0.02em]">插件目录</CardTitle>
                            </div>
                            <div className="flex flex-wrap items-center gap-2">
                                <Badge variant="outline" className="rounded-full bg-white px-3 py-1 text-[11px]">
                                    Visible {filteredItems.length}
                                </Badge>
                                <Badge variant="outline" className="rounded-full bg-white px-3 py-1 text-[11px]">
                                    Sort {sortLabel}
                                </Badge>
                                {loading ? (
                                    <Loader2 className="h-4 w-4 animate-spin text-stone-500" />
                                ) : null}
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="p-0">
                        <ScrollArea className="h-[72vh] min-h-[600px]">
                            <div className="space-y-0">
                                {filteredItems.length === 0 ? (
                                    <div className="px-6 py-10 text-sm text-stone-500">
                                        {loadError
                                            ? `未加载到外部插件：${loadError}`
                                            : "当前条件下没有可显示的插件。"}
                                    </div>
                                ) : (
                                    filteredItems.map((item, index) => (
                                        <MarketRailItem
                                            key={item.id}
                                            item={item}
                                            selected={item.id === selectedId}
                                            divider={index > 0}
                                            loading={
                                                item.lane === "external" &&
                                                pendingPluginId === item.entry.plugin_id
                                            }
                                            onSelect={() => {
                                                setSelectedId(item.id);
                                                setDetailTab("overview");
                                            }}
                                            onInstall={
                                                item.lane === "external"
                                                    ? () => void installExternalPlugin(item.entry)
                                                    : undefined
                                            }
                                        />
                                    ))
                                )}
                            </div>
                        </ScrollArea>
                    </CardContent>
                </Card>

                <Card className="overflow-hidden rounded-[28px] border-stone-200/80 bg-[linear-gradient(180deg,#fffdf8_0%,#fbf8f0_100%)] shadow-[0_22px_70px_rgba(37,31,17,0.06)]">
                    {selectedItem ? (
                        <MarketplaceDetailPanel
                            item={selectedItem}
                            detailTab={detailTab}
                            setDetailTab={setDetailTab}
                            lastInstall={lastInstall}
                            pendingPluginId={pendingPluginId}
                            runtimePackageRoot={snapshot?.runtime.package_root ?? "--"}
                            runtimeMode={snapshot?.runtime.dev_auth_mode ?? "--"}
                            onInstall={
                                selectedItem.lane === "external"
                                    ? () => void installExternalPlugin(selectedItem.entry)
                                    : undefined
                            }
                            onOpen={
                                selectedItem.lane === "builtin"
                                    ? () => navigate(selectedItem.entry.routeHref)
                                    : undefined
                            }
                            onOpenLatestInstance={
                                lastInstall?.page_ids[0]
                                    ? () =>
                                          navigate(
                                              `/apps/${lastInstall.instance_slug}/${lastInstall.page_ids[0]}`,
                                          )
                                    : undefined
                            }
                        />
                    ) : (
                        <CardContent className="flex min-h-[600px] items-center justify-center">
                            <div className="text-sm text-stone-500">没有选中插件。</div>
                        </CardContent>
                    )}
                </Card>
            </section>
        </div>
    );
}

function MarketplaceDetailPanel({
    item,
    detailTab,
    setDetailTab,
    lastInstall,
    pendingPluginId,
    runtimePackageRoot,
    runtimeMode,
    onInstall,
    onOpen,
    onOpenLatestInstance,
}: {
    item: MarketEntryItem;
    detailTab: DetailTab;
    setDetailTab: (value: DetailTab) => void;
    lastInstall: WasmPluginInstallResult | null;
    pendingPluginId: string | null;
    runtimePackageRoot: string;
    runtimeMode: string;
    onInstall?: () => void;
    onOpen?: () => void;
    onOpenLatestInstance?: () => void;
}) {
    const isBuiltin = item.lane === "builtin";
    const latestForItem =
        item.lane === "external" && lastInstall?.plugin_id === item.entry.plugin_id
            ? lastInstall
            : null;
    const manifestId = isBuiltin ? item.entry.id : item.entry.plugin_id;
    const version = isBuiltin ? "builtin" : item.entry.version;
    const compatibilityCount = isBuiltin ? 0 : item.entry.compatibility.length;
    const capabilityCount = isBuiltin ? 0 : item.entry.capabilities.length;
    const tagCount = isBuiltin ? 0 : item.entry.tags.length;

    return (
        <>
            <CardHeader className="border-b border-stone-200/80 bg-[#fcfaf4] pb-5">
                <div className="flex items-start justify-between gap-4">
                    <div className="space-y-4">
                        <div className="flex flex-wrap items-center gap-2 text-[11px] uppercase tracking-[0.28em] text-stone-500">
                            <CircleDot className="h-3.5 w-3.5" />
                            {isBuiltin ? "Built-in Entry" : "External WASM Plugin"}
                        </div>
                        <div>
                            <h2 className="text-4xl font-semibold tracking-[-0.04em] text-stone-950">
                                {item.name}
                            </h2>
                        </div>
                        <div className="flex flex-wrap gap-2">
                            <StateBadge state={item.state} />
                            {item.chips.map((chip) => (
                                <Badge
                                    key={`${item.id}:${chip}`}
                                    variant="outline"
                                    className="rounded-full bg-white/75 px-3 py-1 text-[11px]"
                                >
                                    {chip}
                                </Badge>
                            ))}
                        </div>
                    </div>

                    <div className="flex shrink-0 flex-wrap gap-2">
                        {isBuiltin ? (
                            <Button type="button" className="rounded-full bg-stone-950 px-5" onClick={onOpen}>
                                <ArrowUpRight className="h-4 w-4" />
                                打开入口
                            </Button>
                        ) : (
                            <>
                                <Button
                                    type="button"
                                    className="rounded-full bg-stone-950 px-5"
                                    onClick={onInstall}
                                    disabled={pendingPluginId === item.entry.plugin_id}
                                >
                                    {pendingPluginId === item.entry.plugin_id ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <PackageOpen className="h-4 w-4" />
                                    )}
                                    {item.entry.status === "Available" ? "安装实例" : "新建实例"}
                                </Button>
                                {latestForItem?.page_ids[0] ? (
                                    <Button
                                        type="button"
                                        variant="outline"
                                        className="rounded-full bg-white/80 px-5"
                                        onClick={onOpenLatestInstance}
                                    >
                                        <ArrowUpRight className="h-4 w-4" />
                                        打开最新实例
                                    </Button>
                                ) : null}
                            </>
                        )}
                    </div>
                </div>

                <div className="mt-6 grid gap-3 lg:grid-cols-5">
                    <MiniPanel
                        label="Scope"
                        value={isBuiltin ? "Host" : "External"}
                        detail={isBuiltin ? "built-in" : "user upload"}
                    />
                    <MiniPanel
                        label="Version"
                        value={version}
                        detail={manifestId}
                    />
                    <MiniPanel
                        label="State"
                        value={item.state}
                        detail={isBuiltin ? "fixed" : `${item.instanceCount} instances`}
                    />
                    <MiniPanel
                        label="Compat"
                        value={String(compatibilityCount)}
                        detail={isBuiltin ? "host route" : "compat matrix"}
                    />
                    <MiniPanel
                        label="Caps"
                        value={String(capabilityCount || tagCount)}
                        detail={isBuiltin ? "shell actions" : "caps / tags"}
                    />
                    <MiniPanel
                        label="Runtime"
                        value={isBuiltin ? "Shell" : "PG/MinIO"}
                        detail={isBuiltin ? "route entry" : "persisted firmware"}
                    />
                </div>
            </CardHeader>

            <CardContent className="space-y-5 p-6">
                <Tabs value={detailTab} onValueChange={(value) => setDetailTab(value as DetailTab)}>
                    <TabsList className="grid h-auto w-full grid-cols-3 rounded-2xl bg-stone-100/80 p-1">
                        <TabsTrigger value="overview" className="rounded-xl py-3">
                            概览
                        </TabsTrigger>
                        <TabsTrigger value="package" className="rounded-xl py-3">
                            包与校验
                        </TabsTrigger>
                        <TabsTrigger value="activity" className="rounded-xl py-3">
                            实例与动作
                        </TabsTrigger>
                    </TabsList>

                    <TabsContent value="overview" className="mt-4">
                        {isBuiltin ? (
                            <BuiltinInteractiveOverview entry={item.entry} />
                        ) : (
                            <ExternalInteractiveOverview entry={item.entry} />
                        )}
                    </TabsContent>

                    <TabsContent value="package" className="mt-4">
                        <div className="grid gap-4 lg:grid-cols-[1.05fr_0.95fr]">
                            <InstrumentPanel
                                title="校验仪表"
                                eyebrow="Verification"
                                icon={<ShieldCheck className="h-4 w-4" />}
                            >
                                <ChecklistRow label="file" state="pass" detail="`.wasm` only" />
                                <ChecklistRow label="metadata" state="pass" detail="stored in PostgreSQL" />
                                <ChecklistRow label="binary" state="pass" detail="stored in MinIO" />
                                <ChecklistRow label="descriptor" state="pass" detail="generated by server" />
                                <ChecklistRow
                                    label="plugin kind"
                                    state={isBuiltin ? "warn" : "pass"}
                                    detail={
                                        isBuiltin
                                            ? "built-in entry bypasses upload flow"
                                            : "System / Business firmware category"
                                    }
                                />
                            </InstrumentPanel>

                            <InstrumentPanel
                                title="入库轨迹"
                                eyebrow="Storage Path"
                                icon={<FolderArchive className="h-4 w-4" />}
                            >
                                <PathLine label="package_root" value={runtimePackageRoot} />
                                <PathLine label="auth_mode" value={runtimeMode} />
                                <PathLine
                                    label="selection"
                                    value={isBuiltin ? item.entry.routeHref : item.entry.plugin_id}
                                />
                            </InstrumentPanel>
                        </div>
                    </TabsContent>

                    <TabsContent value="activity" className="mt-4">
                        <div className="grid gap-4 lg:grid-cols-[0.9fr_1.1fr]">
                            <InstrumentPanel
                                title="动作面板"
                                eyebrow="Action Rail"
                                icon={<Settings2 className="h-4 w-4" />}
                            >
                                {isBuiltin ? (
                                    <>
                                        <ActionTile
                                            title="打开当前入口"
                                            detail="进入内置工作域。"
                                            cta="进入"
                                            onClick={onOpen}
                                        />
                                        <ActionTile
                                            title="切换内置项"
                                            detail="左侧只保留 插件 / 系统 两个内置入口。"
                                            passive
                                        />
                                    </>
                                ) : (
                                    <>
                                        <ActionTile
                                            title={item.entry.status === "Available" ? "安装实例" : "新建实例"}
                                            detail="从 PG 插件元数据直接创建实例。"
                                            cta={item.entry.status === "Available" ? "安装" : "创建"}
                                            onClick={onInstall}
                                            loading={pendingPluginId === item.entry.plugin_id}
                                        />
                                        <ActionTile
                                            title="打开最新实例"
                                            detail={
                                                latestForItem?.page_ids[0]
                                                    ? `${latestForItem.instance_slug}`
                                                    : "暂无最近实例。"
                                            }
                                            cta="打开"
                                            passive={!latestForItem?.page_ids[0]}
                                            onClick={
                                                latestForItem?.page_ids[0]
                                                    ? onOpenLatestInstance
                                                    : undefined
                                            }
                                        />
                                    </>
                                )}
                            </InstrumentPanel>

                            <InstrumentPanel
                                title="最近结果"
                                eyebrow="Latest Run"
                                icon={<Sparkles className="h-4 w-4" />}
                            >
                                {latestForItem ? (
                                    <>
                                        <PathLine label="plugin" value={latestForItem.plugin_name} />
                                        <PathLine label="instance" value={latestForItem.instance_slug} />
                                        <PathLine label="entry page" value={latestForItem.page_ids[0] ?? "--"} />
                                    </>
                                ) : (
                                    <div className="rounded-2xl border border-dashed border-stone-300 bg-white/60 px-4 py-6 text-sm text-stone-500">
                                        暂无最近实例结果。
                                    </div>
                                )}
                            </InstrumentPanel>
                        </div>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </>
    );
}

function BuiltinInteractiveOverview({ entry }: { entry: BuiltinEntry }) {
    return (
        <div className="grid gap-4 lg:grid-cols-[1.02fr_0.98fr]">
            <InstrumentPanel
                title="内置功能面"
                eyebrow="Core Role"
                icon={<Puzzle className="h-4 w-4" />}
            >
                {entry.highlights.map((item) => (
                    <SignalRow key={item} label={item} detail={entry.statusTone} />
                ))}
            </InstrumentPanel>

            <InstrumentPanel
                title="市场边界"
                eyebrow="Boundary"
                icon={<ShieldCheck className="h-4 w-4" />}
            >
                <SignalRow label="Built-in only" detail="市场内置项只保留 插件 / 系统。" />
                <SignalRow label="Firmware split" detail="上传时明确选择系统固件或业务固件。" />
                <SignalRow label="Upload gate" detail="市场不接源码目录导入，只接 `.wasm` 固件。" />
                <SignalRow label="Storage path" detail="元数据写 PostgreSQL，二进制写 MinIO。" />
            </InstrumentPanel>
        </div>
    );
}

function ExternalInteractiveOverview({
    entry,
}: {
    entry: WasmPluginMarketplaceEntry;
}) {
    return (
        <div className="grid gap-4 lg:grid-cols-[0.92fr_1.08fr]">
            <InstrumentPanel
                title="能力矩阵"
                eyebrow="Capabilities"
                icon={<Blocks className="h-4 w-4" />}
            >
                {entry.capabilities.length > 0 ? (
                    entry.capabilities.map((capability) => (
                        <SignalRow
                            key={String(capability)}
                            label={String(capability)}
                            detail="descriptor declared"
                        />
                    ))
                ) : (
                    <SignalRow label="未声明 capability" detail="no extra host capability" />
                )}
            </InstrumentPanel>

            <InstrumentPanel
                title="兼容 / 标签"
                eyebrow="Metadata"
                icon={<ShieldCheck className="h-4 w-4" />}
            >
                {entry.compatibility.length > 0 ? (
                    entry.compatibility.map((item) => (
                        <SignalRow key={item} label={item} detail="compat target" />
                    ))
                ) : (
                    <SignalRow label="未声明 compatibility" detail="compat metadata missing" />
                )}
                {entry.tags.length > 0 ? (
                    entry.tags.map((tag) => (
                        <SignalRow key={tag} label={`#${tag}`} detail="search / grouping" />
                    ))
                ) : (
                    <SignalRow label="无标签" detail="tags missing" />
                )}
            </InstrumentPanel>
        </div>
    );
}

function MarketRailItem({
    item,
    selected,
    divider,
    loading,
    onSelect,
    onInstall,
}: {
    item: MarketEntryItem;
    selected: boolean;
    divider: boolean;
    loading: boolean;
    onSelect: () => void;
    onInstall?: () => void;
}) {
    return (
        <div
            className={cn(
                "px-5 py-4 transition",
                divider && "border-t border-stone-200/70",
                selected
                    ? "bg-[linear-gradient(90deg,#f4ecdb_0%,#fbf8f1_60%,#ffffff_100%)]"
                    : "bg-white hover:bg-stone-50/80",
            )}
        >
            <div className="flex items-start gap-3">
                <button type="button" onClick={onSelect} className="min-w-0 flex-1 text-left">
                    <div className="flex items-center gap-3">
                        <div
                            className={cn(
                                "flex h-11 w-11 items-center justify-center rounded-2xl border",
                                selected
                                    ? "border-stone-950 bg-stone-950 text-white"
                                    : "border-stone-200 bg-stone-100 text-stone-600",
                            )}
                        >
                            {item.lane === "builtin" ? (
                                <Puzzle className="h-4 w-4" />
                            ) : (
                                <PackageOpen className="h-4 w-4" />
                            )}
                        </div>
                        <div className="min-w-0 flex-1">
                            <div className="flex items-center justify-between gap-2">
                                <div className="truncate text-sm font-semibold text-stone-950">
                                    {item.name}
                                </div>
                                <StateBadge state={item.state} compact />
                            </div>
                            <div className="mt-2 grid grid-cols-3 gap-2 text-[11px] uppercase tracking-[0.14em] text-stone-400">
                                <RailMiniStat label="lane" value={item.lane} />
                                <RailMiniStat
                                    label="instances"
                                    value={String(item.instanceCount)}
                                />
                                <RailMiniStat
                                    label="mode"
                                    value={item.lane === "builtin" ? "route" : "wasm"}
                                />
                            </div>
                            <p className="mt-2 line-clamp-2 text-sm leading-6 text-stone-500">
                                {item.summary}
                            </p>
                        </div>
                    </div>
                    <div className="mt-3 flex flex-wrap gap-2">
                        {item.chips.slice(0, 3).map((chip) => (
                            <Badge
                                key={`${item.id}:${chip}`}
                                variant="outline"
                                className="rounded-full bg-white/80 px-2.5 py-1 text-[11px]"
                            >
                                {chip}
                            </Badge>
                        ))}
                    </div>
                </button>

                {item.lane === "external" ? (
                    <div className="flex shrink-0 flex-col gap-2">
                        <Button
                            type="button"
                            size="sm"
                            className="rounded-full bg-stone-950 px-3"
                            onClick={onInstall}
                            disabled={loading}
                        >
                            {loading ? (
                                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                            ) : (
                                <ChevronRight className="h-3.5 w-3.5" />
                            )}
                            {item.entry.status === "Available" ? "安装" : "实例"}
                        </Button>
                        <div className="text-center text-[11px] text-stone-400">
                            {item.instanceCount}
                        </div>
                    </div>
                ) : null}
            </div>
        </div>
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
        </div>
    );
}

function SegmentGroup({
    label,
    value,
    options,
    onChange,
}: {
    label: string;
    value: string;
    options: { id: string; label: string }[];
    onChange: (value: string) => void;
}) {
    return (
        <div className="space-y-2">
            <div className="text-[11px] uppercase tracking-[0.18em] text-stone-500">{label}</div>
            <div className="flex flex-wrap gap-2">
                {options.map((option) => (
                    <button
                        key={option.id}
                        type="button"
                        onClick={() => onChange(option.id)}
                        className={cn(
                            "rounded-full border px-3 py-1.5 text-xs font-medium transition",
                            value === option.id
                                ? "border-stone-950 bg-stone-950 text-white"
                                : "border-stone-200 bg-white/80 text-stone-600 hover:bg-stone-50",
                        )}
                    >
                        {option.label}
                    </button>
                ))}
            </div>
        </div>
    );
}

function DockCard({
    eyebrow,
    title,
    icon,
    children,
}: {
    eyebrow: string;
    title: string;
    icon: ReactNode;
    children: ReactNode;
}) {
    return (
        <div className="rounded-[24px] border border-stone-200/80 bg-white/70 p-5 shadow-[0_12px_34px_rgba(43,37,21,0.05)]">
            <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.24em] text-stone-500">
                {icon}
                {eyebrow}
            </div>
            <div className="mt-3 text-lg font-semibold tracking-[-0.02em] text-stone-950">
                {title}
            </div>
            <div className="mt-4 space-y-4">{children}</div>
        </div>
    );
}

function QuickMetricButton({
    label,
    value,
    detail,
    active,
    onClick,
}: {
    label: string;
    value: string;
    detail: string;
    active: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={cn(
                "rounded-[22px] border px-4 py-4 text-left transition",
                active
                    ? "border-stone-950 bg-stone-950 text-white shadow-[0_16px_40px_rgba(30,26,18,0.16)]"
                    : "border-stone-200/80 bg-white/75 text-stone-950 hover:bg-white",
            )}
        >
            <div
                className={cn(
                    "text-[11px] uppercase tracking-[0.18em]",
                    active ? "text-stone-300" : "text-stone-500",
                )}
            >
                {label}
            </div>
            <div className="mt-2 text-2xl font-semibold tracking-[-0.03em]">{value}</div>
            <div className={cn("mt-2 text-sm", active ? "text-stone-200" : "text-stone-500")}>
                {detail}
            </div>
        </button>
    );
}

function CenterFilterSpine({
    active,
    onSelect,
}: {
    active: "all" | "builtin" | "external" | "installed" | "available";
    onSelect: (value: "all" | "builtin" | "external" | "installed" | "available") => void;
}) {
    const nodes = [
        { id: "all", label: "ALL" },
        { id: "builtin", label: "BI" },
        { id: "external", label: "EXT" },
        { id: "installed", label: "IN" },
        { id: "available", label: "AVL" },
    ] as const;

    return (
        <div className="hidden xl:flex xl:flex-col xl:items-center xl:justify-center">
            <div className="flex h-full min-h-[420px] w-full flex-col items-center justify-center rounded-[28px] border border-stone-200/70 bg-white/45 px-4 py-5">
                <div className="text-[10px] uppercase tracking-[0.32em] text-stone-500">
                    Rail
                </div>
                <div className="mt-5 flex h-full flex-col items-center justify-center gap-3">
                    {nodes.map((node) => (
                        <button
                            key={node.id}
                            type="button"
                            onClick={() => onSelect(node.id)}
                            className={cn(
                                "flex h-14 w-14 items-center justify-center rounded-full border text-[11px] font-semibold tracking-[0.2em] transition",
                                active === node.id
                                    ? "border-stone-950 bg-stone-950 text-white"
                                    : "border-stone-200 bg-white/80 text-stone-500 hover:bg-white",
                            )}
                        >
                            {node.label}
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
}

function HeroStat({
    label,
    value,
    detail,
}: {
    label: string;
    value: string;
    detail: string;
}) {
    return (
        <div className="rounded-[22px] border border-stone-200/80 bg-white/75 px-4 py-4">
            <div className="text-[11px] uppercase tracking-[0.18em] text-stone-500">{label}</div>
            <div className="mt-2 text-2xl font-semibold tracking-[-0.03em] text-stone-950">
                {value}
            </div>
            <div className="mt-2 text-sm text-stone-500">{detail}</div>
        </div>
    );
}

function MiniPanel({
    label,
    value,
    detail,
}: {
    label: string;
    value: string;
    detail: string;
}) {
    return (
        <div className="rounded-[20px] border border-stone-200 bg-white/80 px-4 py-4">
            <div className="text-[11px] uppercase tracking-[0.18em] text-stone-500">{label}</div>
            <div className="mt-2 text-lg font-semibold text-stone-950">{value}</div>
            <div className="mt-1 text-sm text-stone-500">{detail}</div>
        </div>
    );
}

function CompactInfoTile({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-2xl border border-stone-200 bg-[#fffdfa] px-4 py-3">
            <div className="text-[11px] uppercase tracking-[0.18em] text-stone-500">{label}</div>
            <div className="mt-2 truncate text-sm font-medium text-stone-950">{value}</div>
        </div>
    );
}

function InstrumentPanel({
    eyebrow,
    title,
    icon,
    children,
}: {
    eyebrow: string;
    title: string;
    icon: ReactNode;
    children: ReactNode;
}) {
    return (
        <div className="rounded-[24px] border border-stone-200/80 bg-white/80 p-5 shadow-[0_10px_30px_rgba(43,37,21,0.04)]">
            <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.22em] text-stone-500">
                {icon}
                {eyebrow}
            </div>
            <div className="mt-3 text-lg font-semibold tracking-[-0.02em] text-stone-950">
                {title}
            </div>
            <div className="mt-4 space-y-3">{children}</div>
        </div>
    );
}

function SignalRow({ label, detail }: { label: string; detail: string }) {
    return (
        <div className="rounded-2xl border border-stone-200 bg-[#fffdfa] px-4 py-3">
            <div className="text-sm font-medium text-stone-950">{label}</div>
            <div className="mt-1 text-sm text-stone-500">{detail}</div>
        </div>
    );
}

function RailMiniStat({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-xl border border-stone-200 bg-white/75 px-2 py-2">
            <div className="truncate">{label}</div>
            <div className="mt-1 truncate text-stone-700">{value}</div>
        </div>
    );
}

function ChecklistRow({
    label,
    state,
    detail,
}: {
    label: string;
    state: "pass" | "warn";
    detail: string;
}) {
    return (
        <div className="flex items-start justify-between gap-4 rounded-2xl border border-stone-200 bg-[#fffdfa] px-4 py-3">
            <div>
                <div className="text-sm font-medium text-stone-950">{label}</div>
                <div className="mt-1 text-sm text-stone-500">{detail}</div>
            </div>
            <Badge
                variant={state === "pass" ? "secondary" : "outline"}
                className={cn(
                    "rounded-full px-3 py-1 text-[11px]",
                    state === "pass" && "bg-emerald-100 text-emerald-800",
                )}
            >
                {state === "pass" ? "PASS" : "WARN"}
            </Badge>
        </div>
    );
}

function PathLine({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-2xl border border-stone-200 bg-[#fffdfa] px-4 py-3">
            <div className="text-[11px] uppercase tracking-[0.18em] text-stone-500">{label}</div>
            <div className="mt-2 break-all font-mono text-xs text-stone-700">{value}</div>
        </div>
    );
}

function ActionTile({
    title,
    detail,
    cta,
    onClick,
    loading,
    passive,
}: {
    title: string;
    detail: string;
    cta?: string;
    onClick?: () => void;
    loading?: boolean;
    passive?: boolean;
}) {
    return (
        <div className="rounded-2xl border border-stone-200 bg-[#fffdfa] px-4 py-4">
            <div className="flex items-start justify-between gap-4">
                <div>
                    <div className="text-sm font-medium text-stone-950">{title}</div>
                    <div className="mt-1 text-sm text-stone-500">{detail}</div>
                </div>
                {cta ? (
                    <Button
                        type="button"
                        size="sm"
                        variant={passive ? "outline" : "default"}
                        className={cn(
                            "rounded-full",
                            !passive && "bg-stone-950 px-3",
                            passive && "bg-white",
                        )}
                        onClick={onClick}
                        disabled={passive || loading}
                    >
                        {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
                        {cta}
                    </Button>
                ) : null}
            </div>
        </div>
    );
}

function ActionBanner({
    tone,
    title,
    detail,
}: {
    tone: "success" | "error";
    title: string;
    detail: string;
}) {
    return (
        <div
            className={cn(
                "rounded-[22px] border px-5 py-4",
                tone === "success"
                    ? "border-emerald-300 bg-emerald-50 text-emerald-900"
                    : "border-rose-300 bg-rose-50 text-rose-900",
            )}
        >
            <div className="flex items-start gap-3">
                {tone === "success" ? (
                    <CheckCircle2 className="mt-0.5 h-4 w-4" />
                ) : (
                    <CircleDot className="mt-0.5 h-4 w-4" />
                )}
                <div>
                    <div className="text-sm font-semibold">{title}</div>
                    <div className="mt-1 text-sm opacity-90">{detail}</div>
                </div>
            </div>
        </div>
    );
}

function StateBadge({
    state,
    compact,
}: {
    state: "Builtin" | "Installed" | "Available" | "Disabled";
    compact?: boolean;
}) {
    const tone =
        state === "Installed"
            ? "bg-emerald-100 text-emerald-800 border-emerald-200"
            : state === "Available"
              ? "bg-amber-100 text-amber-800 border-amber-200"
              : state === "Disabled"
                ? "bg-stone-200 text-stone-700 border-stone-300"
                : "bg-stone-950 text-white border-stone-950";
    return (
        <span
            className={cn(
                "inline-flex items-center rounded-full border px-2.5 py-1 text-xs font-semibold",
                tone,
                compact && "px-2 py-0.5 text-[11px]",
            )}
        >
            {state}
        </span>
    );
}
