import { useEffect, useMemo, useRef, useState, type MouseEvent, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import "@mdxeditor/editor/style.css";
import {
    BlockTypeSelect,
    BoldItalicUnderlineToggles,
    CreateLink,
    ListsToggle,
    markdownShortcutPlugin,
    MDXEditor,
    type MDXEditorMethods,
    headingsPlugin,
    linkPlugin,
    listsPlugin,
    quotePlugin,
    toolbarPlugin,
    UndoRedo,
} from "@mdxeditor/editor";
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
    Check,
    CheckCircle2,
    CheckSquare,
    Database,
    FileArchive,
    FileText,
    Filter,
    FolderTree,
    Hash,
    Inbox,
    Layers3,
    Loader2,
    Link2,
    MoreHorizontal,
    PackageOpen,
    Plus,
    Search,
    Send,
    Settings2,
    ShieldCheck,
    SlidersHorizontal,
    Smile,
    Sparkles,
    Tags,
    Trash2,
    UploadCloud,
    X,
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
    cn,
} from "@addzero/ui";
import { getApiBaseUrl, type KnowledgeNoteDto } from "@addzero/api-client";

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
    sourcePath?: string;
    relativePath?: string;
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

const NOTE_EDITOR_PLUGINS = [
    headingsPlugin({ allowedHeadingLevels: [1, 2, 3] }),
    listsPlugin(),
    quotePlugin(),
    linkPlugin(),
    markdownShortcutPlugin(),
    toolbarPlugin({
        toolbarClassName:
            "!border-0 !border-b !border-stone-200 !bg-[#fffdf7] !px-3 !py-2 lg:!px-4 lg:!py-3",
        toolbarContents: () => (
            <>
                <UndoRedo />
                <BlockTypeSelect />
                <BoldItalicUnderlineToggles />
                <ListsToggle />
                <CreateLink />
            </>
        ),
    }),
] as const;

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
    const [notes, setNotes] = useState<NoteCardData[]>([]);
    const [notesLoading, setNotesLoading] = useState(true);
    const [notesLoadError, setNotesLoadError] = useState<string | null>(null);
    const [selectedNoteId, setSelectedNoteId] = useState("");
    const [activeTag, setActiveTag] = useState("");
    const [editingNoteId, setEditingNoteId] = useState<string | null>(null);
    const [editDrafts, setEditDrafts] = useState<Record<string, { title: string; body: string }>>({});
    const [savingNoteId, setSavingNoteId] = useState<string | null>(null);
    const [deletingNoteId, setDeletingNoteId] = useState<string | null>(null);
    const [autoTaggingNoteIds, setAutoTaggingNoteIds] = useState<string[]>([]);
    const [captureDraft, setCaptureDraft] = useState(
        'for c in tl tr bl br; do defaults write com.apple.dock "wvous-$c-corner" -int 0; defaults write com.apple.dock "wvous-$c-modifier" -int 0; done; killall Dock',
    );
    const [seedTags, setSeedTags] = useState<string[]>([]);
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
    const resolvedActiveTag = activeTag.trim();
    const tagFocusedNotes = useMemo(
        () =>
            resolvedActiveTag
                ? filteredNotes.filter((note) => note.tags.includes(resolvedActiveTag))
                : filteredNotes,
        [filteredNotes, resolvedActiveTag],
    );
    const connectedTags = useMemo(
        () => dfsTags(resolvedActiveTag, tagGraph),
        [resolvedActiveTag, tagGraph],
    );
    const graphLines = useMemo(() => deriveGraphLines(notes), [notes]);
    const tabCounts = useMemo(
        () => ({
            inbox: filteredNotes.length,
            tags: tagNodes.length,
            graph: graphLines.length,
            organize: connectedTags.length,
        }),
        [connectedTags.length, filteredNotes.length, graphLines.length, tagNodes.length],
    );
    const activeTagHeading = resolvedActiveTag ? `#${resolvedActiveTag}` : "全部标签";
    const activeTagSpecValue = resolvedActiveTag ? `#${resolvedActiveTag}` : "未选择";

    useEffect(() => {
        let cancelled = false;

        async function loadNotes() {
            setNotesLoading(true);
            setNotesLoadError(null);
            try {
                const response = await fetch(`${baseUrl}/api/knowledge/entries`, {
                    credentials: "include",
                });
                if (!response.ok) {
                    const text = await response.text();
                    throw new Error(text || `HTTP ${response.status}`);
                }
                const data = (await response.json()) as KnowledgeNoteDto[];
                if (cancelled) {
                    return;
                }
                const loadedNotes = data.map(mapPersistedNote).sort(
                    (left, right) => right.time.localeCompare(left.time),
                );
                setNotes(loadedNotes);
                setSelectedNoteId((current) =>
                    loadedNotes.some((note) => note.id === current)
                        ? current
                        : loadedNotes[0]?.id ?? "",
                );
                setActiveTag((current) =>
                    current && loadedNotes.some((note) => note.tags.includes(current))
                        ? current
                        : "",
                );
            } catch (error) {
                if (!cancelled) {
                    setNotes([]);
                    setNotesLoadError(
                        error instanceof Error
                            ? `加载笔记失败：${error.message}`
                            : "加载笔记失败。",
                    );
                }
            } finally {
                if (!cancelled) {
                    setNotesLoading(false);
                }
            }
        }

        void loadNotes();
        return () => {
            cancelled = true;
        };
    }, [baseUrl]);

    function beginEditingNote(note: NoteCardData) {
        setEditingNoteId(note.id);
        setEditDrafts((current) => ({
            ...current,
            [note.id]: current[note.id] ?? {
                title: note.title,
                body: note.body,
            },
        }));
    }

    function updateEditDraft(noteId: string, patch: Partial<{ title: string; body: string }>) {
        setEditDrafts((current) => ({
            ...current,
            [noteId]: {
                title: patch.title ?? current[noteId]?.title ?? "",
                body: patch.body ?? current[noteId]?.body ?? "",
            },
        }));
    }

    function stopEditingNote(noteId: string) {
        setEditingNoteId((current) => (current === noteId ? null : current));
        setEditDrafts((current) => {
            const next = { ...current };
            delete next[noteId];
            return next;
        });
    }

    async function persistKnowledgeEntry(input: {
        source_path: string;
        relative_path: string;
        title: string;
        body: string;
        tags: string[];
    }) {
        const response = await fetch(`${baseUrl}/api/knowledge/entries`, {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(input),
        });
        if (!response.ok) {
            const text = await response.text();
            throw new Error(text || `HTTP ${response.status}`);
        }
        return (await response.json()) as KnowledgeNoteDto;
    }

    function mergePersistedNote(saved: KnowledgeNoteDto, previousId?: string) {
        const mapped = mapPersistedNote(saved);
        setNotes((current) => {
            const targetIndex = current.findIndex(
                (item) =>
                    item.id === previousId ||
                    item.id === mapped.id ||
                    (mapped.sourcePath && item.sourcePath === mapped.sourcePath),
            );
            const nextNotes =
                targetIndex >= 0
                    ? current.map((item, index) => (index === targetIndex ? mapped : item))
                    : [mapped, ...current];
            return nextNotes.sort((left, right) => right.time.localeCompare(left.time));
        });
        return mapped;
    }

    function updateAutoTagging(noteId: string, pending: boolean) {
        setAutoTaggingNoteIds((current) => {
            if (pending) {
                return current.includes(noteId) ? current : [...current, noteId];
            }
            return current.filter((item) => item !== noteId);
        });
    }

    async function requestAiTags(title: string, body: string) {
        const response = await fetch(`${baseUrl}/api/ai/chat`, {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                messages: [
                    {
                        role: "system",
                        content: [
                            "你是笔记标签整理器。",
                            "请只返回 JSON，不要返回 Markdown、解释或额外文字。",
                            '输出格式必须是 {"tags":["标签1","标签2"]}。',
                            "标签要求：2 到 5 个，短词，中文优先，可混合必要英文技术词。",
                            "优先复用已有标签池；只有在明显不合适时才补充新标签。",
                            `当前已有标签池：${tagNodes.map((tag) => tag.label).join("、") || "暂无"}`,
                        ].join("\n"),
                    },
                    {
                        role: "user",
                        content: [`标题：${title}`, "", "正文：", body.slice(0, 4000)].join("\n"),
                    },
                ],
            }),
        });
        if (!response.ok) {
            const text = await response.text();
            throw new Error(text || `HTTP ${response.status}`);
        }
        const payload = (await response.json()) as {
            message?: {
                content?: string;
            };
        };
        return parseAiTagList(payload.message?.content ?? "");
    }

    async function autoTagPersistedNote(note: KnowledgeNoteDto) {
        const noteId = note.source_path || note.slug;
        updateAutoTagging(noteId, true);
        try {
            const tags = await requestAiTags(note.title, note.body);
            if (tags.length === 0) {
                return;
            }
            const saved = await persistKnowledgeEntry({
                source_path: note.source_path,
                relative_path: note.relative_path,
                title: note.title,
                body: note.body,
                tags,
            });
            const mapped = mergePersistedNote(saved, noteId);
            setSelectedNoteId((current) => (current === noteId ? mapped.id : current));
            setActiveTag((current) => current || mapped.tags[0] || "");
        } catch (error) {
            setCaptureMessage(null);
            setCaptureError(
                error instanceof Error
                    ? `笔记已入库，但 AI 打标失败：${error.message}`
                    : "笔记已入库，但 AI 打标失败。",
            );
        } finally {
            updateAutoTagging(noteId, false);
        }
    }

    function removeNoteFromState(noteId: string) {
        const noteIndex = notes.findIndex((note) => note.id === noteId);
        if (noteIndex < 0) {
            return;
        }
        const nextNotes = notes.filter((note) => note.id !== noteId);
        const fallbackNote =
            nextNotes[Math.min(noteIndex, Math.max(nextNotes.length - 1, 0))] ??
            nextNotes[0];
        setNotes(nextNotes);
        if (selectedNoteId === noteId) {
            setSelectedNoteId(fallbackNote?.id ?? "");
        }
        setActiveTag((current) =>
            current && !nextNotes.some((note) => note.tags.includes(current)) ? "" : current,
        );
        updateAutoTagging(noteId, false);
        stopEditingNote(noteId);
    }

    async function handleDeleteNote(note: NoteCardData) {
        if (deletingNoteId || savingNoteId) {
            return;
        }
        setDeletingNoteId(note.id);
        setCaptureMessage(null);
        setCaptureError(null);
        try {
            const sourcePath = note.sourcePath?.trim();
            if (sourcePath) {
                const response = await fetch(`${baseUrl}/api/knowledge/entries/delete`, {
                    method: "POST",
                    credentials: "include",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        source_path: sourcePath,
                    }),
                });
                if (!response.ok) {
                    const text = await response.text();
                    throw new Error(text || `HTTP ${response.status}`);
                }
            }
            removeNoteFromState(note.id);
            setCaptureMessage(
                note.relativePath?.trim()
                    ? `已删除 ${note.relativePath}`
                    : `已删除 ${note.title}`,
            );
        } catch (error) {
            setCaptureError(
                error instanceof Error
                    ? `删除笔记失败：${error.message}`
                    : "删除笔记失败。",
            );
        } finally {
            setDeletingNoteId((current) => (current === note.id ? null : current));
        }
    }

    async function handleSaveNote(note: NoteCardData) {
        if (savingNoteId || deletingNoteId) {
            return;
        }
        const draft = editDrafts[note.id];
        const body = draft?.body?.trim();
        if (!body) {
            setCaptureMessage(null);
            setCaptureError("笔记内容不能为空。");
            return;
        }
        const title = draft?.title?.trim() || deriveNoteTitle(body);
        setSavingNoteId(note.id);
        setCaptureMessage(null);
        setCaptureError(null);
        try {
            const saved = await persistKnowledgeEntry({
                source_path: note.sourcePath ?? "",
                relative_path: note.relativePath ?? "",
                title,
                body,
                tags: note.tags,
            });
            const mapped = mergePersistedNote(saved, note.id);
            setSelectedNoteId(mapped.id);
            stopEditingNote(note.id);
            setCaptureMessage(
                saved.relative_path?.trim()
                    ? `已保存 ${saved.relative_path}`
                    : `已保存 ${saved.title}`,
            );
        } catch (error) {
            setCaptureError(
                error instanceof Error
                    ? `保存笔记失败：${error.message}`
                    : "保存笔记失败。",
            );
        } finally {
            setSavingNoteId((current) => (current === note.id ? null : current));
        }
    }

    function toggleSharedTag(tag: string) {
        setSeedTags((current) =>
            current.includes(tag)
                ? current.filter((item) => item !== tag)
                : [...current, tag],
        );
        setActiveTag((current) => (current === tag ? "" : tag));
    }

    async function handleCapture() {
        const body = captureDraft.trim();
        if (!body) {
            return;
        }
        const tags = deriveTagsFromCapture(body, seedTags);
        const title = deriveNoteTitle(body);
        const resetComposer = () => {
            setCaptureDraft("");
            setSeedTags([]);
        };
        const pushCapturedNote = (note: NoteCardData) => {
            setSelectedNoteId(note.id);
            setActiveTag((current) => current || note.tags[0] || "");
            setActiveView("inbox");
        };
        setCaptureSaving(true);
        setCaptureMessage(null);
        setCaptureError(null);
        try {
            const saved = await persistKnowledgeEntry({
                source_path: "",
                relative_path: "",
                title,
                body,
                tags,
            });
            const note = mergePersistedNote(saved);
            pushCapturedNote(note);
            resetComposer();
            setCaptureMessage(
                tags.length > 0
                    ? `已记录到 ${saved.relative_path}`
                    : `已记录到 ${saved.relative_path}，AI 正在补标签`,
            );
            if (tags.length === 0) {
                void autoTagPersistedNote(saved);
            }
        } catch (error) {
            setCaptureError(
                error instanceof Error
                    ? `记录笔记失败：${error.message}`
                    : "记录笔记失败。",
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
                <QuickCapture
                    value={captureDraft}
                    tagNodes={tagNodes}
                    seedTags={seedTags}
                    activeTag={resolvedActiveTag}
                    saving={captureSaving}
                    message={captureMessage}
                    error={captureError}
                    onChange={setCaptureDraft}
                    onToggleTag={toggleSharedTag}
                    onSubmit={handleCapture}
                />

                <div className="p-4 lg:p-5">
                    {notesLoadError ? (
                        <div className="mb-4 rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
                            {notesLoadError}
                        </div>
                    ) : null}
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
                                {notesLoading ? (
                                    <Card className="mb-4 rounded-3xl border-stone-300 bg-white shadow-sm">
                                        <CardContent className="flex min-h-40 items-center justify-center p-6 text-sm text-muted-foreground">
                                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                            正在加载笔记…
                                        </CardContent>
                                    </Card>
                                ) : null}
                                {filteredNotes.map((note) => (
                                    <NoteCard
                                        key={note.id}
                                        note={note}
                                        autoTagging={autoTaggingNoteIds.includes(note.id)}
                                        editing={editingNoteId === note.id}
                                        draftTitle={editDrafts[note.id]?.title ?? note.title}
                                        draftBody={editDrafts[note.id]?.body ?? note.body}
                                        savePending={savingNoteId === note.id}
                                        selected={note.id === selectedNote?.id}
                                        deletePending={deletingNoteId === note.id}
                                        onSelect={() => {
                                            setSelectedNoteId(note.id);
                                            setActiveTag(note.tags[0] ?? "");
                                            beginEditingNote(note);
                                        }}
                                        onDraftTitleChange={(value) => updateEditDraft(note.id, { title: value })}
                                        onDraftBodyChange={(value) => updateEditDraft(note.id, { body: value })}
                                        onCancelEdit={() => stopEditingNote(note.id)}
                                        onSaveEdit={() => handleSaveNote(note)}
                                        onDelete={() => handleDeleteNote(note)}
                                    />
                                ))}
                                {!notesLoading && filteredNotes.length === 0 ? (
                                    <Card className="rounded-3xl border-dashed border-stone-300 bg-[#fffdf8] shadow-none">
                                        <CardContent className="flex min-h-40 items-center justify-center p-6 text-sm text-muted-foreground">
                                            还没有笔记。直接在上面记录，标签会从已入库笔记里自动汇聚。
                                        </CardContent>
                                    </Card>
                                ) : null}
                            </section>
                        </TabsContent>

                        <TabsContent value="tags" className="mt-0">
                            <div className="grid gap-4 xl:grid-cols-[20rem_minmax(0,1fr)]">
                                <Card className="rounded-3xl border-stone-300 bg-[#fffdf8] shadow-sm">
                                    <CardHeader className="p-4 pb-3">
                                        <CardTitle className="flex items-center gap-2 text-sm">
                                            <Tags className="h-4 w-4 text-stone-700" />
                                            标签池
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-3 p-4 pt-0">
                                        <div className="rounded-2xl border border-dashed border-stone-300 bg-white/90 p-3 text-sm text-muted-foreground">
                                            顶部这一排共享标签就是唯一入口。录入时点它会顺手打标，下面几个视图也共用同一个筛选状态。
                                        </div>
                                        <SpecLine label="标签总数" value={`${tagNodes.length} 个`} />
                                        <SpecLine label="当前筛选" value={activeTagSpecValue} />
                                        <div className="flex flex-wrap gap-2">
                                            {tagNodes.map((tag) => (
                                                <Badge
                                                    key={tag.label}
                                                    variant="outline"
                                                    className={cn(
                                                        "rounded-full bg-white/80",
                                                        resolvedActiveTag === tag.label &&
                                                            "border-stone-900 text-stone-900",
                                                    )}
                                                >
                                                    #{tag.label} · {tag.noteIds.length}
                                                </Badge>
                                            ))}
                                        </div>
                                    </CardContent>
                                </Card>

                                <div className="space-y-4">
                                    <Card className="rounded-3xl border-stone-300 bg-white shadow-sm">
                                        <CardHeader className="p-4 pb-3">
                                            <CardTitle className="flex items-center gap-2 text-sm">
                                                <Hash className="h-4 w-4 text-amber-600" />
                                                {activeTagHeading}
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-4 p-4 pt-0">
                                            {resolvedActiveTag ? (
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
                                            ) : (
                                                <div className="rounded-2xl border border-dashed border-stone-300 bg-[#fbfaf5] p-4 text-sm text-muted-foreground">
                                                    先点顶部共享标签条里的任意标签，这里就会切到对应标签簇；不点时先展示全部碎片。
                                                </div>
                                            )}
                                            <div className="grid gap-3 lg:grid-cols-2">
                                                {tagFocusedNotes.map((note) => (
                                                    <NoteCard
                                                        key={note.id}
                                                        note={note}
                                                        autoTagging={autoTaggingNoteIds.includes(note.id)}
                                                        compact
                                                        editing={editingNoteId === note.id}
                                                        draftTitle={editDrafts[note.id]?.title ?? note.title}
                                                        draftBody={editDrafts[note.id]?.body ?? note.body}
                                                        savePending={savingNoteId === note.id}
                                                        selected={note.id === selectedNote?.id}
                                                        deletePending={deletingNoteId === note.id}
                                                        onSelect={() => {
                                                            setSelectedNoteId(note.id);
                                                            beginEditingNote(note);
                                                        }}
                                                        onDraftTitleChange={(value) => updateEditDraft(note.id, { title: value })}
                                                        onDraftBodyChange={(value) => updateEditDraft(note.id, { body: value })}
                                                        onCancelEdit={() => stopEditingNote(note.id)}
                                                        onSaveEdit={() => handleSaveNote(note)}
                                                        onDelete={() => handleDeleteNote(note)}
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
                                        <SpecLine label="起点标签" value={activeTagSpecValue} />
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
                                            标签池快照
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-3 p-4 pt-0">
                                        <div className="rounded-2xl border border-dashed border-stone-300 bg-white/90 p-3 text-sm text-muted-foreground">
                                            不再在这里放第二套标签按钮，整理页直接吃顶部共享标签条的当前选择。
                                        </div>
                                        <SpecLine label="当前标签" value={activeTagSpecValue} />
                                        <SpecLine label="标签节点" value={`${tagNodes.length} 个`} />
                                        <div className="flex flex-wrap gap-2">
                                            {tagNodes.map((tag) => (
                                                <Badge
                                                    key={`${tag.label}-organize`}
                                                    variant="outline"
                                                    className={cn(
                                                        "rounded-full bg-white/80",
                                                        resolvedActiveTag === tag.label &&
                                                            "border-stone-900 text-stone-900",
                                                    )}
                                                >
                                                    #{tag.label}
                                                </Badge>
                                            ))}
                                        </div>
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
                                                    {resolvedActiveTag ? (
                                                        <>
                                                            以 <strong>{activeTagHeading}</strong> 为起点，把可达标签簇内的碎片一起归并成一个主题视图。
                                                            这里不先做字段建模，只做去重、摘要、标签收敛和知识线串联。
                                                        </>
                                                    ) : (
                                                        <>
                                                            先从顶部共享标签条点一个标签，再把对应标签簇里的碎片做去重、摘要和主题归并。
                                                        </>
                                                    )}
                                                </p>
                                            </div>
                                            {connectedTags.length > 0 ? (
                                                <div className="flex flex-wrap gap-2">
                                                    {connectedTags.map((tag) => (
                                                        <Badge key={tag} variant="secondary" className="rounded-full">
                                                            #{tag}
                                                        </Badge>
                                                    ))}
                                                </div>
                                            ) : null}
                                            <div className="grid gap-3">
                                                {tagFocusedNotes.map((note) => (
                                                    <NoteCard
                                                        key={`${note.id}-organize`}
                                                        note={note}
                                                        autoTagging={autoTaggingNoteIds.includes(note.id)}
                                                        compact
                                                        editing={editingNoteId === note.id}
                                                        draftTitle={editDrafts[note.id]?.title ?? note.title}
                                                        draftBody={editDrafts[note.id]?.body ?? note.body}
                                                        savePending={savingNoteId === note.id}
                                                        selected={note.id === selectedNote?.id}
                                                        deletePending={deletingNoteId === note.id}
                                                        onSelect={() => {
                                                            setSelectedNoteId(note.id);
                                                            beginEditingNote(note);
                                                        }}
                                                        onDraftTitleChange={(value) => updateEditDraft(note.id, { title: value })}
                                                        onDraftBodyChange={(value) => updateEditDraft(note.id, { body: value })}
                                                        onCancelEdit={() => stopEditingNote(note.id)}
                                                        onSaveEdit={() => handleSaveNote(note)}
                                                        onDelete={() => handleDeleteNote(note)}
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
    tagNodes,
    seedTags,
    activeTag,
    saving,
    message,
    error,
    onChange,
    onToggleTag,
    onSubmit,
}: {
    value: string;
    tagNodes: TagNode[];
    seedTags: string[];
    activeTag: string;
    saving: boolean;
    message: string | null;
    error: string | null;
    onChange: (value: string) => void;
    onToggleTag: (tag: string) => void;
    onSubmit: () => Promise<void>;
}) {
    return (
        <section className="border-b bg-[#f6f5ef] p-3 lg:p-5">
            <Card className="overflow-hidden rounded-3xl border-stone-300 bg-white shadow-[0_16px_44px_rgba(42,37,29,0.10)]">
                <div className="border-b bg-[#fffdf7] px-3 py-2 text-sm text-muted-foreground lg:px-4 lg:py-3">
                    <div className="flex flex-wrap items-center gap-2">
                        <Smile className="h-4 w-4" />
                        <span className="font-medium text-foreground">Markdown Rich Text</span>
                        <div className="ml-auto flex items-center gap-2 text-xs">
                            <Badge variant="outline" className="rounded-full">
                                Markdown
                            </Badge>
                            <Badge variant="outline" className="rounded-full">
                                富文本
                            </Badge>
                        </div>
                    </div>
                </div>
                <RichMarkdownEditor
                    value={value}
                    onChange={onChange}
                    placeholder="记录碎片、命令、结论或上下文。支持标题、列表、引用、链接。"
                    minHeight={144}
                />
                <div className="flex items-center justify-between gap-3 border-t bg-[#fffdf7] px-3 py-2 lg:px-4 lg:py-3">
                    <div className="min-w-0 flex-1 overflow-x-auto">
                        <div className="flex min-w-max items-center gap-2 pr-2 text-muted-foreground">
                            {tagNodes.map((tag) => {
                                const selected = seedTags.includes(tag.label);
                                const filtering = activeTag === tag.label;
                                return (
                                    <Button
                                        key={tag.label}
                                        type="button"
                                        variant={selected ? "default" : "ghost"}
                                        size="sm"
                                        className={cn(
                                            "shrink-0 rounded-full border",
                                            filtering
                                                ? "border-stone-900"
                                                : "border-transparent",
                                        )}
                                        onClick={() => onToggleTag(tag.label)}
                                    >
                                        {tag.label === "闪念" ? (
                                            <Zap className="h-4 w-4 fill-amber-400 text-amber-400" />
                                        ) : (
                                            <Hash className="h-4 w-4" />
                                        )}
                                        {tag.label}
                                    </Button>
                                );
                            })}
                            {tagNodes.length === 0 ? (
                                <span className="rounded-full border border-dashed border-stone-300 bg-white/80 px-3 py-1 text-xs text-muted-foreground">
                                    暂无历史标签，直接记录后会由 AI 补标签
                                </span>
                            ) : null}
                            {seedTags.length === 0 ? (
                                <span className="rounded-full border border-dashed border-stone-300 bg-white/80 px-3 py-1 text-xs text-muted-foreground">
                                    未手选标签时，记录后会异步 AI 打标
                                </span>
                            ) : null}
                        </div>
                    </div>
                    <Button
                        type="button"
                        className="shrink-0 rounded-2xl px-5"
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

function RichMarkdownEditor({
    value,
    onChange,
    placeholder,
    minHeight,
    compact = false,
}: {
    value: string;
    onChange: (value: string) => void;
    placeholder: string;
    minHeight: number;
    compact?: boolean;
}) {
    const editorRef = useRef<MDXEditorMethods>(null);
    const lastMarkdownRef = useRef(value);

    useEffect(() => {
        if (value !== lastMarkdownRef.current) {
            editorRef.current?.setMarkdown(value);
            lastMarkdownRef.current = value;
        }
    }, [value]);

    return (
        <div
            className={cn(
                "rich-note-editor bg-white",
                compact ? "rounded-[18px]" : "rounded-none",
            )}
            style={{ minHeight }}
        >
            <MDXEditor
                ref={editorRef}
                markdown={value}
                onChange={(markdown) => {
                    lastMarkdownRef.current = markdown;
                    onChange(markdown);
                }}
                placeholder={
                    <div className="px-4 py-3 text-sm text-stone-400">{placeholder}</div>
                }
                className={cn(
                    "text-stone-900",
                    compact ? "[&_.mdxeditor-toolbar]:rounded-t-[18px]" : "",
                )}
                contentEditableClassName={cn(
                    "prose prose-stone max-w-none px-4 py-3 text-[15px] leading-7 outline-none",
                    "min-h-[140px]",
                    compact && "min-h-[180px] px-3 py-3 text-[15px]",
                )}
                plugins={[...NOTE_EDITOR_PLUGINS]}
            />
        </div>
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

function mapPersistedNote(note: KnowledgeNoteDto): NoteCardData {
    const tags = dedupeTags(note.tags);
    const mainTag = tags[0] ?? "未整理";
    const sourcePath = note.source_path?.trim() || note.slug;
    const status = tags.length > 0 ? "reviewing" : "captured";

    return {
        id: sourcePath,
        time: formatRemoteTimestamp(note.updated_at),
        title: note.title,
        body: note.body,
        sourcePath: note.source_path,
        relativePath: note.relative_path,
        tags,
        kind: detectNoteKind(note.body),
        source: "笔记工作台",
        accent: accentForTag(mainTag),
        status,
        cluster: tags.length > 0 ? `${mainTag} 标签簇` : "待 AI 打标签",
        structuredKind: "knowledge.note",
        promptContexts: tags.map((tag) => `tag:${tag}`),
        unmapped:
            tags.length > 0
                ? []
                : ["当前未显式打标，等待 AI 归类或人工补标签。"],
        relations: [
            {
                target: note.relative_path || `note/${slugifyTitle(note.title)}`,
                relation: "数据库记录",
                confidence: 100,
            },
            ...(tags[0]
                ? [{ target: `#${tags[0]}`, relation: "标签入口", confidence: 96 }]
                : []),
        ],
        draft: {
            title: note.title,
            summary: note.excerpt?.trim() || summarizeBody(note.body),
            fields: [
                { label: "标准类型", value: "knowledge.note" },
                { label: "标签数", value: String(tags.length) },
                {
                    label: "当前状态",
                    value: tags.length > 0 ? "已打标签" : "待 AI 打标签",
                    tone: tags.length > 0 ? "success" : "warning",
                },
                {
                    label: "数据库路径",
                    value: note.relative_path || note.filename,
                },
            ],
        },
    };
}

function parseAiTagList(content: string): string[] {
    const normalized = content
        .replace(/```json/giu, "")
        .replace(/```/gu, "")
        .trim();
    const candidates = [normalized];

    const objectMatch = normalized.match(/\{[\s\S]*\}/u);
    if (objectMatch) {
        candidates.push(objectMatch[0]);
    }

    const arrayMatch = normalized.match(/\[[\s\S]*\]/u);
    if (arrayMatch) {
        candidates.push(`{"tags":${arrayMatch[0]}}`);
    }

    for (const candidate of candidates) {
        try {
            const parsed = JSON.parse(candidate) as
                | { tags?: unknown }
                | string[]
                | null;
            if (Array.isArray(parsed)) {
                return dedupeTags(parsed.filter((item): item is string => typeof item === "string"));
            }
            if (parsed && Array.isArray(parsed.tags)) {
                return dedupeTags(
                    parsed.tags.filter((item): item is string => typeof item === "string"),
                );
            }
        } catch {
            // fall through to the next candidate
        }
    }

    return dedupeTags(
        normalized
            .split(/[\n,，]/u)
            .map((item) => item.replace(/^[-*#\d.\s]+/u, "").trim())
            .filter(Boolean),
    );
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
    editing = false,
    draftTitle,
    draftBody,
    savePending = false,
    autoTagging = false,
    selected = false,
    compact = false,
    onSelect,
    onDraftTitleChange,
    onDraftBodyChange,
    onCancelEdit,
    onSaveEdit,
    onDelete,
    deletePending = false,
}: {
    note: NoteCardData;
    editing?: boolean;
    draftTitle?: string;
    draftBody?: string;
    savePending?: boolean;
    autoTagging?: boolean;
    selected?: boolean;
    compact?: boolean;
    onSelect?: () => void;
    onDraftTitleChange?: (value: string) => void;
    onDraftBodyChange?: (value: string) => void;
    onCancelEdit?: () => void;
    onSaveEdit?: () => Promise<void> | void;
    onDelete?: () => Promise<void> | void;
    deletePending?: boolean;
}) {
    return (
        <article
            className={cn(
                "mb-4 break-inside-avoid overflow-hidden rounded-3xl border bg-white shadow-[0_12px_28px_rgba(42,37,29,0.07)]",
                editing
                    ? "border-amber-500 ring-2 ring-amber-200/80 shadow-[0_18px_36px_rgba(180,116,0,0.12)]"
                    : selected
                      ? "border-stone-900 ring-1 ring-stone-900/10"
                      : "border-stone-200",
                !editing && onSelect && "cursor-pointer",
            )}
            onClick={editing ? undefined : onSelect}
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
                            {autoTagging ? (
                                <Badge variant="outline" className="rounded-full border-amber-300 bg-amber-50 text-amber-800">
                                    AI 打标中
                                </Badge>
                            ) : null}
                            <span className="text-xs text-muted-foreground">
                                {note.source}
                            </span>
                        </div>
                    </div>
                    {editing ? (
                        <InlineEditActionBar
                            pending={savePending}
                            onCancel={onCancelEdit}
                            onSave={onSaveEdit}
                        />
                    ) : (
                        <InlineDeleteAction
                            pending={deletePending}
                            onConfirm={onDelete}
                        />
                    )}
                </div>
                {editing ? (
                    <div className="mt-4 space-y-3">
                        <div className="rounded-2xl border border-amber-200 bg-[#fffaf0] p-2">
                            <Input
                                value={draftTitle ?? ""}
                                onClick={(event) => event.stopPropagation()}
                                onChange={(event) => onDraftTitleChange?.(event.target.value)}
                                className="border-0 bg-transparent px-2 text-base font-semibold text-stone-950 shadow-none focus-visible:ring-0"
                                placeholder="输入标题，不填则按正文首行生成"
                            />
                        </div>
                        <div className="rounded-2xl border border-stone-200 bg-[#fffdf8] p-2">
                            <div
                                onClick={(event) => event.stopPropagation()}
                                onKeyDown={(event) => {
                                    if ((event.metaKey || event.ctrlKey) && event.key === "Enter" && !savePending) {
                                        event.preventDefault();
                                        void onSaveEdit?.();
                                    }
                                    if (event.key === "Escape" && !savePending) {
                                        event.preventDefault();
                                        onCancelEdit?.();
                                    }
                                }}
                            >
                                <RichMarkdownEditor
                                    value={draftBody ?? ""}
                                    onChange={(value) => onDraftBodyChange?.(value)}
                                    placeholder="直接修改正文内容"
                                    minHeight={180}
                                    compact
                                />
                            </div>
                        </div>
                        <div className="flex flex-wrap items-center justify-between gap-2 rounded-2xl border border-dashed border-amber-200 bg-[#fff9ec] px-3 py-2 text-[11px] text-amber-800">
                            <span>点击卡片即进入编辑，`Cmd/Ctrl + Enter` 保存，`Esc` 取消。</span>
                            <Badge variant="outline" className="rounded-full border-amber-200 bg-white/80 text-amber-700">
                                编辑中
                            </Badge>
                        </div>
                    </div>
                ) : (
                    <div className="mt-4">
                        <h3 className={cn("text-base font-semibold text-stone-950", compact && "text-sm")}>
                            {note.title}
                        </h3>
                        <p className={cn("mt-2 text-[15px] leading-7 text-stone-900", compact && "line-clamp-4 text-sm leading-6")}>
                            {note.body}
                        </p>
                    </div>
                )}
                <div className="mt-4 flex flex-wrap gap-2">
                    {note.tags.length > 0 ? (
                        note.tags.map((tag) => (
                            <Badge key={tag} variant="secondary" className="rounded-full">
                                #{tag}
                            </Badge>
                        ))
                    ) : (
                        <Badge variant="outline" className="rounded-full border-dashed">
                            待打标签
                        </Badge>
                    )}
                </div>
                <div className="mt-4 flex items-center justify-between gap-3 rounded-2xl border bg-[#fbfaf5] px-3 py-2 text-xs text-muted-foreground">
                    <span>{note.cluster}</span>
                    <span>{note.tags.length} tags</span>
                </div>
            </div>
        </article>
    );
}

function InlineEditActionBar({
    pending = false,
    onCancel,
    onSave,
}: {
    pending?: boolean;
    onCancel?: () => void;
    onSave?: () => Promise<void> | void;
}) {
    return (
        <div
            className="flex shrink-0 items-center gap-2"
            onClick={(event) => event.stopPropagation()}
        >
            <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-8 rounded-full border-stone-300 bg-white/90 px-3 text-xs"
                onClick={onCancel}
                disabled={pending}
            >
                取消
            </Button>
            <Button
                type="button"
                size="sm"
                className="h-8 rounded-full bg-stone-900 px-3 text-xs text-white hover:bg-stone-800"
                onClick={() => void onSave?.()}
                disabled={pending}
            >
                {pending ? (
                    <>
                        <Loader2 className="h-3.5 w-3.5 animate-spin" />
                        保存中
                    </>
                ) : (
                    "保存"
                )}
            </Button>
        </div>
    );
}

function InlineDeleteAction({
    pending = false,
    onConfirm,
}: {
    pending?: boolean;
    onConfirm?: () => Promise<void> | void;
}) {
    const [confirming, setConfirming] = useState(false);

    function stopCardSelect(event: MouseEvent<HTMLElement>) {
        event.stopPropagation();
    }

    async function handleConfirm(event: MouseEvent<HTMLButtonElement>) {
        event.stopPropagation();
        if (!onConfirm || pending) {
            return;
        }
        try {
            await onConfirm();
        } finally {
            setConfirming(false);
        }
    }

    return (
        <div
            className="flex shrink-0 items-center justify-end"
            onClick={stopCardSelect}
        >
            {pending ? (
                <span className="inline-flex items-center gap-1.5 rounded-full border border-rose-200 bg-rose-50 px-3 py-1 text-[11px] font-semibold text-rose-700">
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    删除中
                </span>
            ) : confirming ? (
                <div className="inline-flex items-center gap-1 rounded-full border border-rose-200 bg-rose-50/90 p-1 pl-3 shadow-sm">
                    <span className="text-[11px] font-semibold text-rose-700">
                        确认删除
                    </span>
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        aria-label="确认删除"
                        className="h-7 w-7 rounded-full text-emerald-700 hover:bg-emerald-100 hover:text-emerald-800"
                        onClick={handleConfirm}
                    >
                        <Check className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        aria-label="取消删除"
                        className="h-7 w-7 rounded-full text-stone-500 hover:bg-stone-200 hover:text-stone-700"
                        onClick={(event) => {
                            event.stopPropagation();
                            setConfirming(false);
                        }}
                    >
                        <X className="h-3.5 w-3.5" />
                    </Button>
                </div>
            ) : (
                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-8 rounded-full px-3 text-xs font-semibold text-stone-500 hover:bg-rose-50 hover:text-rose-700"
                    onClick={(event) => {
                        event.stopPropagation();
                        setConfirming(true);
                    }}
                >
                    <Trash2 className="h-3.5 w-3.5" />
                    删除
                </Button>
            )}
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

function dedupeTags(tags: string[]): string[] {
    return Array.from(
        new Set(
            tags
                .map((tag) => tag.trim().replace(/^#+/u, "").trim())
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

function formatRemoteTimestamp(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return value.replace("T", " ").replace(/Z$/u, "");
    }
    return formatLocalTimestamp(date);
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
