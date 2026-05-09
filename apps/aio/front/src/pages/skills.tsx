import { useCallback, useEffect, useMemo, useState } from "react";
import {
    Braces,
    CheckCircle2,
    FileCode2,
    Grid2X2,
    LayoutList,
    Loader2,
    RefreshCw,
    Save,
    Search,
    Trash2,
} from "lucide-react";
import { getApiBaseUrl } from "@az/api-client";
import {
    Badge,
    Button,
    Input,
    ScrollArea,
    Textarea,
    cn,
} from "@az/ui";

interface Skill {
    name: string;
    keywords: string[];
    description: string;
    body: string;
    content_hash: string;
    source: "Postgres" | "FileSystem" | "Both" | string;
    updated_at: string;
}

interface SyncReport {
    added_to_fs: string[];
    added_to_pg: string[];
    updated_in_fs: string[];
    updated_in_pg: string[];
    conflicts: string[];
    finished_at: string | null;
    pg_online: boolean;
    fs_root: string;
}

interface DraftSkill {
    name: string;
    keywords: string;
    description: string;
    body: string;
}

type CategoryId = "all" | "dev" | "design" | "ops" | "docs" | "ai";
type ViewMode = "grid" | "list";

const categoryLabels: Record<CategoryId, string> = {
    all: "全部",
    dev: "开发",
    design: "设计",
    ops: "运维",
    docs: "文档",
    ai: "智能体",
};

const categoryHints: Record<Exclude<CategoryId, "all">, string[]> = {
    dev: [
        "code",
        "coding",
        "debug",
        "develop",
        "implement",
        "kotlin",
        "rust",
        "server",
        "test",
        "compose",
        "gradle",
        "ksp",
        "ktor",
    ],
    design: [
        "design",
        "frontend",
        "ui",
        "visual",
        "shadcn",
        "accessibility",
        "seo",
        "compose",
        "form",
        "table",
        "tree",
    ],
    ops: [
        "deploy",
        "release",
        "dotfiles",
        "storage",
        "minio",
        "rustfs",
        "plugin",
        "market",
        "cli",
        "workflow",
    ],
    docs: [
        "doc",
        "docs",
        "document",
        "readme",
        "wiki",
        "present",
        "spreadsheet",
        "skill",
        "spec",
        "roadmap",
    ],
    ai: [
        "ai",
        "agent",
        "openai",
        "prompt",
        "eval",
        "llm",
        "vibe",
        "chat",
        "knowledge",
    ],
};

const sourceLabels: Record<string, string> = {
    FileSystem: "本地文件",
    Postgres: "数据库",
    Both: "双写",
};

