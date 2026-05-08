import { useState, type ReactNode } from "react";
import {
    Badge,
    Button,
    Card,
    CardContent,
    Input,
    ScrollArea,
    Separator,
    Textarea,
    cn,
} from "@addzero/ui";
import {
    AudioLines,
    Bot,
    Box,
    Cable,
    Component,
    Focus,
    Grid3X3,
    LayoutPanelTop,
    Play,
    ScanSearch,
    Sparkles,
    WandSparkles,
} from "lucide-react";

type StudioNode = {
    id: string;
    label: string;
    type: string;
    grid: string;
    accent: string;
    detail: string;
    action: string;
};

const paletteGroups = [
    {
        label: "Host Built-in",
        tone: "rose",
        items: [
            { label: "Hero frame", type: "layout.hero" },
            { label: "Metric rail", type: "data.metric-rail" },
            { label: "Tape button", type: "action.button" },
            { label: "Asset reel", type: "media.asset-reel" },
        ],
    },
    {
        label: "Plugin Generated",
        tone: "mint",
        items: [
            { label: "Install console", type: "plugin.install-console" },
            { label: "Action binder", type: "plugin.action-binder" },
            { label: "Prompt cassette", type: "ai.prompt-cassette" },
            { label: "Preview stage", type: "ai.preview-stage" },
        ],
    },
];

const canvasNodes: StudioNode[] = [
    {
        id: "hero",
        label: "Studio Hero",
        type: "layout.hero",
        grid: "01 / 01 / 13 / 04",
        accent: "rose",
        detail: "Centered entry frame for plugin identity, runtime counts, and editing mode.",
        action: "navigate: /market/wasm",
    },
    {
        id: "catalog",
        label: "Catalog Track",
        type: "plugin.install-console",
        grid: "01 / 04 / 05 / 09",
        accent: "mint",
        detail: "Shows catalog status, package root, and installable wasm plugin bundles.",
        action: "run_http: install_catalog_plugin",
    },
    {
        id: "canvas",
        label: "Canvas Stage",
        type: "layout.canvas",
        grid: "05 / 04 / 09 / 09",
        accent: "cream",
        detail: "Main low-code surface. Built-in nodes are dragged here; generated blocks are mounted here.",
        action: "open_vibe_task: generate_subtree",
    },
    {
        id: "props",
        label: "Props + Actions",
        type: "plugin.action-binder",
        grid: "09 / 04 / 13 / 09",
        accent: "sage",
        detail: "Props editor, event bindings, and reusable button action recipes.",
        action: "emit_event: node_selected",
    },
];

const vibePresets = [
    {
        title: "Generate block",
        detail: "Create a centered asset upload + plugin install block for this page.",
    },
    {
        title: "Rewrite action",
        detail: "Turn the selected button into an `open_vibe_task` launcher with node context.",
    },
    {
        title: "Promote component",
        detail: "Lift the selected subtree into a reusable plugin-scoped component bundle.",
    },
];

const timeline = [
    { label: "CATALOG", value: "09", tone: "rose" },
    { label: "CANVAS", value: "14", tone: "mint" },
    { label: "ACTION", value: "22", tone: "sage" },
    { label: "VIBE", value: "31", tone: "amber" },
];

