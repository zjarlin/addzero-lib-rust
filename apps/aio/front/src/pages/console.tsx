import { useCallback, useEffect, useState } from "react";
import {
    Play,
    Terminal,
    Loader2,
    Save,
    Download,
    Trash2,
    WandSparkles,
} from "lucide-react";
import Editor from "@monaco-editor/react";
import { getApiBaseUrl } from "@addzero/api-client";
import {
    Badge,
    Button,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
    Input,
    ScrollArea,
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
    Textarea,
} from "@addzero/ui";

interface RunResult {
    exit_code: number;
    stdout: string;
    stderr: string;
    vars: Record<string, unknown>;
    duration_ms: number;
}

interface EnvResult {
    vars: Record<string, string | number | boolean | object>;
    stdout: string;
    stderr: string;
}

interface VibeCodingResponse {
    session: {
        summary: {
            id: string;
        };
    };
    bootstrap_prompt: string;
}

interface TerminalSessionSnapshot {
    summary: {
        id: string;
        title: string;
        state: string;
        cwd: string;
        profile: string;
        updated_at: string;
        exit_code: number | null;
        last_error: string | null;
    };
    screen: string;
    cursor_row: number;
    cursor_col: number;
}

const DEFAULT_SOURCE = `// Rhai 脚本示例
print("Hello from AIO Platform!");

let name = "World";
let greeting = \`Hello, \${name}!\`;

greeting;
`;