export default function SkillsPage() {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const [skills, setSkills] = useState<Skill[]>([]);
    const [status, setStatus] = useState<SyncReport | null>(null);
    const [loading, setLoading] = useState(true);
    const [syncing, setSyncing] = useState(false);
    const [saving, setSaving] = useState(false);
    const [deleting, setDeleting] = useState(false);
    const [search, setSearch] = useState("");
    const [category, setCategory] = useState<CategoryId>("all");
    const [viewMode, setViewMode] = useState<ViewMode>("grid");
    const [selectedName, setSelectedName] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [draft, setDraft] = useState<DraftSkill>({
        name: "",
        keywords: "",
        description: "",
        body: "",
    });

    const load = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const [skillsRes, statusRes] = await Promise.all([
                fetch(`${baseUrl}/api/skills`, { credentials: "include" }),
                fetch(`${baseUrl}/api/skills/status`, { credentials: "include" }),
            ]);
            if (!skillsRes.ok) {
                throw new Error(`技能列表加载失败: HTTP ${skillsRes.status}`);
            }
            if (!statusRes.ok) {
                throw new Error(`同步状态加载失败: HTTP ${statusRes.status}`);
            }
            const nextSkills = (await skillsRes.json()) as Skill[];
            setSkills(nextSkills);
            setStatus(await statusRes.json());
            setSelectedName((current) => {
                if (current && nextSkills.some((skill) => skill.name === current)) {
                    return current;
                }
                return nextSkills[0]?.name ?? null;
            });
        } catch (err) {
            setError(err instanceof Error ? err.message : "技能数据加载失败");
        } finally {
            setLoading(false);
        }
    }, [baseUrl]);

    useEffect(() => {
        void load();
    }, [load]);

    const selected = useMemo(
        () => skills.find((skill) => skill.name === selectedName) ?? null,
        [selectedName, skills],
    );

    useEffect(() => {
        if (!selected) {
            setDraft({
                name: "",
                keywords: "",
                description: "",
                body: "",
            });
            return;
        }
        setDraft({
            name: selected.name,
            keywords: selected.keywords.join(", "),
            description: selected.description,
            body: selected.body,
        });
    }, [selected]);

    const classifiedSkills = useMemo(
        () =>
            skills.map((skill) => ({
                skill,
                category: classifySkill(skill),
                parameters: extractParameters(skill.body),
                headings: extractHeadings(skill.body),
            })),
        [skills],
    );

    const filtered = useMemo(() => {
        const keyword = search.trim().toLowerCase();
        return classifiedSkills.filter(({ skill, category: skillCategory }) => {
            const matchesCategory = category === "all" || skillCategory === category;
            const haystack = [
                skill.name,
                skill.description,
                skill.body,
                skill.source,
                ...skill.keywords,
            ]
                .join("\n")
                .toLowerCase();
            return matchesCategory && (!keyword || haystack.includes(keyword));
        });
    }, [classifiedSkills, category, search]);

    const metrics = useMemo(() => {
        const sources = new Set(skills.map((skill) => skill.source));
        const parameters = classifiedSkills.reduce(
            (sum, item) => sum + item.parameters.length,
            0,
        );
        const dbBacked = skills.filter((skill) => skill.source !== "FileSystem").length;
        return {
            total: skills.length,
            dbBacked,
            sources: sources.size,
            parameters,
        };
    }, [classifiedSkills, skills]);

    async function syncSkills() {
        setSyncing(true);
        setError(null);
        setMessage(null);
        try {
            const res = await fetch(`${baseUrl}/api/skills/sync`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({}),
            });
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            const report = (await res.json()) as SyncReport;
            setStatus(report);
            setMessage(buildSyncMessage(report));
            await load();
        } catch (err) {
            setError(err instanceof Error ? err.message : "技能同步失败");
        } finally {
            setSyncing(false);
        }
    }

    async function saveSkill() {
        if (!draft.name.trim()) {
            setError("技能名称不能为空");
            return;
        }
        setSaving(true);
        setError(null);
        setMessage(null);
        try {
            const payload = {
                name: draft.name.trim(),
                keywords: parseKeywordInput(draft.keywords),
                description: draft.description.trim(),
                body: draft.body,
            };
            const res = await fetch(`${baseUrl}/api/skills/upsert`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            const saved = (await res.json()) as Skill;
            setMessage(`已保存 ${saved.name}`);
            await load();
            setSelectedName(saved.name);
        } catch (err) {
            setError(err instanceof Error ? err.message : "保存技能失败");
        } finally {
            setSaving(false);
        }
    }

    async function deleteSkill() {
        if (!selected) return;
        setDeleting(true);
        setError(null);
        setMessage(null);
        try {
            const res = await fetch(
                `${baseUrl}/api/skills/${encodeURIComponent(selected.name)}`,
                {
                    method: "DELETE",
                    credentials: "include",
                },
            );
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            setMessage(`已删除 ${selected.name}`);
            setSelectedName(null);
            await load();
        } catch (err) {
            setError(err instanceof Error ? err.message : "删除技能失败");
        } finally {
            setDeleting(false);
        }
    }

    function createDraft() {
        const name = "new-skill";
        setSelectedName(null);
        setDraft({
            name,
            keywords: "",
            description: "",
            body: `# ${name}\n\n## 适用范围\n\n- \n\n## 工作流程\n\n1. \n`,
        });
        setMessage(null);
        setError(null);
    }

    return (
        <div className="space-y-5">
            <header className="flex flex-col gap-4 border-b pb-5 xl:flex-row xl:items-center xl:justify-between">
                <div className="min-w-0">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-md bg-foreground text-sm font-semibold text-background">
                            S
                        </div>
                        <div>
                            <h1 className="text-2xl font-semibold tracking-tight">
                                Skill Manager
                            </h1>
                            <p className="mt-1 text-sm text-muted-foreground">
                                {status?.fs_root ?? "正在读取技能仓库"}
                            </p>
                        </div>
                    </div>
                </div>
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
                    <div className="relative min-w-0 sm:w-[24rem]">
                        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            placeholder="搜索 Skill 名称、描述或正文..."
                            value={search}
                            onChange={(event) => setSearch(event.target.value)}
                            className="pl-10"
                        />
                    </div>
                    <Button
                        type="button"
                        variant="outline"
                        onClick={() => void syncSkills()}
                        disabled={syncing}
                    >
                        {syncing ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                            <RefreshCw className="h-4 w-4" />
                        )}
                        同步
                    </Button>
                    <Button type="button" onClick={createDraft}>
                        <FileCode2 className="h-4 w-4" />
                        新建
                    </Button>
                </div>
            </header>

            <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
                <MetricBlock label="全部 Skill" value={metrics.total} />
                <MetricBlock label="数据库托管" value={metrics.dbBacked} />
                <MetricBlock label="来源类型" value={metrics.sources} />
                <MetricBlock label="可识别参数" value={metrics.parameters} />
            </section>

            <section className="flex flex-col gap-3 border-y py-3 lg:flex-row lg:items-center lg:justify-between">
                <div className="flex flex-wrap gap-2">
                    {(Object.keys(categoryLabels) as CategoryId[]).map((id) => (
                        <Button
                            key={id}
                            type="button"
                            variant={category === id ? "default" : "outline"}
                            size="sm"
                            onClick={() => setCategory(id)}
                            className="h-8"
                        >
                            {categoryLabels[id]}
                        </Button>
                    ))}
                </div>
                <div className="flex items-center gap-2">
                    <Button
                        type="button"
                        variant={viewMode === "grid" ? "default" : "outline"}
                        size="icon"
                        aria-label="网格视图"
                        onClick={() => setViewMode("grid")}
                        className="h-8 w-8"
                    >
                        <Grid2X2 className="h-4 w-4" />
                    </Button>
                    <Button
                        type="button"
                        variant={viewMode === "list" ? "default" : "outline"}
                        size="icon"
                        aria-label="列表视图"
                        onClick={() => setViewMode("list")}
                        className="h-8 w-8"
                    >
                        <LayoutList className="h-4 w-4" />
                    </Button>
                </div>
            </section>

            {error ? (
                <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                    {error}
                </div>
            ) : null}
            {message ? (
                <div className="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-700 dark:text-emerald-400">
                    {message}
                </div>
            ) : null}

            <section className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_25rem]">
                <div>
                    {loading ? (
                        <div className="flex min-h-[24rem] items-center justify-center text-muted-foreground">
                            <Loader2 className="h-5 w-5 animate-spin" />
                        </div>
                    ) : filtered.length === 0 ? (
                        <div className="flex min-h-[24rem] items-center justify-center rounded-md border border-dashed text-sm text-muted-foreground">
                            没有匹配的技能
                        </div>
                    ) : (
                        <div
                            className={cn(
                                viewMode === "grid"
                                    ? "grid gap-3 md:grid-cols-2 2xl:grid-cols-3"
                                    : "space-y-2",
                            )}
                        >
                            {filtered.map((item) => (
                                <SkillCard
                                    key={item.skill.name}
                                    item={item}
                                    active={item.skill.name === selected?.name}
                                    compact={viewMode === "list"}
                                    onSelect={() => setSelectedName(item.skill.name)}
                                />
                            ))}
                        </div>
                    )}
                </div>

                <SkillDetailPanel
                    selected={selected}
                    draft={draft}
                    setDraft={setDraft}
                    saving={saving}
                    deleting={deleting}
                    onSave={() => void saveSkill()}
                    onDelete={() => void deleteSkill()}
                />
            </section>
        </div>
    );
}