export function WasmStudioWorkbench({
    embedded = false,
}: {
    embedded?: boolean;
}) {
    const [selectedNodeId, setSelectedNodeId] = useState("canvas");
    const [prompt, setPrompt] = useState(
        "Generate a hyper-symmetrical plugin page for wasm package onboarding, with a centered hero reel, stepped install flow, and a mechanical action button that opens vibe editing for the selected node.",
    );

    const selectedNode =
        canvasNodes.find((node) => node.id === selectedNodeId) ?? canvasNodes[0];

    return (
        <div
            className={cn(
                "text-[#1f1a17]",
                embedded ? "rounded-[32px] bg-[#f3eadc]" : "min-h-full bg-[#f3eadc]",
            )}
            style={{
                fontFamily:
                    '"Courier Prime", "IBM Plex Mono", "SFMono-Regular", monospace',
            }}
        >
            <div
                className={cn(
                    "mx-auto flex w-full max-w-[1800px] flex-col gap-6",
                    embedded ? "px-4 py-4 xl:px-6" : "px-6 py-6 xl:px-10",
                )}
            >
                <header className="relative overflow-hidden rounded-[28px] border border-[#5b5145]/20 bg-[linear-gradient(135deg,#f8efe4_0%,#f1e4d7_42%,#e5efe7_100%)] shadow-[0_30px_80px_rgba(54,42,31,0.12)]">
                    <div className="absolute inset-x-0 top-0 h-px bg-[#fff9f2]/80" />
                    <div className="grid gap-6 px-8 py-8 lg:grid-cols-[1fr_auto_1fr] lg:items-center">
                        <div className="flex items-center gap-3 lg:justify-start">
                            <Badge className="rounded-full bg-[#eed4cf] px-3 py-1 text-[11px] uppercase tracking-[0.28em] text-[#6b4b48]">
                                Wasm Plugin Studio
                            </Badge>
                            <Badge
                                variant="outline"
                                className="rounded-full border-[#7b776d] bg-transparent px-3 py-1 text-[11px] uppercase tracking-[0.22em] text-[#4f493f]"
                            >
                                Motion-led canvas
                            </Badge>
                        </div>

                        <div className="text-center">
                            <div
                                className="text-[40px] uppercase leading-none tracking-[0.18em] text-[#2f2822] sm:text-[56px]"
                                style={{
                                    fontFamily:
                                        '"Futura PT", Futura, "Avenir Next", sans-serif',
                                }}
                            >
                                CENTER STAGE
                            </div>
                            <p className="mx-auto mt-3 max-w-[720px] text-sm leading-6 text-[#5d5448]">
                                A unified workbench for fixed host components, plugin-delivered
                                surfaces, and online vibe-generated blocks. The page itself is the
                                canvas document.
                            </p>
                        </div>

                        <div className="flex flex-wrap items-center gap-3 lg:justify-end">
                            <MechanicalPill icon={Cable} label="catalog" value="18 bundles" />
                            <MechanicalPill icon={Grid3X3} label="grid" value="12 x 8" />
                            <MechanicalPill icon={Bot} label="vibe" value="node-aware" />
                        </div>
                    </div>
                </header>

                <section className="grid gap-6 xl:grid-cols-[320px_minmax(0,1fr)_320px]">
                    <SidePanel
                        title="Component Registry"
                        eyebrow="Built-in + generated"
                        tone="rose"
                    >
                        <div className="space-y-5">
                            {paletteGroups.map((group) => (
                                <div key={group.label} className="space-y-3">
                                    <div className="flex items-center justify-between">
                                        <div className="text-[11px] uppercase tracking-[0.26em] text-[#6b6155]">
                                            {group.label}
                                        </div>
                                        <Badge
                                            variant="outline"
                                            className="rounded-full border-[#7b776d]/40 bg-[#fffaf2]/70 px-2 py-0.5 text-[10px] uppercase tracking-[0.16em]"
                                        >
                                            {group.tone}
                                        </Badge>
                                    </div>
                                    <div className="grid gap-3">
                                        {group.items.map((item) => (
                                            <button
                                                key={item.label}
                                                type="button"
                                                className="rounded-[18px] border border-[#5c5247]/15 bg-[#fffaf2]/80 px-4 py-4 text-left transition-transform hover:-translate-y-0.5 hover:bg-white"
                                            >
                                                <div className="flex items-center justify-between gap-3">
                                                    <span
                                                        className="text-sm uppercase tracking-[0.08em] text-[#2e2722]"
                                                        style={{
                                                            fontFamily:
                                                                '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                        }}
                                                    >
                                                        {item.label}
                                                    </span>
                                                    <Component className="size-4 text-[#7a6d5f]" />
                                                </div>
                                                <div className="mt-2 text-xs text-[#72685d]">
                                                    {item.type}
                                                </div>
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </SidePanel>

                    <main className="space-y-6">
                        <Card className="overflow-hidden rounded-[30px] border-[#574c41]/15 bg-[linear-gradient(180deg,#fbf6ee_0%,#f2e8da_100%)] shadow-[0_25px_70px_rgba(53,41,28,0.14)]">
                            <CardContent className="p-0">
                                <div className="border-b border-[#574c41]/10 px-6 py-4">
                                    <div className="grid gap-4 lg:grid-cols-[1fr_auto_1fr] lg:items-center">
                                        <div className="flex items-center gap-3">
                                            <StudioModeChip
                                                icon={LayoutPanelTop}
                                                label="Canvas document"
                                            />
                                            <StudioModeChip icon={ScanSearch} label="Selection sync" />
                                        </div>
                                        <div className="flex items-center justify-center gap-2">
                                            <Button
                                                className="rounded-full border border-[#6f6559] bg-[#2f2822] px-5 text-[#f9f1e7] hover:bg-[#231d18]"
                                            >
                                                <WandSparkles className="size-4" />
                                                Generate Block
                                            </Button>
                                            <Button
                                                variant="outline"
                                                className="rounded-full border-[#786c60] bg-[#fff6ed] px-5"
                                            >
                                                <Sparkles className="size-4" />
                                                Generate Component
                                            </Button>
                                        </div>
                                        <div className="flex items-center justify-end gap-3">
                                            {timeline.map((item) => (
                                                <div
                                                    key={item.label}
                                                    className="flex min-w-[64px] flex-col items-center rounded-[16px] border border-[#5a4f43]/10 bg-[#fff9f2]/75 px-3 py-2"
                                                >
                                                    <div className="text-[10px] uppercase tracking-[0.24em] text-[#776b5f]">
                                                        {item.label}
                                                    </div>
                                                    <div
                                                        className={cn(
                                                            "mt-1 text-lg leading-none",
                                                            item.tone === "rose" && "text-[#9a625d]",
                                                            item.tone === "mint" && "text-[#4d7c68]",
                                                            item.tone === "sage" && "text-[#556a59]",
                                                            item.tone === "amber" && "text-[#8f6840]",
                                                        )}
                                                        style={{
                                                            fontFamily:
                                                                '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                        }}
                                                    >
                                                        {item.value}
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                </div>

                                <div className="grid gap-0 lg:grid-cols-[1fr_1.2fr_1fr]">
                                    <StageStrip
                                        title="Plugin Runtime"
                                        tone="rose"
                                        body="Catalog registration, package root, and install rail. This is where `.azplugin` bundles enter the system."
                                        metric="18"
                                    />
                                    <div className="border-x border-[#574c41]/10 bg-[radial-gradient(circle_at_top,#fff9f2_0%,#f5eadb_38%,#efe1cf_100%)] px-6 py-6">
                                        <div className="mx-auto max-w-[680px] rounded-[28px] border border-[#4f453b]/12 bg-[#fffaf3]/70 p-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.7)]">
                                            <div className="mb-4 flex items-center justify-between">
                                                <div>
                                                    <div className="text-[11px] uppercase tracking-[0.28em] text-[#7d7367]">
                                                        Canvas Stage
                                                    </div>
                                                    <div
                                                        className="mt-2 text-2xl uppercase tracking-[0.14em] text-[#2f2822]"
                                                        style={{
                                                            fontFamily:
                                                                '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                        }}
                                                    >
                                                        Plugin Page Surface
                                                    </div>
                                                </div>
                                                <Badge
                                                    variant="outline"
                                                    className="rounded-full border-[#786c60]/30 bg-[#fff] px-3 py-1 text-[10px] uppercase tracking-[0.18em]"
                                                >
                                                    12-column frame
                                                </Badge>
                                            </div>

                                            <div className="grid grid-cols-12 gap-3 rounded-[22px] border border-dashed border-[#7a6f62]/30 bg-[#f7efe4]/80 p-4">
                                                {canvasNodes.map((node) => (
                                                    <button
                                                        key={node.id}
                                                        type="button"
                                                        onClick={() => setSelectedNodeId(node.id)}
                                                        className={cn(
                                                            "col-span-12 rounded-[20px] border px-4 py-4 text-left transition-all md:col-span-4",
                                                            node.id === "hero" && "md:col-span-12",
                                                            node.id === "catalog" && "md:col-span-4",
                                                            node.id === "canvas" && "md:col-span-4",
                                                            node.id === "props" && "md:col-span-4",
                                                            selectedNodeId === node.id
                                                                ? "border-[#2f2822] bg-white shadow-[0_16px_30px_rgba(36,28,21,0.16)]"
                                                                : "border-[#5e5347]/10 bg-[#fffaf4]/82 hover:border-[#5e5347]/30 hover:bg-white",
                                                        )}
                                                    >
                                                        <div className="flex items-start justify-between gap-3">
                                                            <div>
                                                                <div
                                                                    className="text-sm uppercase tracking-[0.12em] text-[#2d2622]"
                                                                    style={{
                                                                        fontFamily:
                                                                            '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                                    }}
                                                                >
                                                                    {node.label}
                                                                </div>
                                                                <div className="mt-2 text-xs text-[#756b61]">
                                                                    {node.type}
                                                                </div>
                                                            </div>
                                                            <Focus className="size-4 text-[#7a6f62]" />
                                                        </div>
                                                        <div className="mt-4 text-xs leading-5 text-[#5e564f]">
                                                            {node.detail}
                                                        </div>
                                                        <div className="mt-4 flex items-center justify-between text-[10px] uppercase tracking-[0.18em] text-[#8a8075]">
                                                            <span>{node.grid}</span>
                                                            <span>{node.accent}</span>
                                                        </div>
                                                    </button>
                                                ))}
                                            </div>
                                        </div>
                                    </div>
                                    <StageStrip
                                        title="Action Binding"
                                        tone="mint"
                                        body="Props patches, event bindings, and the bridge from static controls to node-aware vibe tasks."
                                        metric="31"
                                    />
                                </div>
                            </CardContent>
                        </Card>

                        <div className="grid gap-6 lg:grid-cols-[1fr_1fr]">
                            <Card className="rounded-[28px] border-[#5a4f43]/12 bg-[#fff9f2] shadow-[0_18px_44px_rgba(49,40,31,0.09)]">
                                <CardContent className="p-6">
                                    <div className="flex items-center justify-between">
                                        <div>
                                            <div className="text-[11px] uppercase tracking-[0.28em] text-[#776d62]">
                                                Selected node
                                            </div>
                                            <div
                                                className="mt-2 text-2xl uppercase tracking-[0.14em] text-[#2f2822]"
                                                style={{
                                                    fontFamily:
                                                        '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                }}
                                            >
                                                {selectedNode.label}
                                            </div>
                                        </div>
                                        <Badge
                                            variant="outline"
                                            className="rounded-full border-[#6d6258]/30 bg-white px-3 py-1 text-[10px] uppercase tracking-[0.18em]"
                                        >
                                            {selectedNode.type}
                                        </Badge>
                                    </div>
                                    <div className="mt-5 grid gap-4 md:grid-cols-2">
                                        <MechanicalField label="Grid area" value={selectedNode.grid} />
                                        <MechanicalField label="Primary action" value={selectedNode.action} />
                                        <MechanicalField
                                            label="Source"
                                            value={
                                                selectedNode.type.startsWith("plugin.")
                                                    ? "plugin generated"
                                                    : "host built-in"
                                            }
                                        />
                                        <MechanicalField label="Selection mode" value="node + props + event" />
                                    </div>
                                    <div className="mt-5 rounded-[20px] border border-[#5a4f43]/10 bg-[#f7eee3] p-4 text-sm leading-6 text-[#5c554c]">
                                        {selectedNode.detail}
                                    </div>
                                </CardContent>
                            </Card>

                            <Card className="rounded-[28px] border-[#5a4f43]/12 bg-[#fff9f2] shadow-[0_18px_44px_rgba(49,40,31,0.09)]">
                                <CardContent className="p-6">
                                    <div className="flex items-center justify-between">
                                        <div>
                                            <div className="text-[11px] uppercase tracking-[0.28em] text-[#776d62]">
                                                Vibe launch prompt
                                            </div>
                                            <div
                                                className="mt-2 text-2xl uppercase tracking-[0.14em] text-[#2f2822]"
                                                style={{
                                                    fontFamily:
                                                        '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                }}
                                            >
                                                Node-aware generation
                                            </div>
                                        </div>
                                        <Button
                                            className="rounded-full border border-[#6f6559] bg-[#2f2822] px-5 text-[#f9f1e7] hover:bg-[#231d18]"
                                        >
                                            <Play className="size-4" />
                                            Open Vibe Task
                                        </Button>
                                    </div>
                                    <div className="mt-5">
                                        <Textarea
                                            value={prompt}
                                            onChange={(event) => setPrompt(event.target.value)}
                                            className="min-h-[180px] rounded-[22px] border-[#5a4f43]/12 bg-[#fffdf9] px-4 py-4 text-sm leading-6"
                                        />
                                    </div>
                                    <div className="mt-4 grid gap-3 md:grid-cols-3">
                                        {vibePresets.map((preset) => (
                                            <button
                                                key={preset.title}
                                                type="button"
                                                onClick={() => setPrompt(preset.detail)}
                                                className="rounded-[18px] border border-[#5a4f43]/10 bg-[#f8f0e6] px-4 py-4 text-left hover:bg-white"
                                            >
                                                <div
                                                    className="text-sm uppercase tracking-[0.08em] text-[#2f2822]"
                                                    style={{
                                                        fontFamily:
                                                            '"Futura PT", Futura, "Avenir Next", sans-serif',
                                                    }}
                                                >
                                                    {preset.title}
                                                </div>
                                                <div className="mt-2 text-xs leading-5 text-[#665d53]">
                                                    {preset.detail}
                                                </div>
                                            </button>
                                        ))}
                                    </div>
                                </CardContent>
                            </Card>
                        </div>
                    </main>

                    <SidePanel
                        title="Node Inspector"
                        eyebrow="Props / events / runtime"
                        tone="mint"
                    >
                        <ScrollArea className="h-[980px] pr-4">
                            <div className="space-y-5">
                                <InspectorSection title="Props tape">
                                    <Input
                                        value={selectedNode.label}
                                        readOnly
                                        className="rounded-[16px] border-[#655a4f]/12 bg-[#fffef9]"
                                    />
                                    <Input
                                        value={selectedNode.grid}
                                        readOnly
                                        className="rounded-[16px] border-[#655a4f]/12 bg-[#fffef9]"
                                    />
                                </InspectorSection>

                                <Separator />

                                <InspectorSection title="Action slot">
                                    <div className="rounded-[18px] border border-[#62574a]/10 bg-[#fffaf4] p-4">
                                        <div className="flex items-center gap-2 text-sm text-[#2f2822]">
                                            <AudioLines className="size-4 text-[#556a59]" />
                                            {selectedNode.action}
                                        </div>
                                        <div className="mt-2 text-xs leading-5 text-[#6b6157]">
                                            Buttons in this studio are fixed host controls whose
                                            action target can be rebound to `open_vibe_task`,
                                            `run_http`, or `emit_event`.
                                        </div>
                                    </div>
                                </InspectorSection>

                                <Separator />

                                <InspectorSection title="Runtime rail">
                                    <div className="grid gap-3">
                                        <RuntimeTag label="package root" value="~/.addzero/aio/wasm-plugin-host" />
                                        <RuntimeTag label="catalog dir" value="~/.addzero/aio/wasm-plugin-catalog" />
                                        <RuntimeTag label="page source" value="canvas_document" />
                                        <RuntimeTag label="promotion" value="plugin asset bundle" />
                                    </div>
                                </InspectorSection>
                            </div>
                        </ScrollArea>
                    </SidePanel>
                </section>
            </div>
        </div>
    );
}

export default function WasmStudioPage() {
    return <WasmStudioWorkbench />;
}

function MechanicalPill({
    icon: Icon,
    label,
    value,
}: {
    icon: typeof Cable;
    label: string;
    value: string;
}) {
    return (
        <div className="flex items-center gap-3 rounded-full border border-[#6b6054]/15 bg-[#fff7ef]/80 px-4 py-2">
            <Icon className="size-4 text-[#5f5448]" />
            <div>
                <div className="text-[10px] uppercase tracking-[0.24em] text-[#7d7164]">
                    {label}
                </div>
                <div
                    className="text-sm uppercase tracking-[0.08em] text-[#2f2822]"
                    style={{ fontFamily: '"Futura PT", Futura, "Avenir Next", sans-serif' }}
                >
                    {value}
                </div>
            </div>
        </div>
    );
}

function StudioModeChip({
    icon: Icon,
    label,
}: {
    icon: typeof LayoutPanelTop;
    label: string;
}) {
    return (
        <div className="flex items-center gap-2 rounded-full border border-[#655a4f]/12 bg-[#fff8f0] px-4 py-2 text-[11px] uppercase tracking-[0.2em] text-[#4f473f]">
            <Icon className="size-4" />
            {label}
        </div>
    );
}

function SidePanel({
    title,
    eyebrow,
    tone,
    children,
}: {
    title: string;
    eyebrow: string;
    tone: string;
    children: ReactNode;
}) {
    return (
        <Card className="overflow-hidden rounded-[30px] border-[#5a4f43]/12 bg-[#fff8f0] shadow-[0_20px_50px_rgba(45,36,28,0.08)]">
            <CardContent className="p-0">
                <div className="border-b border-[#574c41]/10 px-6 py-5">
                    <div className="text-[11px] uppercase tracking-[0.28em] text-[#7c7266]">
                        {eyebrow}
                    </div>
                    <div
                        className="mt-2 text-[28px] uppercase tracking-[0.14em] text-[#2f2822]"
                        style={{ fontFamily: '"Futura PT", Futura, "Avenir Next", sans-serif' }}
                    >
                        {title}
                    </div>
                    <div className="mt-3 flex items-center gap-2 text-[10px] uppercase tracking-[0.2em] text-[#776b60]">
                        <Box className="size-4" />
                        tone / {tone}
                    </div>
                </div>
                <div className="p-6">{children}</div>
            </CardContent>
        </Card>
    );
}

function StageStrip({
    title,
    body,
    metric,
    tone,
}: {
    title: string;
    body: string;
    metric: string;
    tone: string;
}) {
    return (
        <div className="flex flex-col justify-between px-5 py-6">
            <div>
                <div className="text-[11px] uppercase tracking-[0.28em] text-[#7a6f63]">
                    {title}
                </div>
                <div className="mt-4 text-sm leading-6 text-[#5b534a]">{body}</div>
            </div>
            <div className="mt-6 rounded-[22px] border border-[#5b5145]/10 bg-[#fffaf3] px-4 py-4">
                <div className="text-[10px] uppercase tracking-[0.24em] text-[#7f7468]">
                    indicator
                </div>
                <div
                    className={cn(
                        "mt-2 text-[42px] leading-none",
                        tone === "rose" && "text-[#9f6760]",
                        tone === "mint" && "text-[#4f7966]",
                    )}
                    style={{ fontFamily: '"Futura PT", Futura, "Avenir Next", sans-serif' }}
                >
                    {metric}
                </div>
            </div>
        </div>
    );
}

function MechanicalField({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-[18px] border border-[#5b5145]/10 bg-[#fffaf4] px-4 py-4">
            <div className="text-[10px] uppercase tracking-[0.24em] text-[#7a7064]">
                {label}
            </div>
            <div className="mt-2 text-sm leading-6 text-[#2f2822]">{value}</div>
        </div>
    );
}

function InspectorSection({
    title,
    children,
}: {
    title: string;
    children: ReactNode;
}) {
    return (
        <section className="space-y-3">
            <div
                className="text-sm uppercase tracking-[0.12em] text-[#2f2822]"
                style={{ fontFamily: '"Futura PT", Futura, "Avenir Next", sans-serif' }}
            >
                {title}
            </div>
            <div className="space-y-3">{children}</div>
        </section>
    );
}

function RuntimeTag({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-[16px] border border-[#5a4f43]/10 bg-[#fffdf8] px-4 py-3">
            <div className="text-[10px] uppercase tracking-[0.22em] text-[#807568]">
                {label}
            </div>
            <div className="mt-2 break-all text-xs leading-5 text-[#564d44]">{value}</div>
        </div>
    );
}
