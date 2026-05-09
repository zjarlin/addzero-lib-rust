import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import {
    type ColumnDef,
    flexRender,
    getCoreRowModel,
    getFilteredRowModel,
    getPaginationRowModel,
    getSortedRowModel,
    type SortingState,
    useReactTable,
} from "@tanstack/react-table";
import {
    Archive,
    ArrowUpDown,
    Boxes,
    CheckCircle2,
    Database,
    FileArchive,
    FileText,
    Filter,
    FolderTree,
    Layers3,
    Loader2,
    MoreHorizontal,
    PackageOpen,
    Plus,
    Search,
    Settings2,
    ShieldCheck,
    SlidersHorizontal,
    UploadCloud,
} from "lucide-react";
import {
    Badge,
    Button,
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    Input,
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
    cn,
} from "@az/ui";
import { registerNotesFragmentsPlugin } from "../lib/wasm-plugin-runtime";

type AssetModuleId =
    | "notes"
    | "packages"
    | "dotfiles";

interface AssetMetric {
    label: string;
    value: string;
    detail: string;
}

interface AssetModule {
    id: AssetModuleId;
    title: string;
    eyebrow: string;
    detail: string;
    status: string;
    icon: ReactNode;
    accent: string;
    responsibilities: string[];
    metrics: AssetMetric[];
}

interface AssetRecord {
    name: string;
    type: string;
    owner: string;
    status: string;
    updated: string;
    size: string;
    refs: number;
    location: string;
}

const MODULES: AssetModule[] = [
    {
        id: "notes",
        title: "笔记",
        eyebrow: "Notes",
        detail: "沉淀人工笔记、任务记录、知识来源和后续可检索上下文。",
        status: "优先",
        icon: <FileText className="h-4 w-4" />,
        accent: "from-amber-400/20 via-stone-50 to-stone-50",
        responsibilities: ["Markdown", "来源追踪", "标签", "检索索引"],
        metrics: [
            { label: "闪念", value: "286", detail: "quick notes" },
            { label: "标签", value: "42", detail: "semantic tags" },
            { label: "附件", value: "19", detail: "linked assets" },
            { label: "碎片", value: "11", detail: "raw fragments" },
        ],
    },
    {
        id: "packages",
        title: "安装包",
        eyebrow: "Packages",
        detail: "维护本地安装包、CLI 包、桌面包、插件包和版本发布记录。",
        status: "优先",
        icon: <PackageOpen className="h-4 w-4" />,
        accent: "from-blue-500/15 via-stone-50 to-stone-50",
        responsibilities: ["版本", "校验和", "发布目标", "安装记录"],
        metrics: [
            { label: "包", value: "18", detail: "tracked artifacts" },
            { label: "草稿", value: "5", detail: "release queue" },
            { label: "校验", value: "91%", detail: "checksum coverage" },
            { label: "主机", value: "3", detail: "install targets" },
        ],
    },
    {
        id: "dotfiles",
        title: "dotfiles",
        eyebrow: "Dotfiles",
        detail: "把 shell、编辑器、工具链和机器初始化配置纳入资产治理。",
        status: "优先",
        icon: <Settings2 className="h-4 w-4" />,
        accent: "from-orange-500/15 via-stone-50 to-stone-50",
        responsibilities: ["配置同步", "机器画像", "差异检查", "恢复动作"],
        metrics: [
            { label: "配置", value: "64", detail: "tracked entries" },
            { label: "漂移", value: "7", detail: "needs review" },
            { label: "恢复点", value: "12", detail: "snapshots" },
            { label: "机器", value: "3", detail: "host profiles" },
        ],
    },
];

const PATH_TO_MODULE: Array<[string, AssetModuleId]> = [
    ["/assets/notes", "notes"],
    ["/assets/packages", "packages"],
    ["/assets/dotfiles", "dotfiles"],
];