function MetricBlock({ label, value }: { label: string; value: number }) {
    return (
        <div className="rounded-md border px-4 py-3">
            <div className="text-2xl font-semibold tabular-nums">{value}</div>
            <div className="mt-1 text-xs text-muted-foreground">{label}</div>
        </div>
    );
}

function SkillCard({
    item,
    active,
    compact,
    onSelect,
}: {
    item: ReturnType<typeof normalizeSkillItem>;
    active: boolean;
    compact: boolean;
    onSelect: () => void;
}) {
    const { skill, category, parameters, headings } = item;
    const tags = skill.keywords.length > 0 ? skill.keywords : headings.slice(0, 3);

    return (
        <button
            type="button"
            onClick={onSelect}
            className={cn(
                "w-full rounded-md border bg-background p-4 text-left transition hover:border-foreground/40",
                active && "border-foreground shadow-sm",
                compact && "grid gap-3 md:grid-cols-[1.2fr_minmax(0,1.8fr)_auto]",
            )}
        >
            <div className="min-w-0">
                <div className="flex items-center gap-2">
                    <SkillIcon category={category} />
                    <div className="truncate text-sm font-semibold">{skill.name}</div>
                </div>
                <div className="mt-2 flex flex-wrap items-center gap-2">
                    <Badge variant="secondary">{sourceLabels[skill.source] ?? skill.source}</Badge>
                    <Badge variant="outline">{categoryLabels[category]}</Badge>
                </div>
            </div>
            <p
                className={cn(
                    "mt-3 line-clamp-3 text-sm leading-6 text-muted-foreground",
                    compact && "mt-0",
                )}
            >
                {skill.description || "暂无描述"}
            </p>
            <div
                className={cn(
                    "mt-4 flex flex-wrap items-center gap-2",
                    compact && "mt-0 justify-end",
                )}
            >
                {tags.slice(0, 4).map((tag) => (
                    <Badge key={`${skill.name}:${tag}`} variant="outline">
                        {tag}
                    </Badge>
                ))}
                <span className="inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs text-muted-foreground">
                    <Braces className="h-3.5 w-3.5" />
                    {parameters.length}
                </span>
            </div>
        </button>
    );
}

