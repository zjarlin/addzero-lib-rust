import { useMemo, useState, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
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
    AtSign,
    Boxes,
    Brain,
    CheckCircle2,
    CheckSquare,
    Clock3,
    Code2,
    Database,
    FileArchive,
    FileText,
    Filter,
    FolderTree,
    Hash,
    Inbox,
    Layers3,
    Link2,
    List,
    MoreHorizontal,
    Network,
    PackageOpen,
    Paperclip,
    Plus,
    Search,
    Send,
    Settings2,
    ShieldCheck,
    SlidersHorizontal,
    Smile,
    Sparkles,
    TableProperties,
    Tags,
    UploadCloud,
    Wrench,
    Zap,
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
    Textarea,
    cn,
} from "@addzero/ui";

type AssetModuleId =
    | "files"
    | "notes"
    | "packages"
    | "dotfiles"
    | "agents"
    | "agent-skills"
    | "agent-cli"
    | "agent-mcp";

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

interface NoteCardData {
    time: string;
    body: string;
    tags: string[];
    kind: string;
    source: string;
    accent: string;
}

const MODULES: AssetModule[] = [
    {
        id: "files",
        title: "资产文件",
        eyebrow: "Files",
        detail: "统一管理脚本输入输出、模板附件、导出物、插件包和可复用文件素材。",
        status: "优先",
        icon: <FolderTree className="h-4 w-4" />,
        accent: "from-emerald-500/15 via-stone-50 to-stone-50",
        responsibilities: ["目录树", "文件元数据", "引用关系", "导入导出"],
        metrics: [
            { label: "已索引", value: "128", detail: "workspace files" },
            { label: "引用", value: "37", detail: "linked objects" },
            { label: "待校验", value: "6", detail: "checksum queue" },
            { label: "导入源", value: "4", detail: "local roots" },
        ],
    },
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
            { label: "待整理", value: "11", detail: "triage inbox" },
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
    {
        id: "agents",
        title: "Agent 资产",
        eyebrow: "Agent Assets",
        detail: "统一放置能被 Agent 装载、调用或发布的能力资产。",
        status: "优先",
        icon: <Brain className="h-4 w-4" />,
        accent: "from-cyan-500/15 via-stone-50 to-stone-50",
        responsibilities: ["Skill", "CLI", "MCP", "插件元数据"],
        metrics: [
            { label: "Skill", value: "31", detail: "loaded skills" },
            { label: "CLI", value: "8", detail: "commands" },
            { label: "MCP", value: "2", detail: "connected" },
            { label: "缺口", value: "1", detail: "figma missing" },
        ],
    },
    {
        id: "agent-skills",
        title: "Agent Skills",
        eyebrow: "Skills",
        detail: "管理 Codex / Agent 可读取的技能包、触发条件、说明和版本。",
        status: "优先",
        icon: <Wrench className="h-4 w-4" />,
        accent: "from-lime-500/15 via-stone-50 to-stone-50",
        responsibilities: ["SKILL.md", "引用资料", "脚本", "资产"],
        metrics: [
            { label: "技能包", value: "31", detail: "available" },
            { label: "项目级", value: "9", detail: "workspace skills" },
            { label: "引用", value: "76", detail: "reference docs" },
            { label: "脚本", value: "14", detail: "helpers" },
        ],
    },
    {
        id: "agent-cli",
        title: "Agent CLI",
        eyebrow: "CLI",
        detail: "沉淀 Agent 可调用或可生成的命令行能力、参数契约和安装方式。",
        status: "规划",
        icon: <Code2 className="h-4 w-4" />,
        accent: "from-slate-500/15 via-stone-50 to-stone-50",
        responsibilities: ["命令契约", "参数", "安装方法", "运行记录"],
        metrics: [
            { label: "命令", value: "8", detail: "catalog" },
            { label: "契约", value: "3", detail: "typed schemas" },
            { label: "草稿", value: "4", detail: "planned" },
            { label: "运行", value: "22", detail: "history" },
        ],
    },
    {
        id: "agent-mcp",
        title: "Agent MCP",
        eyebrow: "MCP",
        detail: "管理 MCP server、工具暴露、权限边界和连接配置。",
        status: "规划",
        icon: <Network className="h-4 w-4" />,
        accent: "from-rose-500/15 via-stone-50 to-stone-50",
        responsibilities: ["Server", "Tools", "权限", "连接健康"],
        metrics: [
            { label: "服务", value: "3", detail: "declared" },
            { label: "在线", value: "2", detail: "connected" },
            { label: "工具", value: "29", detail: "exposed tools" },
            { label: "缺失", value: "1", detail: "figma" },
        ],
    },
];

