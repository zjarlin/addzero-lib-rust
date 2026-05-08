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
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
    Textarea,
    cn,
} from "@addzero/ui";
import { getApiBaseUrl } from "@addzero/api-client";

type AssetModuleId =
    | "notes"
    | "packages"
    | "dotfiles";

type NoteWorkspaceView =
    | "inbox"
    | "tags"
    | "graph"
    | "organize";

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
    id: string;
    time: string;
    title: string;
    body: string;
    tags: string[];
    kind: string;
    source: string;
    accent: string;
    status: "captured" | "reviewing" | "resolved";
    cluster: string;
    structuredKind: string;
    promptContexts: string[];
    unmapped: string[];
    relations: Array<{
        target: string;
        relation: string;
        confidence: number;
    }>;
    draft: {
        title: string;
        summary: string;
        fields: Array<{
            label: string;
            value: string;
            tone?: "default" | "warning" | "success";
        }>;
    };
}

interface TagNode {
    label: string;
    noteIds: string[];
    neighbors: string[];
}

interface GraphLine {
    source: string;
    target: string;
    weight: number;
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

const NOTE_CARDS: NoteCardData[] = [
    {
        id: "dock-hot-corners",
        time: "2026-05-07 20:14:00",
        title: "关闭 macOS 四角热区命令",
        body: 'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock',
        tags: ["闪念", "macOS", "dock"],
        kind: "Command",
        source: "剪贴板",
        accent: "bg-amber-400",
        status: "reviewing",
        cluster: "macOS 运维",
        structuredKind: "snippet",
        promptContexts: ["设备: Mac mini", "主题: 系统初始化", "偏好: 可复制命令"],
        unmapped: ['“tl tr bl br” 需要转成字段还是保留命令原样', "是否挂到 dotfiles 还是教程集合"],
        relations: [
            { target: "macOS 初始化手册", relation: "可并入", confidence: 88 },
            { target: "~/.zshrc", relation: "相关环境", confidence: 54 },
        ],
        draft: {
            title: "macOS 关闭四角热区",
            summary: "一条可直接执行的系统偏好修正命令，适合归入 macOS 初始化片段库。",
            fields: [
                { label: "标准类型", value: "snippet.command" },
                { label: "适用范围", value: "macOS / Dock" },
                { label: "执行方式", value: "终端一次性执行" },
                { label: "建议归档", value: "reference/macOS-setup", tone: "success" },
            ],
        },
    },
    {
        id: "responses-tool-error",
        time: "2026-05-07 19:45:54",
        title: "Responses API 自定义工具报错",
        body: '{"error":{"code":"invalid_request","message":"unsupported responses tool type custom","type":"invalid_request_error"}}',
        tags: ["debug", "api", "tool"],
        kind: "Error",
        source: "运行日志",
        accent: "bg-rose-400",
        status: "captured",
        cluster: "AI 调试",
        structuredKind: "log",
        promptContexts: ["当前页面 prompt: API 工具兼容性", "提供商: OpenAI"],
        unmapped: ["是 SDK 约束还是模型约束", "是否需要附上当时请求体"],
        relations: [
            { target: "OpenAI tool 兼容性笔记", relation: "应引用", confidence: 73 },
            { target: "AI Provider 配置", relation: "排障对象", confidence: 64 },
        ],
        draft: {
            title: "unsupported responses tool type custom",
            summary: "一条错误日志，适合进入调试记录而不是正式教程；需要补充上下文后再定类。",
            fields: [
                { label: "标准类型", value: "log.error" },
                { label: "来源", value: "runtime / API response" },
                { label: "建议状态", value: "保留待补证据", tone: "warning" },
                { label: "建议归档", value: "debug/openai-tools" },
            ],
        },
    },
    {
        id: "mac-mini-vnc",
        time: "2026-05-07 12:59:28",
        title: "Mac mini 屏幕共享连接方式",
        body: "如果远端是 Mac mini，macOS 自带的屏幕共享可以直接连接。先在远端开启服务，本地 Finder 里按 Command + K，输入 vnc://远端IP。",
        tags: ["remote", "macOS", "操作"],
        kind: "Howto",
        source: "人工输入",
        accent: "bg-emerald-400",
        status: "reviewing",
        cluster: "远程接入",
        structuredKind: "reference",
        promptContexts: ["远程机器: Mac mini", "主题: 远程接管", "关联页面: assets/dotfiles"],
        unmapped: ["是否补充权限开启路径", "需要账号密码信息还是只留方法"],
        relations: [
            { target: "Mac mini 运维清单", relation: "并入章节", confidence: 91 },
            { target: "家庭网络拓扑", relation: "依赖环境", confidence: 42 },
        ],
        draft: {
            title: "Mac mini 屏幕共享连接",
            summary: "远程接入教程，适合沉淀为 reference，并挂到设备实体和网络环境。",
            fields: [
                { label: "标准类型", value: "reference.howto" },
                { label: "主体对象", value: "device: Mac mini" },
                { label: "操作入口", value: "Finder -> Command + K -> vnc://IP" },
                { label: "建议归档", value: "reference/remote-access", tone: "success" },
            ],
        },
    },
    {
        id: "all-in-one-idea",
        time: "2026-05-04 23:28:38",
        title: "all in one 资产工作台想法",
        body: "头脑风暴：我希望有个 all in one 的东西，能把笔记、安装包、密码本、skill、配置文件之类的管理起来。",
        tags: ["资产", "idea", "aio"],
        kind: "Idea",
        source: "闪念",
        accent: "bg-blue-400",
        status: "resolved",
        cluster: "产品方向",
        structuredKind: "decision",
        promptContexts: ["产品愿景", "资产域边界", "跨模块治理"],
        unmapped: [],
        relations: [
            { target: "Asset Workbench 路线", relation: "已并入", confidence: 97 },
            { target: "Knowledge Workbench", relation: "相邻域", confidence: 68 },
        ],
        draft: {
            title: "统一资产治理工作台",
            summary: "已沉淀为产品方向，不再参加整理，只保留来源和关系以便追溯。",
            fields: [
                { label: "标准类型", value: "decision.product" },
                { label: "正式状态", value: "已入库", tone: "success" },
                { label: "来源追踪", value: "capture/all-in-one-idea" },
                { label: "当前归档", value: "library/asset-workbench-vision" },
            ],
        },
    },
    {
        id: "strum-snippet",
        time: "2026-05-02 23:36:18",
        title: "strum derive 速记",
        body: 'strum = { version = "0.26", features = ["derive"] } 以后写 rust 记得这个库可以零成本抽象 rust 简化。',
        tags: ["rust", "crate", "闪念"],
        kind: "Snippet",
        source: "手记",
        accent: "bg-lime-400",
        status: "captured",
        cluster: "Rust 生态",
        structuredKind: "reference",
        promptContexts: ["语言: Rust", "目标: 枚举/derive 简化"],
        unmapped: ["‘零成本抽象’ 是经验判断还是要附具体用法", "要不要挂到 crate 索引"],
        relations: [
            { target: "Rust crate 清单", relation: "候选归档", confidence: 78 },
            { target: "枚举模式笔记", relation: "潜在引用", confidence: 61 },
        ],
        draft: {
            title: "strum derive 依赖提示",
            summary: "目前更像零碎提醒，不够成文；需要补适用场景和示例后再入教程库。",
            fields: [
                { label: "标准类型", value: "reference.snippet" },
                { label: "当前风险", value: "缺少用例与版本上下文", tone: "warning" },
                { label: "建议动作", value: "补充示例后再入库" },
                { label: "候选归档", value: "reference/rust-crates" },
            ],
        },
    },
    {
        id: "bi-axial-rule",
        time: "2026-05-01 10:08:11",
        title: "二维上下文导航规则",
        body: "UI 规则：顶部主轴和左侧路由树是二维上下文，不要把工作台、资产、运行混成一个平铺菜单。",
        tags: ["admin", "导航", "规则"],
        kind: "Decision",
        source: "AGENTS.md",
        accent: "bg-stone-900",
        status: "resolved",
        cluster: "Admin 规则",
        structuredKind: "decision",
        promptContexts: ["admin shell", "导航规范", "provider 分层"],
        unmapped: [],
        relations: [
            { target: "Admin Navigation Convention", relation: "正式来源", confidence: 100 },
            { target: "assets/notes 页面", relation: "直接约束", confidence: 88 },
        ],
        draft: {
            title: "Admin 二维上下文导航",
            summary: "已是正式规则来源，前端整理台应该把这条视作 library 节点，而非再次整理的碎片。",
            fields: [
                { label: "标准类型", value: "decision.ui" },
                { label: "正式状态", value: "规则已生效", tone: "success" },
                { label: "来源", value: "AGENTS.md / Admin Navigation Convention" },
                { label: "当前归档", value: "library/admin-navigation" },
            ],
        },
    },
];

const NOTE_WORKSPACE_TABS = [
    { value: "inbox", label: "原始碎片 Inbox" },
    { value: "tags", label: "标签节点 Tags" },
    { value: "graph", label: "知识连线 Graph" },
    { value: "organize", label: "标签整理 Organize" },
] as const;

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

function NotesWorkbench({ activeModule }: { activeModule: AssetModule }) {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const [activeView, setActiveView] = useState<NoteWorkspaceView>("tags");
    const [search, setSearch] = useState("");
    const [notes, setNotes] = useState(NOTE_CARDS);
    const [selectedNoteId, setSelectedNoteId] = useState(NOTE_CARDS[0]?.id ?? "");
    const [activeTag, setActiveTag] = useState("");
    const [captureDraft, setCaptureDraft] = useState(
        'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock',
    );
    const [seedTags, setSeedTags] = useState<string[]>(["闪念", "代码", "macOS"]);
    const [captureSaving, setCaptureSaving] = useState(false);
    const [captureMessage, setCaptureMessage] = useState<string | null>(null);
    const [captureError, setCaptureError] = useState<string | null>(null);

    const filteredNotes = useMemo(() => {
        const query = search.trim().toLowerCase();
        if (!query) {
            return notes;
        }
        return notes.filter((note) =>
            [note.title, note.body, note.source, ...note.tags].some((value) =>
                value.toLowerCase().includes(query),
            ),
        );
    }, [notes, search]);

    const selectedNote = useMemo(
        () => notes.find((note) => note.id === selectedNoteId) ?? notes[0],
        [notes, selectedNoteId],
    );
    const tagNodes = useMemo(() => deriveTagNodes(notes), [notes]);
    const tagGraph = useMemo(() => buildTagGraph(notes), [notes]);
    const resolvedActiveTag =
        activeTag || tagNodes[0]?.label || selectedNote?.tags[0] || "闪念";
    const tagFocusedNotes = useMemo(
        () => filteredNotes.filter((note) => note.tags.includes(resolvedActiveTag)),
        [filteredNotes, resolvedActiveTag],
    );
    const connectedTags = useMemo(
        () => dfsTags(resolvedActiveTag, tagGraph),
        [resolvedActiveTag, tagGraph],
    );
    const graphLines = useMemo(() => deriveGraphLines(notes), [notes]);
    const noteMetrics: AssetMetric[] = useMemo(
        () => [
            { label: "碎片", value: String(notes.length), detail: "raw notes" },
            { label: "标签", value: String(tagNodes.length), detail: "tag nodes" },
            { label: "连线", value: String(graphLines.length), detail: "tag lines" },
            {
                label: "待归并",
                value: String(tagNodes.filter((tag) => tag.noteIds.length > 1).length),
                detail: "cluster actions",
            },
        ],
        [graphLines.length, notes.length, tagNodes],
    );
    const tabCounts = useMemo(
        () => ({
            inbox: filteredNotes.length,
            tags: tagNodes.length,
            graph: graphLines.length,
            organize: connectedTags.length,
        }),
        [connectedTags.length, filteredNotes.length, graphLines.length, tagNodes.length],
    );

    function toggleSeedTag(tag: string) {
        setSeedTags((current) =>
            current.includes(tag)
                ? current.filter((item) => item !== tag)
                : [...current, tag],
        );
    }

    async function handleCapture() {
        const body = captureDraft.trim();
        if (!body) {
            return;
        }
        const tags = dedupeTags(seedTags);
        const title = deriveNoteTitle(body);
        const resetComposer = () => {
            setCaptureDraft("");
            setSeedTags(["闪念"]);
        };
        const pushCapturedNote = (note: NoteCardData) => {
            setNotes((current) => [note, ...current]);
            setSelectedNoteId(note.id);
            setActiveTag(note.tags[0] ?? "");
            setActiveView("inbox");
        };
        setCaptureSaving(true);
        setCaptureMessage(null);
        setCaptureError(null);
        try {
            const response = await fetch(`${baseUrl}/api/knowledge/entries`, {
                method: "POST",
                credentials: "include",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    source_path: "",
                    relative_path: "",
                    title,
                    body,
                    tags,
                }),
            });
            if (!response.ok) {
                const text = await response.text();
                throw new Error(text || `HTTP ${response.status}`);
            }
            const saved = (await response.json()) as {
                source_path: string;
                relative_path: string;
                title: string;
            };
            const note = buildCapturedNote(body, tags, {
                title: saved.title,
                source: "笔记工作台",
                relatedTarget: saved.relative_path,
            });
            pushCapturedNote(note);
            resetComposer();
            setCaptureMessage(`已记录到 ${saved.relative_path}`);
        } catch (error) {
            const fallbackNote = buildCapturedNote(body, tags, {
                title,
                source: "本地会话",
            });
            pushCapturedNote(fallbackNote);
            resetComposer();
            setCaptureError(
                error instanceof Error
                    ? `知识库同步失败，已先保存在当前页面：${error.message}`
                    : "知识库同步失败，已先保存在当前页面。",
            );
        } finally {
            setCaptureSaving(false);
        }
    }

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
                            value={search}
                            onChange={(event) => setSearch(event.target.value)}
                            className="h-10 rounded-2xl border-stone-300 bg-white/80 pl-9 shadow-sm"
                            placeholder="搜索笔记、标签、来源..."
                        />
                    </div>
                </div>
            </section>

            <section className="min-h-[calc(100vh-4.25rem)]">
                <div className="grid grid-cols-2 border-b bg-[#efeadf] lg:grid-cols-4">
                    {noteMetrics.map((metric) => (
                        <MetricTile key={metric.label} metric={metric} flush />
                    ))}
                </div>
                <QuickCapture
                    value={captureDraft}
                    seedTags={seedTags}
                    saving={captureSaving}
                    message={captureMessage}
                    error={captureError}
                    onChange={setCaptureDraft}
                    onToggleTag={toggleSeedTag}
                    onSubmit={handleCapture}
                />

                <div className="border-b bg-[#fbfaf4] px-4 py-3 lg:px-5">
                    <div className="flex flex-wrap items-center justify-between gap-3">
                        <div>
                            <div className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                                Tag Flow
                            </div>
                            <h2 className="mt-1 text-lg font-semibold">先记录，再打标签，再基于标签聚合整理</h2>
                            <p className="mt-1 text-sm text-muted-foreground">
                                笔记先保持非结构化原样，只做标签和连线；后续整理从标签簇一键发起，不强行先建模。
                            </p>
                        </div>
                        <div className="flex flex-wrap gap-2">
                            {tagNodes.slice(0, 6).map((tag, index) => (
                                    <Badge
                                        key={tag.label}
                                        variant={
                                            index === 0 || tag.label === resolvedActiveTag
                                                ? "default"
                                                : "outline"
                                        }
                                        className="rounded-full"
                                        onClick={() => setActiveTag(tag.label)}
                                    >
                                        #{tag.label}
                                    </Badge>
                                ))}
                            <span className="flex items-center gap-2 rounded-full border bg-white/80 px-3 py-1 text-xs text-muted-foreground">
                                <Clock3 className="h-3.5 w-3.5" />
                                当前标签簇 {connectedTags.length} 个节点，{graphLines.length} 条连线
                            </span>
                        </div>
                    </div>
                </div>

                <div className="p-4 lg:p-5">
                    <Tabs
                        value={activeView}
                        onValueChange={(value) => setActiveView(value as NoteWorkspaceView)}
                        className="space-y-4"
                    >
                        <TabsList className="grid h-auto w-full grid-cols-2 gap-2 rounded-3xl bg-[#ece6d8] p-2 lg:grid-cols-4">
                            {NOTE_WORKSPACE_TABS.map((tab) => (
                                <TabsTrigger
                                    key={tab.value}
                                    value={tab.value}
                                    className="rounded-2xl px-3 py-3 text-left data-[state=active]:bg-white"
                                >
                                    <span className="flex w-full items-center justify-between gap-3">
                                        <span className="text-xs font-semibold uppercase tracking-[0.12em]">
                                            {tab.label}
                                        </span>
                                        <Badge variant="outline" className="rounded-full bg-white/80">
                                            {tabCounts[tab.value]}
                                        </Badge>
                                    </span>
                                </TabsTrigger>
                            ))}
                        </TabsList>

                        <TabsContent value="inbox" className="mt-0">
                            <section className="columns-1 gap-4 lg:columns-2 2xl:columns-3">
                                {filteredNotes.map((note) => (
                                    <NoteCard
                                        key={note.id}
                                        note={note}
                                        selected={note.id === selectedNote?.id}
                                        onSelect={() => {
                                            setSelectedNoteId(note.id);
                                            setActiveTag(note.tags[0] ?? "");
                                        }}
                                    />
                                ))}
                            </section>
                        </TabsContent>

                        <TabsContent value="tags" className="mt-0">
                            <div className="grid gap-4 xl:grid-cols-[20rem_minmax(0,1fr)]">
                                <Card className="rounded-3xl border-stone-300 bg-[#fffdf8] shadow-sm">
                                    <CardHeader className="p-4 pb-3">
                                        <CardTitle className="flex items-center gap-2 text-sm">
                                            <Tags className="h-4 w-4 text-stone-700" />
                                            标签节点
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-3 p-4 pt-0">
                                        {tagNodes.map((tag) => (
                                            <button
                                                key={tag.label}
                                                type="button"
                                                onClick={() => setActiveTag(tag.label)}
                                                className={cn(
                                                    "w-full rounded-2xl border px-3 py-3 text-left transition",
                                                    resolvedActiveTag === tag.label
                                                        ? "border-stone-900 bg-stone-900 text-stone-50"
                                                        : "border-stone-200 bg-white hover:bg-stone-50",
                                                )}
                                            >
                                                <div className="flex items-center justify-between gap-3">
                                                    <span className="font-medium">#{tag.label}</span>
                                                    <Badge variant="outline" className="rounded-full">
                                                        {tag.noteIds.length}
                                                    </Badge>
                                                </div>
                                                <div
                                                    className={cn(
                                                        "mt-2 text-xs",
                                                        resolvedActiveTag === tag.label
                                                            ? "text-stone-300"
                                                            : "text-muted-foreground",
                                                    )}
                                                >
                                                    直连 {tag.neighbors.length} 个标签
                                                </div>
                                            </button>
                                        ))}
                                    </CardContent>
                                </Card>

                                <div className="space-y-4">
                                    <Card className="rounded-3xl border-stone-300 bg-white shadow-sm">
                                        <CardHeader className="p-4 pb-3">
                                            <CardTitle className="flex items-center gap-2 text-sm">
                                                <Hash className="h-4 w-4 text-amber-600" />
                                                #{resolvedActiveTag}
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-4 p-4 pt-0">
                                            <div className="flex flex-wrap gap-2">
                                                {connectedTags.map((tag) => (
                                                    <Badge
                                                        key={tag}
                                                        variant={tag === resolvedActiveTag ? "default" : "outline"}
                                                        className="rounded-full"
                                                    >
                                                        #{tag}
                                                    </Badge>
                                                ))}
                                            </div>
                                            <div className="grid gap-3 lg:grid-cols-2">
                                                {tagFocusedNotes.map((note) => (
                                                    <NoteCard
                                                        key={note.id}
                                                        note={note}
                                                        compact
                                                        selected={note.id === selectedNote?.id}
                                                        onSelect={() => setSelectedNoteId(note.id)}
                                                    />
                                                ))}
                                            </div>
                                        </CardContent>
                                    </Card>
                                </div>
                            </div>
                        </TabsContent>

                        <TabsContent value="graph" className="mt-0">
                            <div className="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_20rem]">
                                <Card className="rounded-3xl border-stone-300 bg-white shadow-sm">
                                    <CardHeader className="p-4 pb-3">
                                        <CardTitle className="flex items-center gap-2 text-sm">
                                            <Layers3 className="h-4 w-4 text-blue-600" />
                                            标签线图
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-3 p-4 pt-0">
                                        {graphLines.map((line) => (
                                            <div
                                                key={`${line.source}-${line.target}`}
                                                className="rounded-2xl border bg-[#fcfbf7] p-3"
                                            >
                                                <div className="flex items-center justify-between gap-3">
                                                    <div className="text-sm font-medium">
                                                        #{line.source} <span className="text-muted-foreground">↔</span> #{line.target}
                                                    </div>
                                                    <Badge variant="outline" className="rounded-full">
                                                        {line.weight}
                                                    </Badge>
                                                </div>
                                                <div className="mt-2 h-2 rounded-full bg-stone-200">
                                                    <div
                                                        className="h-2 rounded-full bg-stone-900"
                                                        style={{ width: `${Math.min(100, line.weight * 22)}%` }}
                                                    />
                                                </div>
                                            </div>
                                        ))}
                                    </CardContent>
                                </Card>

                                <Card className="rounded-3xl border-stone-300 bg-[#fffdf8] shadow-sm">
                                    <CardHeader className="p-4 pb-3">
                                        <CardTitle className="flex items-center gap-2 text-sm">
                                            <Link2 className="h-4 w-4 text-emerald-600" />
                                            DFS 聚合
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-3 p-4 pt-0">
                                        <SpecLine label="起点标签" value={`#${resolvedActiveTag}`} />
                                        <SpecLine label="可达节点" value={`${connectedTags.length} 个`} />
                                        <SpecLine label="片段数" value={`${tagFocusedNotes.length} 条`} />
                                        <div className="flex flex-wrap gap-2">
                                            {connectedTags.map((tag) => (
                                                <Badge key={tag} variant="secondary" className="rounded-full">
                                                    #{tag}
                                                </Badge>
                                            ))}
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>
                        </TabsContent>

                        <TabsContent value="organize" className="mt-0">
                            <div className="grid gap-4 xl:grid-cols-[18rem_minmax(0,1fr)_18rem]">
                                <Card className="rounded-3xl border-stone-300 bg-[#fffdf8] shadow-sm">
                                    <CardHeader className="p-4 pb-3">
                                        <CardTitle className="flex items-center gap-2 text-sm">
                                            <Inbox className="h-4 w-4 text-amber-600" />
                                            标签入口
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-3 p-4 pt-0">
                                        {tagNodes.map((tag) => (
                                            <button
                                                key={`${tag.label}-organize`}
                                                type="button"
                                                onClick={() => setActiveTag(tag.label)}
                                                className={cn(
                                                    "w-full rounded-2xl border px-3 py-3 text-left transition",
                                                    resolvedActiveTag === tag.label
                                                        ? "border-stone-900 bg-stone-900 text-stone-50"
                                                        : "border-stone-200 bg-white hover:bg-stone-50",
                                                )}
                                            >
                                                <div className="font-medium">#{tag.label}</div>
                                                <div
                                                    className={cn(
                                                        "mt-1 text-xs",
                                                        resolvedActiveTag === tag.label
                                                            ? "text-stone-300"
                                                            : "text-muted-foreground",
                                                    )}
                                                >
                                                    {tag.noteIds.length} 条碎片 / {tag.neighbors.length} 个邻居
                                                </div>
                                            </button>
                                        ))}
                                    </CardContent>
                                </Card>

                                <div className="space-y-4">
                                    <Card className="rounded-3xl border-stone-300 bg-white shadow-sm">
                                        <CardHeader className="p-4 pb-3">
                                            <CardTitle className="flex items-center gap-2 text-sm">
                                                <FileText className="h-4 w-4 text-blue-600" />
                                                标签驱动整理
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-4 p-4 pt-0">
                                            <div className="rounded-2xl border border-dashed border-stone-300 bg-[#fbfaf5] p-4">
                                                <div className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                                                    Organize By Tag
                                                </div>
                                                <p className="mt-3 text-sm leading-7 text-stone-900">
                                                    以 <strong>#{resolvedActiveTag}</strong> 为起点，把可达标签簇内的碎片一起归并成一个主题视图。
                                                    这里不先做字段建模，只做去重、摘要、标签收敛和知识线串联。
                                                </p>
                                            </div>
                                            <div className="flex flex-wrap gap-2">
                                                {connectedTags.map((tag) => (
                                                    <Badge key={tag} variant="secondary" className="rounded-full">
                                                        #{tag}
                                                    </Badge>
                                                ))}
                                            </div>
                                            <div className="grid gap-3">
                                                {tagFocusedNotes.map((note) => (
                                                    <NoteCard
                                                        key={`${note.id}-organize`}
                                                        note={note}
                                                        compact
                                                        selected={note.id === selectedNote?.id}
                                                        onSelect={() => setSelectedNoteId(note.id)}
                                                    />
                                                ))}
                                            </div>
                                        </CardContent>
                                    </Card>
                                </div>

                                <div className="space-y-4">
                                    <Card className="rounded-3xl border-stone-300 bg-white shadow-sm">
                                        <CardHeader className="p-4 pb-3">
                                            <CardTitle className="flex items-center gap-2 text-sm">
                                                <CheckSquare className="h-4 w-4 text-emerald-600" />
                                                一键动作
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-2 p-4 pt-0">
                                            <Button type="button" className="w-full rounded-2xl justify-start">
                                                合并重复碎片并提炼结论
                                            </Button>
                                            <Button type="button" variant="outline" className="w-full rounded-2xl justify-start">
                                                输出标签簇摘要
                                            </Button>
                                            <Button type="button" variant="outline" className="w-full rounded-2xl justify-start">
                                                生成主题页草稿
                                            </Button>
                                        </CardContent>
                                    </Card>

                                    <Card className="rounded-3xl border-stone-800 bg-[#171a17] text-stone-50 shadow-sm">
                                        <CardHeader className="p-4 pb-3">
                                            <CardTitle className="flex items-center gap-2 text-sm">
                                                <ShieldCheck className="h-4 w-4 text-amber-300" />
                                                你的思路
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-3 p-4 pt-0 text-sm text-stone-300">
                                            <p>先把碎片留在原样文本里，只打标签。</p>
                                            <p>node 是标签，line 是标签共现或引用，DFS 负责把关联碎片自动聚到标签簇。</p>
                                            <p>整理动作发生在标签簇上，而不是每条碎片单独填表。</p>
                                        </CardContent>
                                    </Card>
                                </div>
                            </div>
                        </TabsContent>
                    </Tabs>
                </div>
            </section>
        </div>
    );
}