function SkillDetailPanel({
    selected,
    draft,
    setDraft,
    saving,
    deleting,
    onSave,
    onDelete,
}: {
    selected: Skill | null;
    draft: DraftSkill;
    setDraft: (updater: (current: DraftSkill) => DraftSkill) => void;
    saving: boolean;
    deleting: boolean;
    onSave: () => void;
    onDelete: () => void;
}) {
    const parameters = extractParameters(draft.body);
    const headings = extractHeadings(draft.body);

    return (
        <aside className="rounded-md border bg-muted/20">
            <div className="flex items-start justify-between gap-3 border-b p-4">
                <div className="min-w-0">
                    <div className="text-sm font-semibold">
                        {draft.name || "新建 Skill"}
                    </div>
                    <div className="mt-1 text-xs text-muted-foreground">
                        {selected
                            ? `更新于 ${formatDate(selected.updated_at)}`
                            : "保存后写入技能仓库"}
                    </div>
                </div>
                {selected ? (
                    <Badge variant="outline">{sourceLabels[selected.source] ?? selected.source}</Badge>
                ) : null}
            </div>

            <ScrollArea className="h-[calc(100vh-18rem)]">
                <div className="space-y-4 p-4">
                    <label className="block">
                        <span className="mb-2 block text-xs font-medium text-muted-foreground">
                            名称
                        </span>
                        <Input
                            value={draft.name}
                            onChange={(event) =>
                                setDraft((current) => ({
                                    ...current,
                                    name: event.target.value,
                                }))
                            }
                            placeholder="skill-name"
                        />
                    </label>

                    <label className="block">
                        <span className="mb-2 block text-xs font-medium text-muted-foreground">
                            描述
                        </span>
                        <Textarea
                            value={draft.description}
                            onChange={(event) =>
                                setDraft((current) => ({
                                    ...current,
                                    description: event.target.value,
                                }))
                            }
                            placeholder="这个 Skill 何时应该被使用"
                            className="min-h-24"
                        />
                    </label>

                    <label className="block">
                        <span className="mb-2 block text-xs font-medium text-muted-foreground">
                            标签
                        </span>
                        <Input
                            value={draft.keywords}
                            onChange={(event) =>
                                setDraft((current) => ({
                                    ...current,
                                    keywords: event.target.value,
                                }))
                            }
                            placeholder="frontend, design, test"
                        />
                    </label>

                    <div className="grid grid-cols-2 gap-2">
                        <MiniBlock label="参数" value={parameters.length} />
                        <MiniBlock label="章节" value={headings.length} />
                    </div>

                    {parameters.length > 0 ? (
                        <div className="rounded-md border bg-background">
                            <div className="border-b px-3 py-2 text-xs font-medium">
                                参数列表
                            </div>
                            <div className="divide-y">
                                {parameters.slice(0, 8).map((parameter) => (
                                    <div
                                        key={parameter}
                                        className="flex items-center justify-between gap-3 px-3 py-2 text-xs"
                                    >
                                        <span className="truncate font-mono">{parameter}</span>
                                        <span className="text-muted-foreground">可选</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ) : null}

                    <label className="block">
                        <span className="mb-2 block text-xs font-medium text-muted-foreground">
                            Skill 正文
                        </span>
                        <Textarea
                            value={draft.body}
                            onChange={(event) =>
                                setDraft((current) => ({
                                    ...current,
                                    body: event.target.value,
                                }))
                            }
                            className="min-h-[24rem] font-mono text-xs leading-5"
                        />
                    </label>
                </div>
            </ScrollArea>

            <div className="flex flex-wrap items-center justify-between gap-2 border-t p-4">
                <Button
                    type="button"
                    variant="destructive"
                    onClick={onDelete}
                    disabled={!selected || deleting}
                >
                    {deleting ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                        <Trash2 className="h-4 w-4" />
                    )}
                    删除
                </Button>
                <Button type="button" onClick={onSave} disabled={saving}>
                    {saving ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                        <Save className="h-4 w-4" />
                    )}
                    保存
                </Button>
            </div>
        </aside>
    );
}

function MiniBlock({ label, value }: { label: string; value: number }) {
    return (
        <div className="rounded-md border bg-background px-3 py-2">
            <div className="text-lg font-semibold tabular-nums">{value}</div>
            <div className="text-xs text-muted-foreground">{label}</div>
        </div>
    );
}

function SkillIcon({ category }: { category: CategoryId }) {
    if (category === "docs") return <FileCode2 className="h-4 w-4 text-sky-600" />;
    if (category === "ops") return <RefreshCw className="h-4 w-4 text-amber-600" />;
    if (category === "ai") return <Braces className="h-4 w-4 text-emerald-600" />;
    if (category === "design") return <Grid2X2 className="h-4 w-4 text-rose-600" />;
    return <CheckCircle2 className="h-4 w-4 text-muted-foreground" />;
}

function normalizeSkillItem(skill: Skill) {
    return {
        skill,
        category: classifySkill(skill),
        parameters: extractParameters(skill.body),
        headings: extractHeadings(skill.body),
    };
}

function classifySkill(skill: Skill): CategoryId {
    const text = [skill.name, skill.description, skill.body, ...skill.keywords]
        .join("\n")
        .toLowerCase();
    const scores = (Object.keys(categoryHints) as Exclude<CategoryId, "all">[])
        .map((categoryId) => ({
            category: categoryId,
            score: categoryHints[categoryId].filter((hint) => text.includes(hint)).length,
        }))
        .sort((left, right) => right.score - left.score);
    return scores[0]?.score ? scores[0].category : "dev";
}

function extractHeadings(body: string): string[] {
    return body
        .split("\n")
        .map((line) => line.match(/^#{1,3}\s+(.+)$/)?.[1]?.trim())
        .filter((value): value is string => Boolean(value))
        .slice(0, 12);
}

function extractParameters(body: string): string[] {
    const names = new Set<string>();
    for (const match of body.matchAll(/\{\{\s*([a-zA-Z_][\w.-]*)\s*\}\}/g)) {
        names.add(match[1]);
    }
    for (const match of body.matchAll(/\$([A-Z][A-Z0-9_]{2,})/g)) {
        names.add(match[1]);
    }
    for (const line of body.split("\n")) {
        const match = line.match(/^\s*[-*]\s+`?([a-zA-Z_][\w.-]*)`?\s*[:：]/);
        if (match) names.add(match[1]);
    }
    return [...names].slice(0, 24);
}

function parseKeywordInput(value: string): string[] {
    return value
        .split(/[,，\n]/)
        .map((item) => item.trim())
        .filter(Boolean);
}

function buildSyncMessage(report: SyncReport): string {
    const total =
        report.added_to_fs.length +
        report.added_to_pg.length +
        report.updated_in_fs.length +
        report.updated_in_pg.length;
    if (report.conflicts.length > 0) {
        return `同步完成，${report.conflicts.length} 个冲突需要处理`;
    }
    return `同步完成，${total} 个变更`;
}

function formatDate(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString();
}