const PATH_TO_MODULE: Array<[string, AssetModuleId]> = [
    ["/assets/agents/skills", "agent-skills"],
    ["/assets/agents/cli", "agent-cli"],
    ["/assets/agents/mcp", "agent-mcp"],
    ["/assets/agents", "agents"],
    ["/assets/notes", "notes"],
    ["/assets/packages", "packages"],
    ["/assets/dotfiles", "dotfiles"],
    ["/assets/files", "files"],
    ["/storage", "files"],
    ["/knowledge", "notes"],
    ["/skills", "agent-skills"],
];

const ASSET_RECORDS: Record<AssetModuleId, AssetRecord[]> = {
    files: [
        {
            name: "aio-plugin-runtime-notes.md",
            type: "Markdown",
            owner: "workspace",
            status: "Indexed",
            updated: "2026-05-07",
            size: "18 KB",
            refs: 5,
            location: "docs/runtime",
        },
        {
            name: "dotfiles-snapshot.zip",
            type: "Archive",
            owner: "local-machine",
            status: "Ready",
            updated: "2026-05-06",
            size: "2.4 MB",
            refs: 3,
            location: "assets/backups",
        },
        {
            name: "demo-plugin.aio-plugin",
            type: "Plugin Package",
            owner: "plugins",
            status: "Draft",
            updated: "2026-05-05",
            size: "812 KB",
            refs: 2,
            location: "plugins/demo",
        },
        {
            name: "aio-admin-prototype-board.html",
            type: "Prototype",
            owner: "design",
            status: "Indexed",
            updated: "2026-05-07",
            size: "46 KB",
            refs: 8,
            location: "docs/prototypes",
        },
        {
            name: "skill-handoff.json",
            type: "JSON",
            owner: "agent-assets",
            status: "Review",
            updated: "2026-05-07",
            size: "9 KB",
            refs: 4,
            location: "docs/prototypes",
        },
        {
            name: "release-manifest.toml",
            type: "Manifest",
            owner: "release",
            status: "Ready",
            updated: "2026-05-04",
            size: "6 KB",
            refs: 6,
            location: "dist",
        },
    ],
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
    agents: [
        {
            name: "frontend-design",
            type: "Skill",
            owner: "codex",
            status: "Active",
            updated: "2026-05-07",
            size: "12 KB",
            refs: 10,
            location: ".agents/skills",
        },
        {
            name: "aio assets scan",
            type: "CLI",
            owner: "aio",
            status: "Planned",
            updated: "2026-05-07",
            size: "contract",
            refs: 2,
            location: "crates/aio",
        },
        {
            name: "chrome-devtools",
            type: "MCP Server",
            owner: "codex",
            status: "Connected",
            updated: "2026-05-07",
            size: "tools",
            refs: 18,
            location: "~/.codex/config.toml",
        },
        {
            name: "figma",
            type: "MCP Server",
            owner: "design",
            status: "Missing",
            updated: "2026-05-07",
            size: "setup",
            refs: 1,
            location: "~/.codex/config.toml",
        },
    ],
    "agent-skills": [
        {
            name: "zjarlin-engineering",
            type: "Skill",
            owner: "codex",
            status: "Active",
            updated: "2026-05-07",
            size: "21 KB",
            refs: 14,
            location: "~/.codex/skills",
        },
        {
            name: "frontend-design",
            type: "Skill",
            owner: "workspace",
            status: "Active",
            updated: "2026-05-07",
            size: "10 KB",
            refs: 7,
            location: ".agents/skills",
        },
        {
            name: "compact-workbench-page-rules",
            type: "Skill",
            owner: "codex",
            status: "Active",
            updated: "2026-05-07",
            size: "8 KB",
            refs: 5,
            location: "~/.codex/skills",
        },
        {
            name: "rust-best-practices",
            type: "Skill",
            owner: "workspace",
            status: "Active",
            updated: "2026-05-01",
            size: "33 KB",
            refs: 9,
            location: ".agents/skills",
        },
    ],
    "agent-cli": [
        {
            name: "aio dotfiles sync",
            type: "CLI",
            owner: "dotfiles",
            status: "Planned",
            updated: "2026-05-07",
            size: "contract",
            refs: 3,
            location: "crates/aio-cli",
        },
        {
            name: "aio skill pack",
            type: "CLI",
            owner: "agent-assets",
            status: "Draft",
            updated: "2026-05-07",
            size: "contract",
            refs: 8,
            location: "crates/aio-cli",
        },
        {
            name: "aio assets import",
            type: "CLI",
            owner: "assets",
            status: "Review",
            updated: "2026-05-07",
            size: "contract",
            refs: 4,
            location: "crates/aio-cli",
        },
    ],
    "agent-mcp": [
        {
            name: "chrome-devtools",
            type: "MCP Server",
            owner: "codex",
            status: "Connected",
            updated: "2026-05-07",
            size: "23 tools",
            refs: 11,
            location: "~/.codex/config.toml",
        },
        {
            name: "playwright",
            type: "MCP Server",
            owner: "codex",
            status: "Connected",
            updated: "2026-05-07",
            size: "18 tools",
            refs: 9,
            location: "~/.codex/config.toml",
        },
        {
            name: "figma",
            type: "MCP Server",
            owner: "design",
            status: "Missing",
            updated: "2026-05-07",
            size: "setup",
            refs: 2,
            location: "~/.codex/config.toml",
        },
    ],
};

