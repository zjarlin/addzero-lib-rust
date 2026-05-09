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
    createApiClient,
    getApiBaseUrl,
    type CliSimpleMetadata,
} from "@az/api-client";
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
    registerCloudflareTunnelPlugin,
    registerNotesFragmentsPlugin,
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
                detail: "插件工件为 `.wasm`，元数据写入 PG，资源写入 MinIO。",
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

const cliSimpleMetadataFields = [
    "name",
    "display_name",
    "version",
    "description",
    "requires",
    "install_cmd",
    "entry_point",
    "category",
] as const;

const cliSimpleMetadataExample: CliSimpleMetadata = {
    name: "gimp",
    display_name: "GIMP",
    version: "1.0.0",
    description: "Raster image processing via gimp -i -b (batch mode)",
    requires: "gimp (apt install gimp)",
    install_cmd:
        "pip install git+https://github.com/HKUDS/CLI-Anything.git#subdirectory=gimp/agent-harness",
    entry_point: "cli-anything-gimp",
    category: "image",
};

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

interface MarketPageProps {
    forcedScene?: MarketScene;
}

export default function MarketPage({ forcedScene }: MarketPageProps = {}) {
    const params = useParams<{ scene?: string }>();
    const routeScene: MarketScene =
        params.scene === "cli" || params.scene === "skill" ? params.scene : "wasm";
    const scene: MarketScene = forcedScene ?? routeScene;

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
    const [registeringCloudflare, setRegisteringCloudflare] = useState(false);
    const [registeringNotes, setRegisteringNotes] = useState(false);
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

    async function registerCloudflarePlugin() {
        setRegisteringCloudflare(true);
        setActionError(null);
        setActionMessage(null);
        try {
            const result = await registerCloudflareTunnelPlugin();
            setActionMessage(
                `已写入 PG/MinIO：${result.plugin_name} ${result.version} (${result.plugin_id})`,
            );
            setLaneFilter("external");
            setStatusFilter("available");
            setSelectedId(`external:${result.plugin_id}`);
            setDetailTab("package");
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await loadSnapshot();
        } catch (err) {
            setActionError(err instanceof Error ? err.message : "写入 Cloudflare Tunnel 插件失败");
        } finally {
            setRegisteringCloudflare(false);
        }
    }

    async function registerNotesPlugin() {
        setRegisteringNotes(true);
        setActionError(null);
        setActionMessage(null);
        try {
            const result = await registerNotesFragmentsPlugin();
            setLastInstall(result);
            setActionMessage(
                `已创建实例：${result.instance_label} (${result.instance_slug})`,
            );
            window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
            await loadSnapshot();
            navigate(`/apps/${result.instance_slug}/${result.page_ids[0] ?? "fragments"}`);
        } catch (err) {
            setActionError(err instanceof Error ? err.message : "注册碎片笔记插件失败");
        } finally {
            setRegisteringNotes(false);
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
                                    onClick={() => void registerCloudflarePlugin()}
                                    disabled={registeringCloudflare}
                                >
                                    {registeringCloudflare ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <PackageOpen className="h-4 w-4" />
                                    )}
                                    Cloudflare Tunnel
                                </Button>
                                <Button
                                    type="button"
                                    variant="outline"
                                    className="rounded-full bg-white/80 px-5"
                                    onClick={() => void registerNotesPlugin()}
                                    disabled={registeringNotes}
                                >
                                    {registeringNotes ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <PackageOpen className="h-4 w-4" />
                                    )}
                                    碎片笔记
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
                                title="Cloudflare Tunnel"
                                eyebrow="DB-first Plugin"
                                icon={<PackageOpen className="h-4 w-4" />}
                            >
                                <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
                                    <div className="space-y-2">
                                        <div className="text-sm font-medium text-stone-900">
                                            写入 tunnel 插件元数据和 CLI 资源
                                        </div>
                                        <p className="text-sm text-stone-500">
                                            `.wasm` 作为插件工件进入 MinIO；addhost、showhost、rmhost、autohost 作为 CLI 资源入库，安装实例时发布到本机 PATH。
                                        </p>
                                    </div>
                                    <Button
                                        type="button"
                                        className="rounded-full bg-stone-950 px-5"
                                        onClick={() => void registerCloudflarePlugin()}
                                        disabled={registeringCloudflare}
                                    >
                                        {registeringCloudflare ? (
                                            <Loader2 className="h-4 w-4 animate-spin" />
                                        ) : (
                                            <PackageOpen className="h-4 w-4" />
                                        )}
                                        写入
                                    </Button>
                                </div>
                                <div className="grid gap-2 sm:grid-cols-3">
                                    <CompactInfoTile label="工件" value=".wasm" />
                                    <CompactInfoTile label="元数据" value="PostgreSQL" />
                                    <CompactInfoTile label="资源" value="MinIO CLI" />
                                </div>
                            </DockCard>

                            <DockCard
                                title="碎片笔记"
                                eyebrow="DB-first Plugin"
                                icon={<PackageOpen className="h-4 w-4" />}
                            >
                                <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
                                    <div className="space-y-2">
                                        <div className="text-sm font-medium text-stone-900">
                                            注册 notes-fragments.wasm 并打开实例
                                        </div>
                                        <p className="text-sm text-stone-500">
                                            笔记不再是内置资产页；平台只负责注册 `.wasm` 插件、保存 PG 元数据和 MinIO 工件，碎片流由插件页面承载。
                                        </p>
                                    </div>
                                    <Button
                                        type="button"
                                        className="rounded-full bg-stone-950 px-5"
                                        onClick={() => void registerNotesPlugin()}
                                        disabled={registeringNotes}
                                    >
                                        {registeringNotes ? (
                                            <Loader2 className="h-4 w-4 animate-spin" />
                                        ) : (
                                            <PackageOpen className="h-4 w-4" />
                                        )}
                                        打开
                                    </Button>
                                </div>
                                <div className="grid gap-2 sm:grid-cols-3">
                                    <CompactInfoTile label="入口" value="/apps/:slug/fragments" />
                                    <CompactInfoTile label="schema" value="notes_fragments" />
                                    <CompactInfoTile label="整理" value="后续独立页面" />
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
    if (scene === "cli") {
        return <CliSimpleMetadataPage />;
    }

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

function CliSimpleMetadataPage() {
    const api = useMemo(() => createApiClient(getApiBaseUrl()), []);
    const [entries, setEntries] = useState<CliSimpleMetadata[]>([]);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [search, setSearch] = useState("");
    const [selectedName, setSelectedName] = useState<string | null>(null);
    const [draftText, setDraftText] = useState(
        formatCliSimpleMetadata(cliSimpleMetadataExample),
    );
    const [message, setMessage] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const loadEntries = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const nextEntries = await api.cliSimpleCatalog();
            setEntries(nextEntries);
            setSelectedName((current) => {
                if (current && nextEntries.some((entry) => entry.name === current)) {
                    return current;
                }
                return nextEntries[0]?.name ?? null;
            });
            if (nextEntries.length === 0) {
                setDraftText(formatCliSimpleMetadata(cliSimpleMetadataExample));
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : "CLI 元数据加载失败");
        } finally {
            setLoading(false);
        }
    }, [api]);

    useEffect(() => {
        void loadEntries();
    }, [loadEntries]);

    const selectedEntry = useMemo(
        () => entries.find((entry) => entry.name === selectedName) ?? null,
        [entries, selectedName],
    );

    useEffect(() => {
        if (selectedEntry) {
            setDraftText(formatCliSimpleMetadata(selectedEntry));
        }
    }, [selectedEntry]);

    const filteredEntries = useMemo(() => {
        const keyword = search.trim().toLowerCase();
        if (!keyword) {
            return entries;
        }
        return entries.filter((entry) =>
            [
                entry.name,
                entry.display_name,
                entry.version,
                entry.description,
                entry.requires,
                entry.install_cmd,
                entry.entry_point,
                entry.category,
            ]
                .join("\n")
                .toLowerCase()
                .includes(keyword),
        );
    }, [entries, search]);

    const categories = useMemo(() => {
        const values = new Set(
            entries
                .map((entry) => entry.category.trim())
                .filter((category) => category.length > 0),
        );
        return [...values].sort((left, right) =>
            left.localeCompare(right, "zh-Hans-CN"),
        );
    }, [entries]);

    async function saveMetadata() {
        setSaving(true);
        setError(null);
        setMessage(null);
        try {
            const input = parseCliSimpleMetadataJson(draftText);
            const saved = await api.cliSimpleUpsert(input);
            setMessage(`已保存 ${saved.name}`);
            setDraftText(formatCliSimpleMetadata(saved));
            await loadEntries();
            setSelectedName(saved.name);
        } catch (err) {
            setError(err instanceof Error ? err.message : "CLI 元数据保存失败");
        } finally {
            setSaving(false);
        }
    }

    function useExample() {
        setSelectedName(null);
        setDraftText(formatCliSimpleMetadata(cliSimpleMetadataExample));
        setMessage("已载入 GIMP 示例");
        setError(null);
    }

    return (
        <div className="space-y-6">
            <section className="overflow-hidden rounded-[28px] border border-stone-200/80 bg-[linear-gradient(135deg,#fff8e8_0%,#f5f7ef_48%,#eef6ff_100%)] shadow-[0_24px_80px_rgba(35,35,25,0.08)]">
                <div className="px-6 py-6">
                    <div className="flex flex-wrap items-center justify-between gap-3">
                        <div className="flex items-center gap-2 text-[11px] font-medium uppercase tracking-[0.24em] text-stone-500">
                            <PackageOpen className="h-3.5 w-3.5" />
                            Agent Asset / CLI
                        </div>
                        <div className="flex flex-wrap gap-2">
                            <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                8 字段合同
                            </Badge>
                            <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                PostgreSQL
                            </Badge>
                            <Badge variant="outline" className="rounded-full bg-white/80 px-3 py-1 text-[11px]">
                                Desktop-only install
                            </Badge>
                        </div>
                    </div>
                    <div className="mt-5 grid gap-5 lg:grid-cols-[minmax(0,1fr)_320px]">
                        <div>
                            <h1 className="text-3xl font-semibold tracking-[-0.045em] text-stone-950 sm:text-4xl">
                                CLI 元数据只收这 8 个字段
                            </h1>
                            <p className="mt-3 max-w-3xl text-sm leading-6 text-stone-600">
                                贡献者提交的 JSON 必须完全匹配示例字段；安装执行仍属于桌面端本机能力，Web
                                侧不承载本机执行逻辑。
                            </p>
                        </div>
                        <div className="grid grid-cols-3 gap-2">
                            <MetricTile label="CLI" value={String(entries.length)} />
                            <MetricTile label="分类" value={String(categories.length)} />
                            <MetricTile label="合同" value="8" />
                        </div>
                    </div>
                </div>
            </section>

            <div className="grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)]">
                <Card className="overflow-hidden rounded-[24px] border-stone-200/80 bg-white/95 shadow-[0_18px_54px_rgba(37,31,17,0.06)]">
                    <CardHeader className="border-b border-stone-200/80 bg-[#fbfaf5] pb-4">
                        <div className="flex items-center justify-between gap-3">
                            <div>
                                <CardTitle className="text-base tracking-[-0.02em]">
                                    CLI 目录
                                </CardTitle>
                                <p className="mt-1 text-xs text-muted-foreground">
                                    字段来源只取 simple metadata
                                </p>
                            </div>
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                className="rounded-full"
                                onClick={() => void loadEntries()}
                                disabled={loading}
                            >
                                {loading ? (
                                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                                ) : (
                                    <RefreshCw className="h-3.5 w-3.5" />
                                )}
                                刷新
                            </Button>
                        </div>
                        <div className="relative mt-4">
                            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-stone-400" />
                            <Input
                                value={search}
                                onChange={(event) => setSearch(event.target.value)}
                                placeholder="搜索 name / category / command"
                                className="rounded-full border-stone-200 bg-white pl-9"
                            />
                        </div>
                    </CardHeader>
                    <CardContent className="p-0">
                        <ScrollArea className="h-[560px]">
                            <div className="space-y-2 p-3">
                                {filteredEntries.map((entry) => (
                                    <button
                                        key={entry.name}
                                        type="button"
                                        onClick={() => setSelectedName(entry.name)}
                                        className={cn(
                                            "w-full rounded-2xl border p-4 text-left transition",
                                            selectedName === entry.name
                                                ? "border-stone-950 bg-stone-950 text-white shadow-[0_12px_28px_rgba(28,25,18,0.18)]"
                                                : "border-stone-200 bg-white hover:border-stone-300 hover:bg-stone-50",
                                        )}
                                    >
                                        <div className="flex items-center justify-between gap-3">
                                            <div className="min-w-0">
                                                <div className="truncate text-sm font-semibold">
                                                    {entry.display_name || entry.name}
                                                </div>
                                                <div
                                                    className={cn(
                                                        "mt-1 truncate text-xs",
                                                        selectedName === entry.name
                                                            ? "text-stone-300"
                                                            : "text-stone-500",
                                                    )}
                                                >
                                                    {entry.name} · {entry.version}
                                                </div>
                                            </div>
                                            <Badge
                                                variant="outline"
                                                className={cn(
                                                    "shrink-0 rounded-full text-[10px]",
                                                    selectedName === entry.name
                                                        ? "border-white/30 bg-white/10 text-white"
                                                        : "bg-white",
                                                )}
                                            >
                                                {entry.category}
                                            </Badge>
                                        </div>
                                        <p
                                            className={cn(
                                                "mt-3 line-clamp-2 text-xs leading-5",
                                                selectedName === entry.name
                                                    ? "text-stone-200"
                                                    : "text-stone-600",
                                            )}
                                        >
                                            {entry.description}
                                        </p>
                                    </button>
                                ))}
                                {!loading && filteredEntries.length === 0 ? (
                                    <div className="rounded-2xl border border-dashed border-stone-200 p-5 text-center text-sm text-muted-foreground">
                                        暂无匹配 CLI 元数据
                                    </div>
                                ) : null}
                            </div>
                        </ScrollArea>
                    </CardContent>
                </Card>

                <Card className="overflow-hidden rounded-[24px] border-stone-200/80 bg-white/95 shadow-[0_18px_54px_rgba(37,31,17,0.06)]">
                    <CardHeader className="border-b border-stone-200/80 bg-[#fffdf8] pb-4">
                        <div className="flex flex-wrap items-center justify-between gap-3">
                            <div>
                                <CardTitle className="text-base tracking-[-0.02em]">
                                    贡献者 JSON
                                </CardTitle>
                                <p className="mt-1 text-xs text-muted-foreground">
                                    不允许嵌套字段，不允许额外字段，字段值均为字符串
                                </p>
                            </div>
                            <div className="flex flex-wrap gap-2">
                                <Button
                                    type="button"
                                    variant="outline"
                                    className="rounded-full"
                                    onClick={useExample}
                                >
                                    GIMP 示例
                                </Button>
                                <Button
                                    type="button"
                                    className="rounded-full bg-stone-950 px-5"
                                    onClick={() => void saveMetadata()}
                                    disabled={saving}
                                >
                                    {saving ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <CheckCircle2 className="h-4 w-4" />
                                    )}
                                    保存
                                </Button>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-4 p-5">
                        <Textarea
                            value={draftText}
                            onChange={(event) => setDraftText(event.target.value)}
                            spellCheck={false}
                            className="min-h-[420px] resize-y rounded-2xl border-stone-200 bg-[#11130f] font-mono text-sm leading-6 text-stone-50 shadow-inner placeholder:text-stone-500"
                        />
                        {error ? (
                            <div className="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                                {error}
                            </div>
                        ) : null}
                        {message ? (
                            <div className="rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
                                {message}
                            </div>
                        ) : null}
                        <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-4">
                            {cliSimpleMetadataFields.map((field) => (
                                <div
                                    key={field}
                                    className="rounded-2xl border border-stone-200 bg-stone-50/80 px-3 py-2"
                                >
                                    <div className="font-mono text-[12px] text-stone-900">
                                        {field}
                                    </div>
                                    <div className="mt-1 text-[11px] text-stone-500">
                                        string required
                                    </div>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}

function MetricTile({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-2xl border border-white/70 bg-white/75 p-4 text-center shadow-[0_14px_34px_rgba(34,34,22,0.06)]">
            <div className="text-2xl font-semibold tracking-[-0.04em] text-stone-950">
                {value}
            </div>
            <div className="mt-1 text-[11px] uppercase tracking-[0.18em] text-stone-500">
                {label}
            </div>
        </div>
    );
}

function formatCliSimpleMetadata(input: CliSimpleMetadata) {
    return JSON.stringify(
        {
            name: input.name,
            display_name: input.display_name,
            version: input.version,
            description: input.description,
            requires: input.requires,
            install_cmd: input.install_cmd,
            entry_point: input.entry_point,
            category: input.category,
        },
        null,
        2,
    );
}

function parseCliSimpleMetadataJson(value: string): CliSimpleMetadata {
    let parsed: unknown;
    try {
        parsed = JSON.parse(value);
    } catch (err) {
        throw new Error(err instanceof Error ? `JSON 解析失败：${err.message}` : "JSON 解析失败");
    }
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        throw new Error("CLI 元数据必须是单个 JSON object");
    }

    const object = parsed as Record<string, unknown>;
    const allowed = new Set<string>(cliSimpleMetadataFields);
    const extraFields = Object.keys(object).filter((field) => !allowed.has(field));
    if (extraFields.length > 0) {
        throw new Error(`只允许 8 个字段，多余字段：${extraFields.join(", ")}`);
    }

    const missingFields = cliSimpleMetadataFields.filter(
        (field) => !Object.prototype.hasOwnProperty.call(object, field),
    );
    if (missingFields.length > 0) {
        throw new Error(`缺少字段：${missingFields.join(", ")}`);
    }

    const metadata = {} as CliSimpleMetadata;
    for (const field of cliSimpleMetadataFields) {
        const fieldValue = object[field];
        if (typeof fieldValue !== "string") {
            throw new Error(`字段 ${field} 必须是字符串`);
        }
        const trimmed = fieldValue.trim();
        if (!trimmed) {
            throw new Error(`字段 ${field} 不能为空`);
        }
        metadata[field] = trimmed;
    }
    return metadata;
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
