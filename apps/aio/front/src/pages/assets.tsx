import { useMemo } from "react";
import { useLocation } from "react-router-dom";
import {
    type ColumnDef,
    flexRender,
    getCoreRowModel,
    getPaginationRowModel,
    useReactTable,
} from "@tanstack/react-table";
import {
    AtSign,
    Boxes,
    Brain,
    CheckSquare,
    Code2,
    Filter,
    FileArchive,
    FileText,
    FolderTree,
    Hash,
    Link2,
    List,
    Network,
    PackageOpen,
    Paperclip,
    Search,
    Send,
    Settings2,
    Smile,
    Sparkles,
    TableProperties,
    Wrench,
    Zap,
} from "lucide-react";
import {
    Badge,
    Button,
    Input,
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
    Textarea,
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

interface AssetModule {
    id: AssetModuleId;
    title: string;
    eyebrow: string;
    detail: string;
    status: string;
    icon: React.ReactNode;
    responsibilities: string[];
}

interface AssetRecord {
    name: string;
    type: string;
    owner: string;
    status: string;
    updated: string;
}

const assetColumns: ColumnDef<AssetRecord>[] = [
    {
        accessorKey: "name",
        header: "名称",
    },
    {
        accessorKey: "type",
        header: "类型",
    },
    {
        accessorKey: "owner",
        header: "归属",
    },
    {
        accessorKey: "status",
        header: "状态",
        cell: ({ row }) => <Badge variant="secondary">{row.original.status}</Badge>,
    },
    {
        accessorKey: "updated",
        header: "更新时间",
    },
];

const MODULES: AssetModule[] = [
    {
        id: "files",
        title: "资产文件",
        eyebrow: "Files",
        detail: "统一管理脚本输入输出、模板附件、导出物、插件包和可复用文件素材。",
        status: "优先",
        icon: <FolderTree className="h-4 w-4" />,
        responsibilities: ["目录树", "文件元数据", "引用关系", "导入导出"],
    },
    {
        id: "notes",
        title: "笔记",
        eyebrow: "Notes",
        detail: "沉淀人工笔记、任务记录、知识来源和后续可检索上下文。",
        status: "优先",
        icon: <FileText className="h-4 w-4" />,
        responsibilities: ["Markdown", "来源追踪", "标签", "检索索引"],
    },
    {
        id: "packages",
        title: "安装包",
        eyebrow: "Packages",
        detail: "维护本地安装包、CLI 包、桌面包、插件包和版本发布记录。",
        status: "优先",
        icon: <PackageOpen className="h-4 w-4" />,
        responsibilities: ["版本", "校验和", "发布目标", "安装记录"],
    },
    {
        id: "dotfiles",
        title: "dotfiles",
        eyebrow: "Dotfiles",
        detail: "把 shell、编辑器、工具链和机器初始化配置纳入资产治理。",
        status: "优先",
        icon: <Settings2 className="h-4 w-4" />,
        responsibilities: ["配置同步", "机器画像", "差异检查", "恢复动作"],
    },
    {
        id: "agents",
        title: "Agent 资产",
        eyebrow: "Agent Assets",
        detail: "统一放置能被 Agent 装载、调用或发布的能力资产。",
        status: "优先",
        icon: <Brain className="h-4 w-4" />,
        responsibilities: ["Skill", "CLI", "MCP", "插件元数据"],
    },
    {
        id: "agent-skills",
        title: "Agent Skills",
        eyebrow: "Skills",
        detail: "管理 Codex / Agent 可读取的技能包、触发条件、说明和版本。",
        status: "优先",
        icon: <Wrench className="h-4 w-4" />,
        responsibilities: ["SKILL.md", "引用资料", "脚本", "资产"],
    },
    {
        id: "agent-cli",
        title: "Agent CLI",
        eyebrow: "CLI",
        detail: "沉淀 Agent 可调用或可生成的命令行能力、参数契约和安装方式。",
        status: "规划",
        icon: <Code2 className="h-4 w-4" />,
        responsibilities: ["命令契约", "参数", "安装方法", "运行记录"],
    },
    {
        id: "agent-mcp",
        title: "Agent MCP",
        eyebrow: "MCP",
        detail: "管理 MCP server、工具暴露、权限边界和连接配置。",
        status: "规划",
        icon: <Network className="h-4 w-4" />,
        responsibilities: ["Server", "Tools", "权限", "连接健康"],
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
        },
        {
            name: "dotfiles-snapshot.zip",
            type: "Archive",
            owner: "local-machine",
            status: "Ready",
            updated: "2026-05-06",
        },
        {
            name: "demo-plugin.aio-plugin",
            type: "Plugin Package",
            owner: "plugins",
            status: "Draft",
            updated: "2026-05-05",
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
        },
        {
            name: "codex-skill-pack.tgz",
            type: "Skill Package",
            owner: "agent-assets",
            status: "Ready",
            updated: "2026-05-06",
        },
    ],
    dotfiles: [
        {
            name: "zshrc",
            type: "Shell",
            owner: "mac-mini",
            status: "Changed",
            updated: "2026-05-07",
        },
        {
            name: "gitconfig",
            type: "Git",
            owner: "all-hosts",
            status: "Synced",
            updated: "2026-05-05",
        },
    ],
    agents: [
        {
            name: "frontend-design",
            type: "Skill",
            owner: "codex",
            status: "Active",
            updated: "2026-05-07",
        },
        {
            name: "aio assets scan",
            type: "CLI",
            owner: "aio",
            status: "Planned",
            updated: "2026-05-07",
        },
    ],
    "agent-skills": [
        {
            name: "zjarlin-engineering",
            type: "Skill",
            owner: "codex",
            status: "Active",
            updated: "2026-05-07",
        },
        {
            name: "rust-best-practices",
            type: "Skill",
            owner: "workspace",
            status: "Active",
            updated: "2026-05-01",
        },
    ],
    "agent-cli": [
        {
            name: "aio dotfiles sync",
            type: "CLI",
            owner: "dotfiles",
            status: "Planned",
            updated: "2026-05-07",
        },
        {
            name: "aio skill pack",
            type: "CLI",
            owner: "agent-assets",
            status: "Draft",
            updated: "2026-05-07",
        },
    ],
    "agent-mcp": [
        {
            name: "chrome-devtools",
            type: "MCP Server",
            owner: "codex",
            status: "Connected",
            updated: "2026-05-07",
        },
        {
            name: "figma",
            type: "MCP Server",
            owner: "design",
            status: "Missing",
            updated: "2026-05-07",
        },
    ],
};