const NOTE_CARDS: NoteCardData[] = [
    {
        time: "2026-05-07 20:14:00",
        body: 'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock',
        tags: ["闪念", "macOS", "dock"],
        kind: "Command",
        source: "剪贴板",
        accent: "bg-amber-400",
    },
    {
        time: "2026-05-07 19:45:54",
        body: '{"error":{"code":"invalid_request","message":"unsupported responses tool type custom","type":"invalid_request_error"}}',
        tags: ["debug", "api", "tool"],
        kind: "Error",
        source: "运行日志",
        accent: "bg-rose-400",
    },
    {
        time: "2026-05-07 12:59:28",
        body: "如果远端是 Mac mini，macOS 自带的屏幕共享可以直接连接。先在远端开启服务，本地 Finder 里按 Command + K，输入 vnc://远端IP。",
        tags: ["remote", "macOS", "操作"],
        kind: "Howto",
        source: "人工输入",
        accent: "bg-emerald-400",
    },
    {
        time: "2026-05-04 23:28:38",
        body: "头脑风暴：我希望有个 all in one 的东西，能把笔记、安装包、密码本、skill、配置文件之类的管理起来。",
        tags: ["资产", "idea", "aio"],
        kind: "Idea",
        source: "闪念",
        accent: "bg-blue-400",
    },
    {
        time: "2026-05-02 23:36:18",
        body: 'strum = { version = "0.26", features = ["derive"] } 以后写 rust 记得这个库可以零成本抽象 rust 简化。',
        tags: ["rust", "crate", "闪念"],
        kind: "Snippet",
        source: "手记",
        accent: "bg-lime-400",
    },
    {
        time: "2026-05-01 10:08:11",
        body: "UI 规则：顶部主轴和左侧路由树是二维上下文，不要把工作台、资产、运行混成一个平铺菜单。",
        tags: ["admin", "导航", "规则"],
        kind: "Decision",
        source: "AGENTS.md",
        accent: "bg-stone-900",
    },
];

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
        "files"
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
        return <NotesWorkbench activeModule={activeModule} />;
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
                    <div className="flex overflow-x-auto border-b bg-card/80 xl:grid xl:grid-cols-8 xl:overflow-visible">
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