function QuickCapture({
    value,
    seedTags,
    saving,
    message,
    error,
    onChange,
    onToggleTag,
    onSubmit,
}: {
    value: string;
    seedTags: string[];
    saving: boolean;
    message: string | null;
    error: string | null;
    onChange: (value: string) => void;
    onToggleTag: (tag: string) => void;
    onSubmit: () => Promise<void>;
}) {
    const presetTags = ["闪念", "代码", "教程", "账号", "密钥", "网站", "游戏", "决策"];

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
                    value={value}
                    onChange={(event) => onChange(event.target.value)}
                    onKeyDown={(event) => {
                        if ((event.metaKey || event.ctrlKey) && event.key === "Enter" && !saving) {
                            event.preventDefault();
                            void onSubmit();
                        }
                    }}
                />
                <div className="flex flex-wrap items-center justify-between gap-2 border-t bg-[#fffdf7] px-3 py-2 lg:gap-3 lg:px-4 lg:py-3">
                    <div className="flex flex-wrap items-center gap-2 text-muted-foreground">
                        {presetTags.map((tag) => (
                            <Button
                                key={tag}
                                type="button"
                                variant={seedTags.includes(tag) ? "default" : "ghost"}
                                size="sm"
                                className="rounded-full"
                                onClick={() => onToggleTag(tag)}
                            >
                                {tag === "闪念" ? (
                                    <Zap className="h-4 w-4 fill-amber-400 text-amber-400" />
                                ) : tag === "代码" ? (
                                    <Code2 className="h-4 w-4" />
                                ) : (
                                    <Hash className="h-4 w-4" />
                                )}
                                {tag}
                            </Button>
                        ))}
                    </div>
                    <Button
                        type="button"
                        className="rounded-2xl px-5"
                        onClick={() => void onSubmit()}
                        disabled={!value.trim() || saving}
                    >
                        <Send className="h-4 w-4" />
                        {saving ? "记录中" : "记录"}
                    </Button>
                </div>
                {(message || error) && (
                    <div className="border-t bg-[#fffdf7] px-4 py-3 text-sm">
                        {message ? (
                            <p className="text-emerald-700">{message}</p>
                        ) : (
                            <p className="text-rose-700">{error}</p>
                        )}
                    </div>
                )}
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

function buildCapturedNote(
    body: string,
    seedTags: string[],
    options?: {
        title?: string;
        source?: string;
        relatedTarget?: string;
    },
): NoteCardData {
    const tags = deriveTagsFromCapture(body, seedTags);
    const title = options?.title?.trim() || inferNoteTitle(body);
    const mainTag = tags.find((tag) => tag !== "闪念") ?? tags[0] ?? "未整理";
    return {
        id: `capture-${Date.now()}`,
        time: formatNoteTimestamp(new Date()),
        title,
        body,
        tags,
        kind: "Fragment",
        source: options?.source || "人工记录",
        accent: accentFromTag(mainTag),
        status: "captured",
        cluster: `${mainTag} 标签簇`,
        structuredKind: "tagged-note",
        promptContexts: tags.map((tag) => `tag:${tag}`),
        unmapped: [
            "等待后续按标签簇聚合整理",
            "当前仅完成原文记录，尚未生成主题页",
        ],
        relations: [
            {
                target: options?.relatedTarget || `capture/${slugifyTitle(title)}`,
                relation: "新建记录",
                confidence: 100,
            },
            { target: `#${mainTag}`, relation: "标签入口", confidence: 96 },
        ],
        draft: {
            title,
            summary: summarizeBody(body),
            fields: [
                { label: "标准类型", value: "capture.raw" },
                { label: "当前状态", value: "已记录待整理", tone: "warning" },
                { label: "标签数", value: String(tags.length) },
                { label: "后续入口", value: `tag/${mainTag}`, tone: "success" },
            ],
        },
    };
}

function deriveTagsFromCapture(body: string, seedTags: string[]) {
    const inlineTags = Array.from(
        body.matchAll(/#([\p{L}\p{N}_-]+)/gu),
        (match) => match[1],
    );
    return Array.from(
        new Set(
            [...seedTags, ...inlineTags]
                .map((tag) => tag.trim())
                .filter(Boolean),
        ),
    );
}

function inferNoteTitle(body: string) {
    const firstLine = body
        .split("\n")
        .map((line) => line.trim())
        .find(Boolean) ?? "未命名碎片";
    return firstLine.length > 28 ? `${firstLine.slice(0, 28)}…` : firstLine;
}

function formatNoteTimestamp(date: Date) {
    return date.toISOString().slice(0, 19).replace("T", " ");
}

function accentFromTag(tag: string) {
    if (["密钥", "账号"].includes(tag)) {
        return "bg-rose-400";
    }
    if (["代码", "rust", "macOS"].includes(tag)) {
        return "bg-amber-400";
    }
    if (["教程", "网站", "remote"].includes(tag)) {
        return "bg-emerald-400";
    }
    if (["决策", "规则"].includes(tag)) {
        return "bg-stone-900";
    }
    return "bg-blue-400";
}

function deriveTagNodes(notes: NoteCardData[]) {
    const index = new Map<string, { noteIds: Set<string>; neighbors: Set<string> }>();

    for (const note of notes) {
        for (const tag of note.tags) {
            const current = index.get(tag) ?? {
                noteIds: new Set<string>(),
                neighbors: new Set<string>(),
            };
            current.noteIds.add(note.id);
            for (const neighbor of note.tags) {
                if (neighbor !== tag) {
                    current.neighbors.add(neighbor);
                }
            }
            index.set(tag, current);
        }
    }

    return Array.from(index.entries())
        .map(([label, value]) => ({
            label,
            noteIds: Array.from(value.noteIds),
            neighbors: Array.from(value.neighbors).sort(),
        }))
        .sort((a, b) => b.noteIds.length - a.noteIds.length || a.label.localeCompare(b.label));
}

function buildTagGraph(notes: NoteCardData[]) {
    const graph = new Map<string, Set<string>>();
    for (const note of notes) {
        for (const tag of note.tags) {
            const neighbors = graph.get(tag) ?? new Set<string>();
            for (const other of note.tags) {
                if (other !== tag) {
                    neighbors.add(other);
                }
            }
            graph.set(tag, neighbors);
        }
    }
    return graph;
}

function dfsTags(start: string, graph: Map<string, Set<string>>) {
    if (!start) {
        return [];
    }
    const visited = new Set<string>();
    const stack = [start];
    while (stack.length) {
        const current = stack.pop()!;
        if (visited.has(current)) {
            continue;
        }
        visited.add(current);
        for (const next of graph.get(current) ?? []) {
            if (!visited.has(next)) {
                stack.push(next);
            }
        }
    }
    return Array.from(visited).sort();
}

function deriveGraphLines(notes: NoteCardData[]) {
    const weights = new Map<string, number>();
    for (const note of notes) {
        const tags = Array.from(new Set(note.tags)).sort();
        for (let index = 0; index < tags.length; index += 1) {
            for (let inner = index + 1; inner < tags.length; inner += 1) {
                const source = tags[index];
                const target = tags[inner];
                const key = `${source}::${target}`;
                weights.set(key, (weights.get(key) ?? 0) + 1);
            }
        }
    }

    return Array.from(weights.entries())
        .map(([key, weight]) => {
            const [source, target] = key.split("::");
            return { source, target, weight };
        })
        .sort((a, b) => b.weight - a.weight || a.source.localeCompare(b.source));
}

function NoteCard({
    note,
    selected = false,
    compact = false,
    onSelect,
}: {
    note: NoteCardData;
    selected?: boolean;
    compact?: boolean;
    onSelect?: () => void;
}) {
    return (
        <article
            className={cn(
                "mb-4 break-inside-avoid overflow-hidden rounded-3xl border bg-white shadow-[0_12px_28px_rgba(42,37,29,0.07)]",
                selected ? "border-stone-900 ring-1 ring-stone-900/10" : "border-stone-200",
                onSelect && "cursor-pointer",
            )}
            onClick={onSelect}
        >
            <div className={cn("h-1.5", note.accent)} />
            <div className={cn("p-4", compact && "p-3")}>
                <div className="flex items-start justify-between gap-3">
                    <div>
                        <time className="text-xs font-medium text-muted-foreground">
                            {note.time}
                        </time>
                        <div className="mt-1 flex flex-wrap items-center gap-2">
                            <Badge variant="outline" className="rounded-full">
                                {note.kind}
                            </Badge>
                            <StatusBadge
                                status={note.status === "captured" ? "Draft" : "Review"}
                            />
                            <span className="text-xs text-muted-foreground">
                                {note.source}
                            </span>
                        </div>
                    </div>
                    <Button type="button" variant="ghost" size="icon" className="h-8 w-8 rounded-full">
                        <MoreHorizontal className="h-4 w-4" />
                    </Button>
                </div>
                <div className="mt-4">
                    <h3 className={cn("text-base font-semibold text-stone-950", compact && "text-sm")}>
                        {note.title}
                    </h3>
                    <p className={cn("mt-2 text-[15px] leading-7 text-stone-900", compact && "line-clamp-4 text-sm leading-6")}>
                        {note.body}
                    </p>
                </div>
                <div className="mt-4 flex flex-wrap gap-2">
                    {note.tags.map((tag) => (
                        <Badge key={tag} variant="secondary" className="rounded-full">
                            #{tag}
                        </Badge>
                    ))}
                </div>
                <div className="mt-4 flex items-center justify-between gap-3 rounded-2xl border bg-[#fbfaf5] px-3 py-2 text-xs text-muted-foreground">
                    <span>{note.cluster}</span>
                    <span>{note.tags.length} tags</span>
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

function dedupeTags(tags: string[]): string[] {
    return Array.from(
        new Set(
            tags
                .map((tag) => tag.trim())
                .filter((tag) => tag.length > 0),
        ),
    );
}

function deriveNoteTitle(body: string): string {
    const firstLine = body
        .split("\n")
        .map((line) => line.trim())
        .find((line) => line.length > 0);
    if (!firstLine) {
        return "未命名笔记";
    }
    return firstLine.replace(/^#+\s*/, "").slice(0, 48);
}

function summarizeBody(body: string): string {
    const flat = body.replace(/\s+/g, " ").trim();
    if (!flat) {
        return "空白记录，待补充正文。";
    }
    return flat.slice(0, 84);
}

function detectNoteKind(body: string): string {
    if (body.includes("```") || body.includes("cargo ") || body.includes("defaults write")) {
        return "Command";
    }
    if (body.trim().startsWith("{") || body.trim().startsWith("[")) {
        return "Log";
    }
    if (body.includes("http://") || body.includes("https://")) {
        return "Reference";
    }
    return "Note";
}

function accentForTag(tag: string): string {
    if (tag.includes("代码") || tag.toLowerCase().includes("rust")) {
        return "bg-lime-400";
    }
    if (tag.includes("决策")) {
        return "bg-stone-900";
    }
    if (tag.includes("网站") || tag.includes("教程")) {
        return "bg-blue-400";
    }
    if (tag.includes("账号") || tag.includes("密钥")) {
        return "bg-rose-400";
    }
    if (tag.includes("闪念")) {
        return "bg-amber-400";
    }
    return "bg-emerald-400";
}

function formatLocalTimestamp(date: Date): string {
    const parts = [
        date.getFullYear(),
        padNumber(date.getMonth() + 1),
        padNumber(date.getDate()),
    ];
    const clock = [
        padNumber(date.getHours()),
        padNumber(date.getMinutes()),
        padNumber(date.getSeconds()),
    ];
    return `${parts.join("-")} ${clock.join(":")}`;
}

function padNumber(value: number): string {
    return value.toString().padStart(2, "0");
}

function slugifyTitle(value: string): string {
    return value
        .toLowerCase()
        .replace(/[^a-z0-9\u4e00-\u9fa5]+/g, "-")
        .replace(/^-+|-+$/g, "")
        .slice(0, 48) || "note";
}