const NOTE_CARDS = [
    {
        time: "2026-05-07 20:14:00",
        body: 'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock',
        tags: ["闪念", "macOS"],
    },
    {
        time: "2026-05-07 19:45:54",
        body: '{"error":{"code":"invalid_request","message":"unsupported responses tool type custom","type":"invalid_request_error"}}',
        tags: ["闪念", "debug"],
    },
    {
        time: "2026-05-07 12:59:28",
        body: "如果远端是 Mac mini，macOS 自带的屏幕共享可以直接连接。先在远端开启服务，本地 Finder 里按 Command + K，输入 vnc://远端IP。",
        tags: ["笔记", "remote"],
    },
    {
        time: "2026-05-04 23:28:38",
        body: "头脑风暴：我希望有个 all in one 的东西，能把笔记、安装包、密码本、skill、配置文件之类的管理起来。",
        tags: ["资产", "idea"],
    },
    {
        time: "2026-05-02 23:36:18",
        body: 'strum = { version = "0.26", features = ["derive"] } 以后写 rust 记得这个库可以零成本抽象 rust 简化',
        tags: ["rust", "闪念"],
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
        return <NotesPrototype />;
    }

    return (
        <div className="space-y-0">
            <section className="border-b bg-card">
                <div className="border-b px-5 py-3">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <Boxes className="h-3.5 w-3.5" />
                        Asset Context
                    </div>
                    <div className="mt-2 flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                        <div>
                            <h1 className="text-3xl font-semibold tracking-tight">
                                资产工作台
                            </h1>
                            <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                                资产场景不再挂在“资源”杂项下面。这里先按个人资产和 Agent
                                资产拆路由树，优先承接文件、笔记、安装包、dotfiles、skill、CLI 和 MCP。
                            </p>
                        </div>
                        <Badge variant="secondary" className="w-fit">
                            {activeModule.title}
                        </Badge>
                    </div>
                </div>

                <div className="grid gap-0 md:grid-cols-4">
                    {MODULES.slice(0, 4).map((item, index) => (
                        <AssetSignal key={item.id} item={item} index={index} />
                    ))}
                </div>
            </section>

            <section className="grid items-start xl:grid-cols-[1.05fr_0.95fr]">
                <div className="border-b bg-card xl:border-r">
                    <div className="border-b px-5 py-3">
                        <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                            {activeModule.icon}
                            {activeModule.eyebrow}
                        </div>
                        <h2 className="mt-2 text-xl font-semibold tracking-tight">
                            {activeModule.title}
                        </h2>
                        <p className="mt-2 text-sm text-muted-foreground">
                            {activeModule.detail}
                        </p>
                    </div>
                    <AssetDataTable data={ASSET_RECORDS[activeId]} />
                </div>

                <div className="border-b bg-card">
                    <div className="border-b px-5 py-3">
                        <h2 className="text-base font-semibold">功能划分</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            左侧树按资产场景展开，不和顶部主轴混在一起。
                        </p>
                    </div>
                    <div className="space-y-0">
                        {MODULES.map((item, index) => (
                            <div
                                key={item.id}
                                className={`px-5 py-4 ${index > 0 ? "border-t" : ""}`}
                            >
                                <div className="flex items-start justify-between gap-4">
                                    <div>
                                        <div className="flex items-center gap-2 text-sm font-medium">
                                            <span className="text-muted-foreground">
                                                {item.icon}
                                            </span>
                                            {item.title}
                                        </div>
                                        <p className="mt-1 text-sm text-muted-foreground">
                                            {item.detail}
                                        </p>
                                    </div>
                                    <Badge
                                        variant={
                                            item.status === "优先" ? "default" : "secondary"
                                        }
                                        className="shrink-0"
                                    >
                                        {item.status}
                                    </Badge>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </section>

            <section className="border-b bg-card p-4">
                <div className="flex items-center gap-2 text-sm font-medium">
                    <FileArchive className="h-4 w-4 text-muted-foreground" />
                    第一阶段落地顺序
                </div>
                <div className="mt-3 grid gap-3 md:grid-cols-3">
                    <PriorityCard title="1. 资产文件 + 笔记" detail="先有可浏览、可索引、可引用的本地资产基座。" />
                    <PriorityCard title="2. 安装包 + dotfiles" detail="再把机器环境和可安装产物纳入统一目录。" />
                    <PriorityCard title="3. Agent 资产" detail="最后把 skill、CLI、MCP 作为可发布能力资产管理。" />
                </div>
            </section>
        </div>
    );
}

function NotesPrototype() {
    return (
        <div className="min-h-full bg-muted/30">
            <section className="border-b bg-background px-5 py-3">
                <div className="flex items-center justify-between gap-4">
                    <div className="flex items-center gap-2">
                        <Zap className="h-5 w-5 fill-amber-400 text-amber-400" />
                        <h1 className="text-xl font-semibold tracking-tight">闪念</h1>
                    </div>
                    <div className="flex items-center gap-2">
                        <div className="relative hidden w-72 md:block">
                            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                            <Input className="h-9 pl-9" placeholder="搜索笔记..." />
                        </div>
                        <Button type="button" variant="outline" size="sm">
                            <Filter className="h-4 w-4" />
                            筛选
                        </Button>
                    </div>
                </div>
            </section>

            <section className="border-b bg-background p-4">
                <div className="rounded-lg border bg-card shadow-sm">
                    <div className="flex flex-wrap items-center gap-3 border-b px-4 py-3 text-muted-foreground">
                        <Smile className="h-4 w-4" />
                        <span className="text-lg font-semibold text-foreground">H</span>
                        <span className="text-base font-semibold text-foreground">B</span>
                        <span className="italic text-foreground">I</span>
                        <Link2 className="h-4 w-4" />
                        <List className="h-4 w-4" />
                        <CheckSquare className="h-4 w-4" />
                        <TableProperties className="h-4 w-4" />
                        <Hash className="h-4 w-4" />
                        <AtSign className="h-4 w-4" />
                        <Paperclip className="h-4 w-4" />
                    </div>
                    <Textarea
                        className="min-h-28 resize-none border-0 text-base shadow-none focus-visible:ring-0"
                        defaultValue={'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock'}
                    />
                    <div className="flex items-center justify-between border-t px-4 py-3">
                        <div className="flex flex-wrap items-center gap-3 text-muted-foreground">
                            <Zap className="h-4 w-4 fill-amber-400 text-amber-400" />
                            <Hash className="h-4 w-4" />
                            <Link2 className="h-4 w-4" />
                            <Paperclip className="h-4 w-4" />
                            <Sparkles className="h-4 w-4" />
                        </div>
                        <Button type="button" className="rounded-xl">
                            <Send className="h-4 w-4" />
                            记录
                        </Button>
                    </div>
                </div>
            </section>

            <section className="grid gap-4 p-4 xl:grid-cols-2">
                {NOTE_CARDS.map((note) => (
                    <article key={`${note.time}-${note.body}`} className="rounded-xl border bg-card p-4 shadow-sm">
                        <div className="flex items-start justify-between gap-4">
                            <time className="text-sm text-muted-foreground">{note.time}</time>
                            <Button type="button" variant="ghost" size="icon" className="h-7 w-7">
                                <span className="text-lg leading-none">...</span>
                            </Button>
                        </div>
                        <p className="mt-3 text-base leading-7 text-foreground">{note.body}</p>
                        <div className="mt-4 flex flex-wrap gap-2">
                            {note.tags.map((tag) => (
                                <Badge key={tag} variant="secondary">
                                    {tag}
                                </Badge>
                            ))}
                        </div>
                    </article>
                ))}
            </section>
        </div>
    );
}

function AssetDataTable({ data }: { data: AssetRecord[] }) {
    const table = useReactTable({
        data,
        columns: assetColumns,
        getCoreRowModel: getCoreRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
        initialState: {
            pagination: {
                pageSize: 5,
            },
        },
    });

    return (
        <div className="space-y-0">
            <div className="flex items-center justify-between gap-3 border-b px-5 py-3">
                <div className="relative min-w-0 flex-1">
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input className="h-9 max-w-sm pl-9" placeholder="筛选资产..." />
                </div>
                <Button type="button" variant="outline" size="sm">
                    <Filter className="h-4 w-4" />
                    视图
                </Button>
            </div>
            <div className="bg-background">
                <Table>
                    <TableHeader className="bg-muted/40 text-xs uppercase tracking-[0.14em]">
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => (
                                    <TableHead key={header.id} className="px-5">
                                        {header.isPlaceholder
                                            ? null
                                            : flexRender(
                                                  header.column.columnDef.header,
                                                  header.getContext(),
                                              )}
                                    </TableHead>
                                ))}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows.map((row) => (
                            <TableRow key={row.id}>
                                {row.getVisibleCells().map((cell) => (
                                    <TableCell key={cell.id} className="px-5 py-3">
                                        {flexRender(
                                            cell.column.columnDef.cell,
                                            cell.getContext(),
                                        )}
                                    </TableCell>
                                ))}
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </div>
            <div className="flex items-center justify-between px-5 py-3 text-sm text-muted-foreground">
                <span>
                    {table.getRowModel().rows.length} 条记录
                </span>
                <div className="flex items-center gap-2">
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

function AssetSignal({ item, index }: { item: AssetModule; index: number }) {
    return (
        <div
            className={`px-5 py-3 ${
                index > 0 ? "border-t md:border-l md:border-t-0" : ""
            }`}
        >
            <div className="flex items-center gap-2 text-sm font-medium">
                <span className="text-muted-foreground">{item.icon}</span>
                {item.title}
            </div>
            <p className="mt-2 text-sm text-muted-foreground">{item.detail}</p>
        </div>
    );
}

function PriorityCard({ title, detail }: { title: string; detail: string }) {
    return (
        <div className="border px-4 py-3">
            <div className="text-sm font-medium">{title}</div>
            <p className="mt-2 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}