function NotesWorkbench({ activeModule }: { activeModule: AssetModule }) {
    return (
        <div className="min-h-full bg-[#f6f5ef] text-[#181915]">
            <section className="sticky top-0 z-10 border-b bg-[#fbfaf4]/95 px-3 py-3 backdrop-blur lg:px-5">
                <div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_28rem_auto] xl:items-center">
                    <div className="flex min-w-0 items-center gap-3">
                        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border bg-[#171a17] text-amber-300 shadow-sm">
                            <Zap className="h-5 w-5 fill-current" />
                        </div>
                        <div className="min-w-0">
                            <div className="flex flex-wrap items-center gap-2 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                                Blinko Reference
                                <Badge variant="outline" className="rounded-full bg-white/70">
                                    {activeModule.eyebrow}
                                </Badge>
                            </div>
                            <h1 className="truncate text-2xl font-semibold tracking-tight">
                                闪念笔记资产流
                            </h1>
                        </div>
                    </div>
                    <div className="relative min-w-0">
                        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            className="h-10 rounded-2xl border-stone-300 bg-white/80 pl-9 shadow-sm"
                            placeholder="搜索笔记、标签、来源..."
                        />
                    </div>
                    <div className="flex flex-wrap justify-start gap-2 xl:justify-end">
                        <Button type="button" variant="outline" size="sm" className="rounded-full bg-white/80">
                            <Filter className="h-4 w-4" />
                            筛选
                        </Button>
                        <Button type="button" size="sm" className="rounded-full">
                            <Plus className="h-4 w-4" />
                            新笔记
                        </Button>
                    </div>
                </div>
            </section>

            <section className="grid min-h-[calc(100vh-4.25rem)] xl:grid-cols-[minmax(0,1fr)_20rem]">
                <main className="min-w-0 border-r">
                    <div className="grid grid-cols-2 border-b bg-[#efeadf] lg:grid-cols-4">
                        {activeModule.metrics.map((metric) => (
                            <MetricTile key={metric.label} metric={metric} flush />
                        ))}
                    </div>
                    <QuickCapture />
                    <div className="border-b bg-[#fbfaf4] px-4 py-3 lg:px-5">
                        <div className="flex flex-wrap items-center justify-between gap-3">
                            <div className="flex flex-wrap gap-2">
                                {["全部", "闪念", "代码", "远程", "决策", "未整理"].map(
                                    (tag, index) => (
                                        <Badge
                                            key={tag}
                                            variant={index === 0 ? "default" : "outline"}
                                            className="rounded-full"
                                        >
                                            {tag}
                                        </Badge>
                                    ),
                                )}
                            </div>
                            <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                <Clock3 className="h-3.5 w-3.5" />
                                最近 7 天 42 条，11 条待整理
                            </div>
                        </div>
                    </div>
                    <div className="p-4 lg:p-5">
                        <section className="columns-1 gap-4 lg:columns-2 2xl:columns-3">
                            {NOTE_CARDS.map((note) => (
                                <NoteCard key={`${note.time}-${note.body}`} note={note} />
                            ))}
                        </section>
                    </div>
                </main>
                <aside className="bg-[#fbfaf4]">
                    <Card className="m-4 rounded-2xl border-stone-300 bg-white/80 shadow-sm">
                        <CardHeader className="p-4 pb-2">
                            <CardTitle className="flex items-center gap-2 text-sm">
                                <Inbox className="h-4 w-4 text-amber-600" />
                                捕获队列
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-3 p-4 pt-1">
                            <SpecLine label="入口" value="剪贴板 / 手输 / 日志" />
                            <SpecLine label="默认状态" value="待整理" />
                            <SpecLine label="索引" value="Postgres + FTS" />
                            <SpecLine label="下一步" value="标签归并与引用资产" />
                        </CardContent>
                    </Card>
                    <Card className="m-4 rounded-2xl border-stone-300 bg-white/80 shadow-sm">
                        <CardHeader className="p-4 pb-2">
                            <CardTitle className="flex items-center gap-2 text-sm">
                                <Tags className="h-4 w-4 text-stone-700" />
                                高频标签
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="flex flex-wrap gap-2 p-4 pt-1">
                            {["闪念", "debug", "macOS", "rust", "aio", "admin", "skill", "remote"].map(
                                (tag) => (
                                    <Badge key={tag} variant="secondary" className="rounded-full">
                                        #{tag}
                                    </Badge>
                                ),
                            )}
                        </CardContent>
                    </Card>
                    <Card className="m-4 rounded-2xl border-stone-300 bg-[#171a17] text-stone-50 shadow-sm">
                        <CardHeader className="p-4 pb-2">
                            <CardTitle className="flex items-center gap-2 text-sm">
                                <Sparkles className="h-4 w-4 text-amber-300" />
                                设计边界
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-3 p-4 pt-1 text-sm text-stone-300">
                            <p>笔记不是“知识库子页”，而是资产域下的快速捕获和整理入口。</p>
                            <p>顶部主轴保持资产域；左侧路由树负责切换文件、笔记、安装包和 Agent 资产。</p>
                        </CardContent>
                    </Card>
                </aside>
            </section>
        </div>
    );
}