export default function ScriptConsole() {
    const [source, setSource] = useState(DEFAULT_SOURCE);
    const [scriptName, setScriptName] = useState("");
    const [savedScripts, setSavedScripts] = useState<string[]>([]);
    const [result, setResult] = useState<RunResult | null>(null);
    const [envResult, setEnvResult] = useState<EnvResult | null>(null);
    const [running, setRunning] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<"run" | "env">("run");
    const [vibeCwd, setVibeCwd] = useState("");
    const [vibeGoal, setVibeGoal] = useState("");
    const [vibeSkillPath, setVibeSkillPath] = useState("");
    const [vibeContext, setVibeContext] = useState("");
    const [vibeLaunching, setVibeLaunching] = useState(false);
    const [vibeSessionId, setVibeSessionId] = useState<string | null>(null);
    const [vibeSession, setVibeSession] = useState<TerminalSessionSnapshot | null>(
        null,
    );
    const [vibeInput, setVibeInput] = useState("");
    const [vibeBusy, setVibeBusy] = useState(false);
    const [vibeError, setVibeError] = useState<string | null>(null);

    const baseUrl = getApiBaseUrl();

    const fetchScripts = useCallback(async () => {
        try {
            const res = await fetch(`${baseUrl}/api/scripts`, {
                credentials: "include",
            });
            if (res.ok) setSavedScripts(await res.json());
        } catch {
            /* offline — ignore */
        }
    }, [baseUrl]);

    useEffect(() => {
        fetchScripts();
    }, [fetchScripts]);

    useEffect(() => {
        if (!vibeCwd) {
            setVibeCwd("/Users/zjarlin/IdeaProjects/zjarlin/addzero-lib-rust");
        }
    }, [vibeCwd]);

    useEffect(() => {
        if (!vibeSessionId) {
            return;
        }

        let cancelled = false;

        const loadSnapshot = async () => {
            try {
                const res = await fetch(
                    `${baseUrl}/api/admin/terminal/sessions/${vibeSessionId}`,
                    {
                        credentials: "include",
                    },
                );
                if (!res.ok) {
                    throw new Error(`HTTP ${res.status}: ${await res.text()}`);
                }
                const data: TerminalSessionSnapshot = await res.json();
                if (!cancelled) {
                    setVibeSession(data);
                    setVibeError(null);
                }
            } catch (err) {
                if (!cancelled) {
                    setVibeError(
                        err instanceof Error
                            ? err.message
                            : "加载会话快照失败",
                    );
                }
            }
        };

        void loadSnapshot();
        const timer = window.setInterval(() => {
            void loadSnapshot();
        }, 1500);

        return () => {
            cancelled = true;
            window.clearInterval(timer);
        };
    }, [baseUrl, vibeSessionId]);

    async function runScript() {
        setRunning(true);
        setError(null);
        setResult(null);
        setEnvResult(null);
        try {
            const res = await fetch(`${baseUrl}/api/engine/rhai/run`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ source, vars: {} }),
            });
            if (!res.ok)
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            setResult(await res.json());
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        } finally {
            setRunning(false);
        }
    }

    async function evalEnv() {
        setRunning(true);
        setError(null);
        setResult(null);
        setEnvResult(null);
        try {
            const res = await fetch(`${baseUrl}/api/engine/rhai/eval-env`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ source, vars: {} }),
            });
            if (!res.ok)
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            setEnvResult(await res.json());
            setActiveTab("env");
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        } finally {
            setRunning(false);
        }
    }

    async function saveScript() {
        const name = scriptName.trim() || "untitled";
        try {
            await fetch(`${baseUrl}/api/scripts`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ name, source }),
            });
            setScriptName(name);
            fetchScripts();
        } catch (err) {
            setError(err instanceof Error ? err.message : "Save failed");
        }
    }

    async function loadScript(name: string) {
        try {
            const res = await fetch(
                `${baseUrl}/api/scripts/${encodeURIComponent(name)}`,
                {
                    credentials: "include",
                },
            );
            if (!res.ok) throw new Error("Not found");
            const data = await res.json();
            setSource(data.source);
            setScriptName(data.name);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Load failed");
        }
    }

    async function deleteScript(name: string) {
        try {
            await fetch(`${baseUrl}/api/scripts/${encodeURIComponent(name)}`, {
                method: "DELETE",
                credentials: "include",
            });
            fetchScripts();
        } catch (err) {
            setError(err instanceof Error ? err.message : "Delete failed");
        }
    }

    async function startVibeCoding() {
        setVibeLaunching(true);
        setError(null);
        setVibeError(null);
        try {
            const res = await fetch(`${baseUrl}/api/vibe-coding/start`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    profile: "codex",
                    cwd: vibeCwd,
                    goal: vibeGoal,
                    skill_path: vibeSkillPath || undefined,
                    window_context: vibeContext || undefined,
                    title: "Codex Vibe Coding",
                }),
            });
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            const data: VibeCodingResponse = await res.json();
            setVibeSessionId(data.session.summary.id);
            setVibeSession(data.session as TerminalSessionSnapshot);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Vibe coding failed");
        } finally {
            setVibeLaunching(false);
        }
    }

    async function sendVibeInput() {
        if (!vibeSessionId || !vibeInput.trim()) {
            return;
        }

        setVibeBusy(true);
        setVibeError(null);
        try {
            const res = await fetch(
                `${baseUrl}/api/admin/terminal/sessions/${vibeSessionId}/input`,
                {
                    method: "POST",
                    credentials: "include",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ data: `${vibeInput}\n` }),
                },
            );
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            const data: TerminalSessionSnapshot = await res.json();
            setVibeSession(data);
            setVibeInput("");
        } catch (err) {
            setVibeError(
                err instanceof Error ? err.message : "发送终端输入失败",
            );
        } finally {
            setVibeBusy(false);
        }
    }

    async function closeVibeSession() {
        if (!vibeSessionId) {
            return;
        }

        setVibeBusy(true);
        setVibeError(null);
        try {
            const res = await fetch(
                `${baseUrl}/api/admin/terminal/sessions/${vibeSessionId}`,
                {
                    method: "DELETE",
                    credentials: "include",
                },
            );
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}: ${await res.text()}`);
            }
            setVibeSessionId(null);
            setVibeSession(null);
            setVibeInput("");
        } catch (err) {
            setVibeError(
                err instanceof Error ? err.message : "关闭终端会话失败",
            );
        } finally {
            setVibeBusy(false);
        }
    }

    return (
        <div className="space-y-6">
            <div>
                <h1 className="text-3xl font-bold tracking-tight">脚本控制台</h1>
                <p className="mt-1 text-muted-foreground">
                    在线编写、保存和运行 Rhai 脚本 · 支持环境变量配置
                </p>
            </div>

            <Card className="border-primary/20 bg-card/80 shadow-none">
                <CardHeader>
                    <div className="flex items-center gap-2 text-sm font-semibold">
                        <WandSparkles className="h-4 w-4 text-primary" />
                        Vibe Coding
                    </div>
                    <CardDescription>
                        直接在当前窗口上下文里启动 Codex CLI，会自动注入工作目录、目标和可选 skill 路径。
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="grid gap-3 lg:grid-cols-2">
                        <Input
                            value={vibeCwd}
                            onChange={(e) => setVibeCwd(e.target.value)}
                            placeholder="工作目录"
                            className="bg-background/80"
                        />
                        <Input
                            value={vibeSkillPath}
                            onChange={(e) => setVibeSkillPath(e.target.value)}
                            placeholder="插件开发文档.skill 路径（可选）"
                            className="bg-background/80"
                        />
                        <Textarea
                            value={vibeGoal}
                            onChange={(e) => setVibeGoal(e.target.value)}
                            placeholder="告诉 Codex 当前想完成什么"
                            className="min-h-24 bg-background/80 lg:col-span-2"
                        />
                        <Textarea
                            value={vibeContext}
                            onChange={(e) => setVibeContext(e.target.value)}
                            placeholder="当前窗口上下文（例如：插件市场页、当前正在做 registry/install/vibe coding 自举）"
                            className="min-h-20 bg-background/80 lg:col-span-2"
                        />
                    </div>
                    <div className="flex flex-wrap items-center gap-3">
                        <Button
                            type="button"
                            onClick={startVibeCoding}
                            disabled={vibeLaunching || !vibeCwd.trim() || !vibeGoal.trim()}
                        >
                            {vibeLaunching ? (
                                <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                                <WandSparkles className="h-4 w-4" />
                            )}
                            启动 Codex 会话
                        </Button>
                        {vibeSessionId ? (
                            <Badge variant="secondary" className="font-mono">
                                {vibeSessionId}
                            </Badge>
                        ) : null}
                    </div>
                    {vibeError ? (
                        <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                            {vibeError}
                        </div>
                    ) : null}
                    {vibeSession ? (
                        <Card className="bg-background/70 shadow-none">
                            <CardContent className="space-y-3 p-4">
                                <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground">
                                    <div className="flex flex-wrap items-center gap-2">
                                        <span className="font-medium text-foreground">
                                            {vibeSession.summary.title}
                                        </span>
                                        <Badge variant="secondary">
                                            {vibeSession.summary.state}
                                        </Badge>
                                        <span className="font-mono">
                                            {vibeSession.summary.cwd}
                                        </span>
                                    </div>
                                    <Button
                                        type="button"
                                        variant="destructive"
                                        size="sm"
                                        onClick={closeVibeSession}
                                        disabled={vibeBusy}
                                    >
                                        <Trash2 className="h-3.5 w-3.5" />
                                        关闭会话
                                    </Button>
                                </div>
                                <ScrollArea className="h-80 rounded-lg border bg-black px-3 py-3">
                                    <pre className="text-xs leading-5 whitespace-pre-wrap text-emerald-100">
                                        {vibeSession.screen || "等待终端输出..."}
                                    </pre>
                                </ScrollArea>
                                <div className="flex gap-3">
                                    <Input
                                        value={vibeInput}
                                        onChange={(e) => setVibeInput(e.target.value)}
                                        onKeyDown={(e) => {
                                            if (e.key === "Enter") {
                                                void sendVibeInput();
                                            }
                                        }}
                                        placeholder="向当前 Codex 会话发送一行输入"
                                        className="bg-background/80"
                                    />
                                    <Button
                                        type="button"
                                        variant="outline"
                                        onClick={sendVibeInput}
                                        disabled={vibeBusy || !vibeInput.trim()}
                                    >
                                        发送
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    ) : null}
                </CardContent>
            </Card>

            <div className="grid gap-6 lg:grid-cols-[16rem_1fr_1fr]">
                <Card className="bg-card/80 shadow-none">
                    <CardHeader className="pb-3">
                        <CardTitle className="text-sm uppercase tracking-[0.2em] text-muted-foreground">
                            已保存
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="p-0">
                        <ScrollArea className="h-80 px-2 pb-2">
                            {savedScripts.length === 0 ? (
                                <p className="p-4 text-xs text-muted-foreground">
                                    暂无脚本
                                </p>
                            ) : (
                                <div className="space-y-1">
                                    {savedScripts.map((name) => (
                                        <div
                                            key={name}
                                            className="group flex items-center justify-between gap-2 rounded-md px-2 py-1"
                                        >
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                className="h-9 flex-1 justify-start"
                                                onClick={() => loadScript(name)}
                                            >
                                                {name}.rhai
                                            </Button>
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                size="icon"
                                                className="opacity-0 transition group-hover:opacity-100"
                                                onClick={() => deleteScript(name)}
                                                title="删除"
                                            >
                                                <Trash2 className="h-3.5 w-3.5 text-destructive" />
                                            </Button>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </ScrollArea>
                    </CardContent>
                </Card>

                <Card className="bg-card/80 shadow-none">
                    <CardHeader className="pb-3">
                        <div className="flex items-center justify-between gap-3">
                            <CardTitle className="text-sm uppercase tracking-[0.2em] text-muted-foreground">
                                编辑器
                            </CardTitle>
                            <div className="flex items-center gap-2">
                                <Button
                                    type="button"
                                    size="sm"
                                    onClick={runScript}
                                    disabled={running}
                                    title="运行 (Ctrl+Enter)"
                                >
                                    <Play className="h-3.5 w-3.5" />
                                    运行
                                </Button>
                                <Button
                                    type="button"
                                    size="sm"
                                    variant="secondary"
                                    onClick={evalEnv}
                                    disabled={running}
                                    title="作为环境变量配置求值"
                                >
                                    <Download className="h-3.5 w-3.5" />
                                    环境
                                </Button>
                                <Button
                                    type="button"
                                    size="sm"
                                    variant="outline"
                                    onClick={saveScript}
                                    title="保存脚本"
                                >
                                    <Save className="h-3.5 w-3.5" />
                                </Button>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        <Input
                            value={scriptName}
                            onChange={(e) => setScriptName(e.target.value)}
                            placeholder="脚本名称（保存时使用）"
                            className="h-9 text-xs"
                        />
                        <div className="overflow-hidden rounded-xl border">
                            <Editor
                                height="18rem"
                                defaultLanguage="rust"
                                theme="vs-dark"
                                value={source}
                                onChange={(value) => setSource(value ?? "")}
                                onMount={(editor, monaco) => {
                                    editor.addAction({
                                        id: "run-rhai",
                                        label: "Run Rhai Script",
                                        keybindings: [
                                            monaco.KeyMod.CtrlCmd |
                                                monaco.KeyCode.Enter,
                                        ],
                                        run: () => runScript(),
                                    });
                                }}
                                options={{
                                    fontSize: 13,
                                    fontFamily:
                                        "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
                                    minimap: { enabled: false },
                                    lineNumbers: "on",
                                    scrollBeyondLastLine: false,
                                    wordWrap: "on",
                                    tabSize: 2,
                                    automaticLayout: true,
                                }}
                            />
                        </div>
                    </CardContent>
                </Card>

                <Card className="bg-card/80 shadow-none">
                    <CardHeader className="pb-3">
                        <CardTitle className="text-sm uppercase tracking-[0.2em] text-muted-foreground">
                            输出
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <Tabs
                            value={activeTab}
                            onValueChange={(value) =>
                                setActiveTab(value as "run" | "env")
                            }
                        >
                            <TabsList className="grid w-full grid-cols-2">
                                <TabsTrigger value="run">运行结果</TabsTrigger>
                                <TabsTrigger value="env">环境变量</TabsTrigger>
                            </TabsList>
                            <TabsContent value="run">
                                <ScrollArea className="h-80 rounded-xl border bg-black/90 p-4 font-mono text-sm">
                                    {running ? (
                                        <div className="flex h-full items-center justify-center">
                                            <Loader2 className="h-6 w-6 animate-spin text-zinc-500" />
                                        </div>
                                    ) : error ? (
                                        <div className="whitespace-pre-wrap text-red-400">
                                            {error}
                                        </div>
                                    ) : result ? (
                                        <div className="space-y-4">
                                            {result.stdout ? (
                                                <div>
                                                    <div className="mb-1 text-xs text-zinc-500">
                                                        stdout
                                                    </div>
                                                    <div className="whitespace-pre-wrap text-emerald-200">
                                                        {result.stdout}
                                                    </div>
                                                </div>
                                            ) : null}
                                            {result.stderr ? (
                                                <div>
                                                    <div className="mb-1 text-xs text-zinc-500">
                                                        stderr
                                                    </div>
                                                    <div className="whitespace-pre-wrap text-amber-200">
                                                        {result.stderr}
                                                    </div>
                                                </div>
                                            ) : null}
                                            <div>
                                                <div className="mb-1 text-xs text-zinc-500">
                                                    返回值
                                                </div>
                                                <div className="text-white">
                                                    {JSON.stringify(
                                                        result.vars._result,
                                                        null,
                                                        2,
                                                    )}
                                                </div>
                                            </div>
                                            <div className="flex gap-4 text-xs text-zinc-500">
                                                <span>exit: {result.exit_code}</span>
                                                <span>{result.duration_ms}ms</span>
                                            </div>
                                        </div>
                                    ) : (
                                        <div className="flex h-full items-center justify-center text-zinc-600">
                                            <div className="text-center">
                                                <Terminal className="mx-auto mb-2 h-8 w-8" />
                                                <p>点击“运行”或按 Ctrl+Enter</p>
                                            </div>
                                        </div>
                                    )}
                                </ScrollArea>
                            </TabsContent>
                            <TabsContent value="env">
                                <ScrollArea className="h-80 rounded-xl border bg-black/90 p-4 font-mono text-sm">
                                    {running ? (
                                        <div className="flex h-full items-center justify-center">
                                            <Loader2 className="h-6 w-6 animate-spin text-zinc-500" />
                                        </div>
                                    ) : error ? (
                                        <div className="whitespace-pre-wrap text-red-400">
                                            {error}
                                        </div>
                                    ) : envResult ? (
                                        <div className="space-y-3">
                                            {Object.keys(envResult.vars).length === 0 ? (
                                                <p className="text-zinc-500">无导出变量</p>
                                            ) : (
                                                Object.entries(envResult.vars).map(
                                                    ([key, value]) => (
                                                        <div key={key}>
                                                            <div className="text-xs text-emerald-300">
                                                                {key}
                                                            </div>
                                                            <div className="text-white">
                                                                {typeof value === "object"
                                                                    ? JSON.stringify(
                                                                          value,
                                                                          null,
                                                                          2,
                                                                      )
                                                                    : String(value)}
                                                            </div>
                                                        </div>
                                                    ),
                                                )
                                            )}
                                        </div>
                                    ) : (
                                        <div className="flex h-full items-center justify-center text-zinc-600">
                                            <div className="text-center">
                                                <Terminal className="mx-auto mb-2 h-8 w-8" />
                                                <p>点击“环境”生成导出变量</p>
                                            </div>
                                        </div>
                                    )}
                                </ScrollArea>
                            </TabsContent>
                        </Tabs>
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}
