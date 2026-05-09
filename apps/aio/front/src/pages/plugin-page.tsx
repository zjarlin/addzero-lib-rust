import { useCallback, useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { getApiBaseUrl } from "@az/api-client";
import { Badge } from "@az/ui";
import {
    fetchInstanceWasmPluginPage,
    fetchSystemWasmPluginPage,
    type WasmPluginPageSchema,
    type WasmPluginResolvedPage,
} from "../lib/wasm-plugin-runtime";

export function SystemPluginPage() {
    const { pluginId = "", pageId = "" } = useParams();
    const load = useCallback(
        (baseUrl: string) => fetchSystemWasmPluginPage(pluginId, pageId, baseUrl),
        [pluginId, pageId],
    );
    return (
        <PluginPageScaffold load={load} />
    );
}

export function InstancePluginPage() {
    const { instanceSlug = "", pageId = "" } = useParams();
    const load = useCallback(
        (baseUrl: string) =>
            fetchInstanceWasmPluginPage(instanceSlug, pageId, baseUrl),
        [instanceSlug, pageId],
    );
    return (
        <PluginPageScaffold load={load} />
    );
}

function PluginPageScaffold({
    load,
}: {
    load: (baseUrl: string) => Promise<WasmPluginResolvedPage>;
}) {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const [page, setPage] = useState<WasmPluginResolvedPage | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;

        async function run() {
            setLoading(true);
            setError(null);
            try {
                const resolved = await load(baseUrl);
                if (!cancelled) {
                    setPage(resolved);
                }
            } catch (err) {
                if (!cancelled) {
                    setError(
                        err instanceof Error ? err.message : "加载插件页面失败",
                    );
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void run();
        return () => {
            cancelled = true;
        };
    }, [baseUrl, load]);

    if (loading) {
        return <div className="text-sm text-muted-foreground">加载插件页面中…</div>;
    }

    if (error) {
        return (
            <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-4 text-sm text-destructive">
                {error}
            </div>
        );
    }

    if (!page) {
        return (
            <div className="rounded-lg border px-4 py-4 text-sm text-muted-foreground">
                插件页面不存在。
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <div className="space-y-3 rounded-lg border bg-card px-5 py-5">
                <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="secondary">{page.scope}</Badge>
                    <Badge variant="outline">{page.plugin_name}</Badge>
                    <span className="font-mono text-xs text-muted-foreground">
                        {page.plugin_id} / {page.page_id}
                    </span>
                </div>
                <div>
                    <h1 className="text-2xl font-semibold tracking-tight">{page.title}</h1>
                    <p className="mt-2 text-sm text-muted-foreground">{page.subtitle}</p>
                </div>
                <div className="text-xs text-muted-foreground">
                    {page.breadcrumbs.join(" / ")}
                </div>
            </div>

            <PluginSchemaView schema={page.schema} />
        </div>
    );
}

function PluginSchemaView({ schema }: { schema: WasmPluginPageSchema }) {
    switch (schema.kind) {
        case "markdown":
            return (
                <article className="prose prose-sm max-w-none rounded-lg border bg-card px-5 py-5 dark:prose-invert">
                    <pre className="whitespace-pre-wrap font-sans text-sm leading-6">
                        {schema.body}
                    </pre>
                </article>
            );
        case "table":
            return (
                <div className="rounded-lg border bg-card">
                    <div className="overflow-x-auto">
                        <table className="min-w-full text-sm">
                            <thead className="border-b bg-muted/30">
                                <tr>
                                    {schema.columns.map((column) => (
                                        <th
                                            key={column}
                                            className="px-4 py-3 text-left font-medium"
                                        >
                                            {column}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {schema.rows.length > 0 ? (
                                    schema.rows.map((row, index) => (
                                        <tr key={`${index}-${row.cells.join("|")}`} className="border-b">
                                            {row.cells.map((cell, cellIndex) => (
                                                <td key={`${index}-${cellIndex}`} className="px-4 py-3">
                                                    {cell}
                                                </td>
                                            ))}
                                        </tr>
                                    ))
                                ) : (
                                    <tr>
                                        <td
                                            className="px-4 py-6 text-muted-foreground"
                                            colSpan={Math.max(schema.columns.length, 1)}
                                        >
                                            {schema.empty_message}
                                        </td>
                                    </tr>
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>
            );
        case "board":
            return (
                <div className="space-y-4">
                    <div className="grid gap-4 md:grid-cols-3">
                        {schema.metrics.map((metric) => (
                            <div key={metric.label} className="rounded-lg border bg-card px-4 py-4">
                                <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                                    {metric.label}
                                </div>
                                <div className="mt-2 text-2xl font-semibold">{metric.value}</div>
                                <div className="mt-2 text-sm text-muted-foreground">{metric.detail}</div>
                            </div>
                        ))}
                    </div>
                    {schema.groups.map((group) => (
                        <div key={group.title} className="rounded-lg border bg-card px-5 py-5">
                            <h2 className="text-base font-semibold">{group.title}</h2>
                            <div className="mt-4 space-y-3">
                                {group.items.map((item) => (
                                    <div key={`${group.title}-${item.title}`} className="rounded-md border bg-muted/20 px-4 py-3">
                                        <div className="text-sm font-medium">{item.title}</div>
                                        <p className="mt-2 text-sm text-muted-foreground">{item.detail}</p>
                                        <div className="mt-2 text-xs text-muted-foreground">{item.meta}</div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            );
        case "detail":
            return (
                <div className="space-y-4">
                    <div className="rounded-lg border bg-card px-5 py-5">
                        <p className="text-sm text-muted-foreground">{schema.summary}</p>
                        <div className="mt-4 grid gap-3 md:grid-cols-2">
                            {schema.fields.map((field) => (
                                <div key={field.label} className="rounded-md border bg-muted/20 px-4 py-3">
                                    <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                                        {field.label}
                                    </div>
                                    <div className="mt-2 text-sm">{field.value}</div>
                                </div>
                            ))}
                        </div>
                    </div>
                    <div className="rounded-lg border bg-card px-5 py-5">
                        <h2 className="text-base font-semibold">时间线</h2>
                        <div className="mt-4 space-y-3">
                            {schema.timeline.map((item) => (
                                <div key={`${item.meta}-${item.title}`} className="rounded-md border bg-muted/20 px-4 py-3">
                                    <div className="text-sm font-medium">{item.title}</div>
                                    <p className="mt-2 text-sm text-muted-foreground">{item.detail}</p>
                                    <div className="mt-2 text-xs text-muted-foreground">{item.meta}</div>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            );
        case "form":
            return (
                <div className="grid gap-3 md:grid-cols-2">
                    {schema.fields.map((field) => (
                        <div key={field.label} className="rounded-lg border bg-card px-4 py-4">
                            <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                                {field.label}
                            </div>
                            <div className="mt-2 text-sm">{field.value}</div>
                        </div>
                    ))}
                </div>
            );
        case "graph":
            return (
                <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
                    <div className="rounded-lg border bg-card px-5 py-5">
                        <h2 className="text-base font-semibold">节点</h2>
                        <div className="mt-4 space-y-3">
                            {schema.nodes.map((node) => (
                                <div key={node.id} className="rounded-md border bg-muted/20 px-4 py-3">
                                    <div className="text-sm font-medium">{node.label}</div>
                                    <div className="mt-1 text-xs text-muted-foreground">{node.category}</div>
                                    <p className="mt-2 text-sm text-muted-foreground">{node.description}</p>
                                </div>
                            ))}
                        </div>
                    </div>
                    <div className="rounded-lg border bg-card px-5 py-5">
                        <h2 className="text-base font-semibold">边</h2>
                        <div className="mt-4 space-y-3">
                            {schema.edges.map((edge, index) => (
                                <div key={`${edge.source}-${edge.target}-${index}`} className="rounded-md border bg-muted/20 px-4 py-3">
                                    <div className="text-sm font-medium">
                                        {edge.source} → {edge.target}
                                    </div>
                                    <div className="mt-1 text-xs text-muted-foreground">
                                        {edge.kind}
                                        {edge.label ? ` · ${edge.label}` : ""}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            );
        default:
            return (
                <div className="rounded-lg border bg-card px-5 py-5 text-sm text-muted-foreground">
                    当前页面 schema 暂未支持。
                </div>
            );
    }
}