function QuickCapture() {
    return (
        <section className="border-b bg-[#f6f5ef] p-3 lg:p-5">
            <Card className="overflow-hidden rounded-3xl border-stone-300 bg-white shadow-[0_16px_44px_rgba(42,37,29,0.10)]">
                <div className="flex flex-wrap items-center gap-2 border-b bg-[#fffdf7] px-3 py-2 text-sm text-muted-foreground lg:gap-3 lg:px-4 lg:py-3">
                    <Smile className="h-4 w-4" />
                    <span className="font-semibold text-foreground">H</span>
                    <span className="font-black text-foreground">B</span>
                    <span className="italic text-foreground">I</span>
                    <Link2 className="h-4 w-4" />
                    <List className="h-4 w-4" />
                    <CheckSquare className="h-4 w-4" />
                    <TableProperties className="h-4 w-4" />
                    <Code2 className="h-4 w-4" />
                    <Hash className="h-4 w-4" />
                    <AtSign className="h-4 w-4" />
                    <Paperclip className="h-4 w-4" />
                    <div className="ml-auto flex items-center gap-2 text-xs">
                        <Badge variant="outline" className="rounded-full">
                            Markdown
                        </Badge>
                        <Badge variant="outline" className="rounded-full">
                            待整理
                        </Badge>
                    </div>
                </div>
                <Textarea
                    className="min-h-28 resize-none border-0 bg-white px-4 py-3 text-base leading-7 shadow-none focus-visible:ring-0 lg:min-h-32 lg:py-4"
                    defaultValue={'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock'}
                />
                <div className="flex flex-wrap items-center justify-between gap-2 border-t bg-[#fffdf7] px-3 py-2 lg:gap-3 lg:px-4 lg:py-3">
                    <div className="flex flex-wrap items-center gap-2 text-muted-foreground">
                        <Button type="button" variant="ghost" size="sm" className="rounded-full">
                            <Zap className="h-4 w-4 fill-amber-400 text-amber-400" />
                            闪念
                        </Button>
                        <Button type="button" variant="ghost" size="sm" className="rounded-full">
                            <Hash className="h-4 w-4" />
                            标签
                        </Button>
                        <Button type="button" variant="ghost" size="sm" className="rounded-full">
                            <Link2 className="h-4 w-4" />
                            引用
                        </Button>
                        <Button type="button" variant="ghost" size="sm" className="rounded-full">
                            <Paperclip className="h-4 w-4" />
                            附件
                        </Button>
                    </div>
                    <Button type="button" className="rounded-2xl px-5">
                        <Send className="h-4 w-4" />
                        记录
                    </Button>
                </div>
            </Card>
        </section>
    );
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
                            第一阶段落地顺序
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3 p-4 pt-1 text-sm text-stone-300">
                        <StepLine index="01" title="资产文件 + 笔记" />
                        <StepLine index="02" title="安装包 + dotfiles" />
                        <StepLine index="03" title="Agent Skill / CLI / MCP" />
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

function NoteCard({ note }: { note: NoteCardData }) {
    return (
        <article className="mb-4 break-inside-avoid overflow-hidden rounded-3xl border border-stone-200 bg-white shadow-[0_12px_28px_rgba(42,37,29,0.07)]">
            <div className={cn("h-1.5", note.accent)} />
            <div className="p-4">
                <div className="flex items-start justify-between gap-3">
                    <div>
                        <time className="text-xs font-medium text-muted-foreground">
                            {note.time}
                        </time>
                        <div className="mt-1 flex flex-wrap items-center gap-2">
                            <Badge variant="outline" className="rounded-full">
                                {note.kind}
                            </Badge>
                            <span className="text-xs text-muted-foreground">
                                {note.source}
                            </span>
                        </div>
                    </div>
                    <Button type="button" variant="ghost" size="icon" className="h-8 w-8 rounded-full">
                        <MoreHorizontal className="h-4 w-4" />
                    </Button>
                </div>
                <p className="mt-4 text-[15px] leading-7 text-stone-900">{note.body}</p>
                <div className="mt-4 flex flex-wrap gap-2">
                    {note.tags.map((tag) => (
                        <Badge key={tag} variant="secondary" className="rounded-full">
                            #{tag}
                        </Badge>
                    ))}
                </div>
            </div>
        </article>
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