const ASSET_RECORDS: Record<AssetModuleId, AssetRecord[]> = {
    notes: [],
    packages: [
        {
            name: "aio-desktop.dmg",
            type: "Desktop",
            owner: "release",
            status: "Draft",
            updated: "2026-05-07",
            size: "88 MB",
            refs: 4,
            location: "dist/mac",
        },
        {
            name: "codex-skill-pack.tgz",
            type: "Skill Package",
            owner: "agent-assets",
            status: "Ready",
            updated: "2026-05-06",
            size: "740 KB",
            refs: 12,
            location: "dist/skills",
        },
        {
            name: "aio-cli-aarch64-apple-darwin.tar.gz",
            type: "CLI",
            owner: "runtime",
            status: "Indexed",
            updated: "2026-05-05",
            size: "12 MB",
            refs: 7,
            location: "dist/cli",
        },
        {
            name: "market-index.sqlite",
            type: "Registry",
            owner: "plugins",
            status: "Review",
            updated: "2026-05-04",
            size: "4.1 MB",
            refs: 3,
            location: "dist/market",
        },
    ],
    dotfiles: [
        {
            name: "zshrc",
            type: "Shell",
            owner: "mac-mini",
            status: "Changed",
            updated: "2026-05-07",
            size: "11 KB",
            refs: 9,
            location: "~/.zshrc",
        },
        {
            name: "gitconfig",
            type: "Git",
            owner: "all-hosts",
            status: "Synced",
            updated: "2026-05-05",
            size: "3 KB",
            refs: 4,
            location: "~/.gitconfig",
        },
        {
            name: "nvim",
            type: "Editor",
            owner: "macbook",
            status: "Review",
            updated: "2026-05-07",
            size: "68 KB",
            refs: 11,
            location: "~/.config/nvim",
        },
        {
            name: "codex-config.toml",
            type: "Agent",
            owner: "all-hosts",
            status: "Synced",
            updated: "2026-05-07",
            size: "7 KB",
            refs: 16,
            location: "~/.codex",
        },
    ],
};

