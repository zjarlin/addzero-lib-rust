import { useEffect, useMemo, useRef, useState, type MouseEvent } from "react";
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
    Check,
    FileText,
    Loader2,
    Search,
    Send,
    Smile,
    Trash2,
    X,
    Zap,
} from "lucide-react";
import {
    Badge,
    Button,
    Card,
    CardContent,
    Input,
    cn,
} from "@az/ui";
import { type KnowledgeNoteDto } from "@az/api-client";

export interface NotesFragmentsPluginSchema {
    list_path: string;
    save_path: string;
    delete_path: string;
    placeholder: string;
    empty_message: string;
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

export function NotesFragmentsPluginView({
    schema,
    baseUrl,
}: {
    schema: NotesFragmentsPluginSchema;
    baseUrl: string;
}) {
    const [search, setSearch] = useState("");
    const [notes, setNotes] = useState<NoteCardData[]>([]);
    const [notesLoading, setNotesLoading] = useState(true);
    const [notesLoadError, setNotesLoadError] = useState<string | null>(null);
    const [selectedNoteId, setSelectedNoteId] = useState("");
    const [editingNoteId, setEditingNoteId] = useState<string | null>(null);
    const [editDrafts, setEditDrafts] = useState<Record<string, { title: string; body: string }>>({});
    const [savingNoteId, setSavingNoteId] = useState<string | null>(null);
    const [deletingNoteId, setDeletingNoteId] = useState<string | null>(null);
    const [captureDraft, setCaptureDraft] = useState("");
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

    useEffect(() => {
        let cancelled = false;

        async function loadNotes() {
            setNotesLoading(true);
            setNotesLoadError(null);
            try {
                const response = await fetch(`${baseUrl}${schema.list_path}`, {
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
    }, [baseUrl, schema.list_path]);

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
        const response = await fetch(`${baseUrl}${schema.save_path}`, {
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
                const response = await fetch(`${baseUrl}${schema.delete_path}`, {
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

    async function handleCapture() {
        const body = captureDraft.trim();
        if (!body) {
            return;
        }
        const tags = deriveTagsFromCapture(body);
        const title = deriveNoteTitle(body);
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
            setSelectedNoteId(note.id);
            setCaptureDraft("");
            const savedLocation = saved.relative_path?.trim() || saved.title;
            setCaptureMessage(`已记录到 ${savedLocation}`);
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
                                WASM Plugin
                                <Badge variant="outline" className="rounded-full bg-white/70">
                                    Notes
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
                    saving={captureSaving}
                    message={captureMessage}
                    error={captureError}
                    placeholder={schema.placeholder}
                    onChange={setCaptureDraft}
                    onSubmit={handleCapture}
                />

                <div className="p-4 lg:p-5">
                    {notesLoadError ? (
                        <div className="mb-4 rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
                            {notesLoadError}
                        </div>
                    ) : null}
                    <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
                        <div className="flex items-center gap-2 text-sm font-semibold">
                            <FileText className="h-4 w-4 text-muted-foreground" />
                            碎片流
                        </div>
                        <Badge variant="outline" className="rounded-full bg-white/80">
                            {filteredNotes.length} 条
                        </Badge>
                    </div>
                    <section className="columns-1 gap-4 lg:columns-2 2xl:columns-3">
                        {notesLoading ? (
                            <Card className="mb-4 rounded-3xl border-stone-300 bg-white shadow-sm">
                                <CardContent className="flex min-h-40 items-center justify-center p-6 text-sm text-muted-foreground">
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                    正在加载笔记...
                                </CardContent>
                            </Card>
                        ) : null}
                        {filteredNotes.map((note) => (
                            <NoteCard
                                key={note.id}
                                note={note}
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
                        {!notesLoading && filteredNotes.length === 0 ? (
                            <Card className="rounded-3xl border-dashed border-stone-300 bg-[#fffdf8] shadow-none">
                                <CardContent className="flex min-h-40 items-center justify-center p-6 text-sm text-muted-foreground">
                                    {schema.empty_message}
                                </CardContent>
                            </Card>
                        ) : null}
                    </section>
                </div>
            </section>
        </div>
    );
}

function QuickCapture({
    value,
    saving,
    message,
    error,
    placeholder,
    onChange,
    onSubmit,
}: {
    value: string;
    saving: boolean;
    message: string | null;
    error: string | null;
    placeholder: string;
    onChange: (value: string) => void;
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
                    placeholder={placeholder}
                    minHeight={144}
                />
                <div className="flex items-center justify-between gap-3 border-t bg-[#fffdf7] px-3 py-2 lg:px-4 lg:py-3">
                    <div className="min-w-0 flex-1">
                        <span className="rounded-full border border-dashed border-stone-300 bg-white/80 px-3 py-1 text-xs text-muted-foreground">
                            正文里的 #标签 会随碎片一起保存
                        </span>
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
                onChange={(markdown: string) => {
                    lastMarkdownRef.current = markdown;
                    onChange(markdown);
                }}
                placeholder={placeholder}
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
        source: "笔记插件",
        accent: accentForTag(mainTag),
        status,
        cluster: tags.length > 0 ? `${mainTag} 碎片` : "未打标签碎片",
    };
}

function deriveTagsFromCapture(body: string) {
    const inlineTags = Array.from(
        body.matchAll(/#([\p{L}\p{N}_-]+)/gu),
        (match) => match[1],
    );
    return Array.from(
        new Set(
            inlineTags
                .map((tag) => tag.trim())
                .filter(Boolean),
        ),
    );
}

function NoteCard({
    note,
    editing = false,
    draftTitle,
    draftBody,
    savePending = false,
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
                            无标签
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