const assetColumns: ColumnDef<AssetRecord>[] = [
    {
        accessorKey: "name",
        header: ({ column }) => (
            <SortableHeader
                label="资产"
                sorted={column.getIsSorted()}
                onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            />
        ),
        cell: ({ row }) => (
            <div className="min-w-0">
                <div className="truncate font-medium">{row.original.name}</div>
                <div className="mt-0.5 flex items-center gap-1 text-xs text-muted-foreground">
                    <FolderTree className="h-3 w-3" />
                    <span className="truncate">{row.original.location}</span>
                </div>
            </div>
        ),
    },
    {
        accessorKey: "type",
        header: "类型",
        cell: ({ row }) => (
            <Badge variant="outline" className="rounded-full font-medium">
                {row.original.type}
            </Badge>
        ),
    },
    {
        accessorKey: "owner",
        header: "归属",
    },
    {
        accessorKey: "refs",
        header: ({ column }) => (
            <SortableHeader
                label="引用"
                sorted={column.getIsSorted()}
                onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            />
        ),
        cell: ({ row }) => (
            <span className="font-mono text-sm">{row.original.refs}</span>
        ),
    },
    {
        accessorKey: "size",
        header: "体量",
    },
    {
        accessorKey: "status",
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
    {
        accessorKey: "updated",
        header: ({ column }) => (
            <SortableHeader
                label="更新时间"
                sorted={column.getIsSorted()}
                onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            />
        ),
    },
];

function moduleIdFromPath(pathname: string): AssetModuleId {
    return (
        PATH_TO_MODULE.find(([prefix]) => pathname.startsWith(prefix))?.[1] ??
        "notes"
    );
}

export default function AssetsPage() {
    const location = useLocation();
    const activeId = moduleIdFromPath(location.pathname);
    const activeModule = useMemo(
        () => MODULES.find((item) => item.id === activeId) ?? MODULES[0],
        [activeId],
    );

    if (activeId === "notes") {
        return <NotesPluginRedirectPage />;
    }

    return (
        <div className="min-h-full bg-[#f5f1e8] text-foreground">
            <section
                className={cn(
                    "border-b bg-gradient-to-br px-4 py-3 lg:px-5",
                    activeModule.accent,
                )}
            >
                <div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_auto] xl:items-end">
                    <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                            <Boxes className="h-3.5 w-3.5" />
                            Asset Workbench
                            <Badge variant="outline" className="rounded-full bg-background/70">
                                {activeModule.eyebrow}
                            </Badge>
                        </div>
                        <div className="mt-2 flex flex-wrap items-end gap-3">
                            <h1 className="text-2xl font-semibold tracking-tight lg:text-3xl">
                                {activeModule.title}
                            </h1>
                            <StatusBadge status={activeModule.status} />
                        </div>
                        <p className="mt-1 max-w-4xl text-sm leading-6 text-muted-foreground">
                            {activeModule.detail}
                        </p>
                    </div>
                    <div className="flex flex-wrap gap-2">
                        <Button type="button" variant="outline" size="sm" className="bg-background/70">
                            <UploadCloud className="h-4 w-4" />
                            导入
                        </Button>
                        <Button type="button" variant="outline" size="sm" className="bg-background/70">
                            <SlidersHorizontal className="h-4 w-4" />
                            规则
                        </Button>
                        <Button type="button" size="sm">
                            <Plus className="h-4 w-4" />
                            新建资产
                        </Button>
                    </div>
                </div>
                <div className="mt-3 grid grid-cols-2 gap-2 xl:grid-cols-4">
                    {activeModule.metrics.map((metric) => (
                        <MetricTile key={metric.label} metric={metric} />
                    ))}
                </div>
            </section>

            <section className="grid min-h-[calc(100vh-14rem)] xl:grid-cols-[minmax(0,1fr)_21rem]">
                <main className="min-w-0 border-r bg-background">
                    <div className="flex overflow-x-auto border-b bg-card/80 xl:grid xl:grid-cols-3 xl:overflow-visible">
                        {MODULES.map((item) => (
                            <ModuleCell
                                key={item.id}
                                item={item}
                                active={item.id === activeId}
                            />
                        ))}
                    </div>
                    <AssetDataTable
                        data={ASSET_RECORDS[activeId]}
                        module={activeModule}
                    />
                </main>
                <AssetContextPanel module={activeModule} activeId={activeId} />
            </section>
        </div>
    );
}

let notesPluginRegistrationPromise: ReturnType<
    typeof registerNotesFragmentsPlugin
> | null = null;

function NotesPluginRedirectPage() {
    const navigate = useNavigate();
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;

        async function run() {
            setError(null);
            try {
                notesPluginRegistrationPromise ??= registerNotesFragmentsPlugin();
                const result = await notesPluginRegistrationPromise;
                if (!cancelled) {
                    window.dispatchEvent(new Event("aio:plugin-runtime-updated"));
                    navigate(`/apps/${result.instance_slug}/fragments`, {
                        replace: true,
                    });
                }
            } catch (err) {
                notesPluginRegistrationPromise = null;
                if (!cancelled) {
                    setError(
                        err instanceof Error
                            ? notesPluginRegistrationErrorMessage(err.message)
                            : "注册碎片笔记插件失败",
                    );
                }
            }
        }

        void run();
        return () => {
            cancelled = true;
        };
    }, [navigate]);

    return (
        <div className="flex min-h-[60vh] items-center justify-center bg-[#f5f1e8] p-6">
            <Card className="w-full max-w-xl rounded-2xl border-stone-300 bg-white shadow-sm">
                <CardHeader className="space-y-2">
                    <CardTitle className="flex items-center gap-2 text-lg">
                        <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                        正在打开碎片笔记插件
                    </CardTitle>
                    <p className="text-sm leading-6 text-muted-foreground">
                        首次进入会注册 `notes-fragments.wasm`，插件元数据写入 PostgreSQL，二进制写入 MinIO，然后跳转到插件实例页。
                    </p>
                </CardHeader>
                <CardContent>
                    {error ? (
                        <div className="space-y-3 rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm leading-6 text-rose-700">
                            <div>注册失败：{error}</div>
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                className="border-rose-200 bg-white text-rose-700 hover:bg-rose-100"
                                onClick={() => navigate("/env")}
                            >
                                去环境配置
                            </Button>
                        </div>
                    ) : (
                        <div className="rounded-xl border bg-muted/30 px-4 py-3 text-sm text-muted-foreground">
                            正在准备插件实例...
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}

function notesPluginRegistrationErrorMessage(message: string): string {
    if (
        message.includes("PostgreSQL metadata storage") ||
        message.includes("MinIO binary storage") ||
        message.includes("bare wasm upload requires")
    ) {
        return "需要先配置 PostgreSQL 和 MinIO。插件元数据必须写入 PostgreSQL，`.wasm` 二进制必须写入 MinIO。";
    }
    return message;
}

function AssetDataTable({
    data,
    module,
}: {
    data: AssetRecord[];
    module: AssetModule;
}) {
    const [globalFilter, setGlobalFilter] = useState("");
    const [sorting, setSorting] = useState<SortingState>([]);
    const table = useReactTable({
        data,
        columns: assetColumns,
        state: {
            globalFilter,
            sorting,
        },
        onGlobalFilterChange: setGlobalFilter,
        onSortingChange: setSorting,
        getCoreRowModel: getCoreRowModel(),
        getFilteredRowModel: getFilteredRowModel(),
        getSortedRowModel: getSortedRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
        initialState: {
            pagination: {
                pageSize: 6,
            },
        },
    });

    return (
        <div className="min-w-0">
            <div className="flex flex-wrap items-center justify-between gap-3 border-b bg-card px-4 py-3 lg:px-5">
                <div className="min-w-0">
                    <div className="flex items-center gap-2 text-sm font-semibold">
                        <Database className="h-4 w-4 text-muted-foreground" />
                        {module.title}目录
                    </div>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                        TanStack Table：过滤、排序、分页、后续接列显隐和批量操作。
                    </p>
                </div>
                <div className="flex min-w-0 flex-1 flex-wrap justify-end gap-2">
                    <div className="relative min-w-[16rem] max-w-sm flex-1">
                        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            value={globalFilter}
                            onChange={(event) => setGlobalFilter(event.target.value)}
                            className="h-9 pl-9"
                            placeholder="筛选名称、类型、路径..."
                        />
                    </div>
                    <Button type="button" variant="outline" size="sm">
                        <Filter className="h-4 w-4" />
                        条件
                    </Button>
                    <Button type="button" variant="outline" size="sm">
                        <Archive className="h-4 w-4" />
                        批量
                    </Button>
                </div>
            </div>

            <div className="overflow-x-auto">
                <Table>
                    <TableHeader className="bg-muted/50">
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => (
                                    <TableHead key={header.id} className="h-10 px-4 text-xs">
                                        {header.isPlaceholder
                                            ? null
                                            : flexRender(
                                                  header.column.columnDef.header,
                                                  header.getContext(),
                                              )}
                                    </TableHead>
                                ))}
                                <TableHead className="h-10 w-12 px-4 text-right" />
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows.length ? (
                            table.getRowModel().rows.map((row) => (
                                <TableRow key={row.id} className="hover:bg-muted/35">
                                    {row.getVisibleCells().map((cell) => (
                                        <TableCell key={cell.id} className="px-4 py-3 align-middle">
                                            {flexRender(
                                                cell.column.columnDef.cell,
                                                cell.getContext(),
                                            )}
                                        </TableCell>
                                    ))}
                                    <TableCell className="px-4 py-3 text-right">
                                        <Button
                                            type="button"
                                            variant="ghost"
                                            size="icon"
                                            className="h-8 w-8"
                                        >
                                            <MoreHorizontal className="h-4 w-4" />
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell
                                    colSpan={assetColumns.length + 1}
                                    className="h-40 text-center text-sm text-muted-foreground"
                                >
                                    没有匹配的资产。调整筛选条件或从本地目录导入。
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-3 border-t bg-card px-4 py-3 text-sm text-muted-foreground lg:px-5">
                <span>
                    已显示 {table.getRowModel().rows.length} /{" "}
                    {table.getFilteredRowModel().rows.length} 条
                </span>
                <div className="flex items-center gap-2">
                    <span className="text-xs">
                        第 {table.getState().pagination.pageIndex + 1} /{" "}
                        {table.getPageCount() || 1} 页
                    </span>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={!table.getCanPreviousPage()}
                        onClick={() => table.previousPage()}
                    >
                        上一页
                    </Button>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={!table.getCanNextPage()}
                        onClick={() => table.nextPage()}
                    >
                        下一页
                    </Button>
                </div>
            </div>
        </div>
    );
}

function AssetContextPanel({
    module,
    activeId,
}: {
    module: AssetModule;
    activeId: AssetModuleId;
}) {
    const firstRecord = ASSET_RECORDS[activeId][0];

    return (
        <aside className="bg-[#fbfaf6]">
            <div className="border-b px-4 py-3">
                <div className="flex items-center justify-between gap-3">
                    <div>
                        <div className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                            Context Detail
                        </div>
                        <h2 className="mt-1 text-base font-semibold">{module.title}</h2>
                    </div>
                    <span className="rounded-2xl border bg-background p-2 text-muted-foreground">
                        {module.icon}
                    </span>
                </div>
            </div>

            <div className="space-y-4 p-4">
                <Card className="rounded-2xl shadow-sm">
                    <CardHeader className="p-4 pb-2">
                        <CardTitle className="flex items-center gap-2 text-sm">
                            <ShieldCheck className="h-4 w-4 text-emerald-600" />
                            模块边界
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3 p-4 pt-1">
                        <p className="text-sm leading-6 text-muted-foreground">
                            这里是资产主轴下的侧轴节点，不回退到“平台总览”。
                        </p>
                        <div className="grid grid-cols-2 gap-2">
                            {module.responsibilities.map((item) => (
                                <div
                                    key={item}
                                    className="rounded-xl border bg-muted/30 px-3 py-2 text-sm font-medium"
                                >
                                    {item}
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>

                <Card className="rounded-2xl shadow-sm">
                    <CardHeader className="p-4 pb-2">
                        <CardTitle className="flex items-center gap-2 text-sm">
                            <FileArchive className="h-4 w-4 text-blue-600" />
                            当前对象
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3 p-4 pt-1">
                        {firstRecord ? (
                            <>
                                <SpecLine label="名称" value={firstRecord.name} />
                                <SpecLine label="类型" value={firstRecord.type} />
                                <SpecLine label="路径" value={firstRecord.location} />
                                <SpecLine label="引用" value={`${firstRecord.refs} 个`} />
                                <SpecLine label="状态" value={firstRecord.status} />
                            </>
                        ) : (
                            <p className="text-sm text-muted-foreground">
                                当前模块没有表格对象，使用专属工作流展示。
                            </p>
                        )}
                    </CardContent>
                </Card>

                <Card className="rounded-2xl border-stone-800 bg-[#181915] text-stone-50 shadow-sm">
                    <CardHeader className="p-4 pb-2">
                        <CardTitle className="flex items-center gap-2 text-sm">
                            <Layers3 className="h-4 w-4 text-amber-300" />
                            当前收敛范围
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3 p-4 pt-1 text-sm text-stone-300">
                        <StepLine index="01" title="笔记" />
                        <StepLine index="02" title="安装包" />
                        <StepLine index="03" title="dotfiles" />
                    </CardContent>
                </Card>
            </div>
        </aside>
    );
}

function ModuleCell({ item, active }: { item: AssetModule; active: boolean }) {
    return (
        <div
            className={cn(
                "min-w-40 shrink-0 border-r px-3 py-3 xl:min-w-0 xl:shrink",
                active ? "bg-[#181915] text-stone-50" : "bg-card hover:bg-muted/35",
            )}
        >
            <div className="flex items-center justify-between gap-2">
                <div className="flex min-w-0 items-center gap-2">
                    <span className={active ? "text-amber-300" : "text-muted-foreground"}>
                        {item.icon}
                    </span>
                    <span className="truncate text-sm font-semibold">{item.title}</span>
                </div>
                <span
                    className={cn(
                        "h-2 w-2 shrink-0 rounded-full",
                        item.status === "优先" ? "bg-emerald-400" : "bg-amber-400",
                    )}
                />
            </div>
            <p
                className={cn(
                    "mt-1 line-clamp-2 text-xs leading-5",
                    active ? "text-stone-300" : "text-muted-foreground",
                )}
            >
                {item.responsibilities.join(" / ")}
            </p>
        </div>
    );
}

function MetricTile({ metric, flush = false }: { metric: AssetMetric; flush?: boolean }) {
    return (
        <div
            className={cn(
                "border bg-background/70 px-4 py-3",
                flush ? "border-b-0 border-l-0 border-t-0" : "rounded-2xl shadow-sm",
            )}
        >
            <div className="flex items-start justify-between gap-3">
                <div>
                    <div className="text-xs font-medium text-muted-foreground">
                        {metric.label}
                    </div>
                    <div className="mt-1 text-2xl font-semibold tracking-tight">
                        {metric.value}
                    </div>
                </div>
                <CheckCircle2 className="h-4 w-4 text-emerald-600" />
            </div>
            <div className="mt-1 text-xs text-muted-foreground">{metric.detail}</div>
        </div>
    );
}

function StatusBadge({ status }: { status: string }) {
    return (
        <Badge
            variant="outline"
            className={cn("rounded-full font-semibold", statusBadgeClass(status))}
        >
            {status}
        </Badge>
    );
}

function statusBadgeClass(status: string) {
    const normalized = status.toLowerCase();
    if (
        normalized.includes("ready") ||
        normalized.includes("indexed") ||
        normalized.includes("active") ||
        normalized.includes("connected") ||
        normalized.includes("synced") ||
        status === "优先"
    ) {
        return "border-emerald-200 bg-emerald-50 text-emerald-700";
    }
    if (
        normalized.includes("draft") ||
        normalized.includes("review") ||
        normalized.includes("planned") ||
        normalized.includes("changed") ||
        status === "规划"
    ) {
        return "border-amber-200 bg-amber-50 text-amber-700";
    }
    if (normalized.includes("missing") || normalized.includes("disabled")) {
        return "border-rose-200 bg-rose-50 text-rose-700";
    }
    return "border-blue-200 bg-blue-50 text-blue-700";
}

function SortableHeader({
    label,
    sorted,
    onClick,
}: {
    label: string;
    sorted: false | "asc" | "desc";
    onClick: () => void;
}) {
    return (
        <Button
            type="button"
            variant="ghost"
            size="sm"
            className="-ml-3 h-8 px-2 text-xs font-semibold"
            onClick={onClick}
        >
            {label}
            <ArrowUpDown
                className={cn(
                    "h-3.5 w-3.5",
                    sorted ? "text-foreground" : "text-muted-foreground",
                )}
            />
        </Button>
    );
}

function SpecLine({ label, value }: { label: string; value: string }) {
    return (
        <div className="grid grid-cols-[4.5rem_minmax(0,1fr)] gap-3 text-sm">
            <span className="text-muted-foreground">{label}</span>
            <span className="truncate font-medium">{value}</span>
        </div>
    );
}

function StepLine({ index, title }: { index: string; title: string }) {
    return (
        <div className="flex items-center gap-3">
            <span className="flex h-7 w-7 items-center justify-center rounded-full border border-stone-600 text-xs text-amber-300">
                {index}
            </span>
            <span>{title}</span>
        </div>
    );
}
