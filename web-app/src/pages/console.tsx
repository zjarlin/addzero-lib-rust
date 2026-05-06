import { useCallback, useEffect, useState } from "react";
import { Play, Terminal, Loader2, Save, Download, Trash2 } from "lucide-react";
import Editor from "@monaco-editor/react";
import { getApiBaseUrl } from "@addzero/api-client";

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

    return (
        <div className="space-y-6">
            <div>
                <h1 className="text-3xl font-bold tracking-tight text-white">
                    脚本控制台
                </h1>
                <p className="mt-1 text-zinc-400">
                    在线编写、保存和运行 Rhai 脚本 · 支持环境变量配置
                </p>
            </div>

            <div className="grid gap-6 lg:grid-cols-[16rem_1fr_1fr]">
                {/* Script List */}
                <div className="space-y-3">
                    <h2 className="text-sm font-semibold uppercase tracking-[0.2em] text-zinc-500">
                        已保存
                    </h2>
                    <div className="max-h-80 overflow-auto rounded-xl border border-white/10 bg-black/20 p-2">
                        {savedScripts.length === 0 ? (
                            <p className="p-3 text-xs text-zinc-600">
                                暂无脚本
                            </p>
                        ) : (
                            <div className="space-y-1">
                                {savedScripts.map((name) => (
                                    <div
                                        key={name}
                                        className="group flex items-center justify-between rounded-lg px-3 py-2 text-sm transition hover:bg-white/5"
                                    >
                                        <button
                                            type="button"
                                            className="flex-1 text-left text-zinc-300"
                                            onClick={() => loadScript(name)}
                                        >
                                            {name}.rhai
                                        </button>
                                        <button
                                            type="button"
                                            className="opacity-0 transition group-hover:opacity-100"
                                            onClick={() => deleteScript(name)}
                                            title="删除"
                                        >
                                            <Trash2 className="h-3.5 w-3.5 text-red-400" />
                                        </button>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* Editor */}
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <h2 className="text-sm font-semibold uppercase tracking-[0.2em] text-zinc-500">
                            编辑器
                        </h2>
                        <div className="flex items-center gap-1">
                            <button
                                type="button"
                                onClick={runScript}
                                disabled={running}
                                className="inline-flex items-center gap-1.5 rounded-md border border-emerald-400/60 bg-emerald-500/10 px-3 py-1.5 text-xs text-emerald-100 transition hover:bg-emerald-500/20 disabled:opacity-50"
                                title="运行 (Ctrl+Enter)"
                            >
                                <Play className="h-3.5 w-3.5" />
                                运行
                            </button>
                            <button
                                type="button"
                                onClick={evalEnv}
                                disabled={running}
                                className="inline-flex items-center gap-1.5 rounded-md border border-cyan-400/60 bg-cyan-500/10 px-3 py-1.5 text-xs text-cyan-100 transition hover:bg-cyan-500/20 disabled:opacity-50"
                                title="作为环境变量配置求值"
                            >
                                <Download className="h-3.5 w-3.5" />
                                环境
                            </button>
                            <button
                                type="button"
                                onClick={saveScript}
                                className="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-zinc-300 transition hover:bg-white/10"
                                title="保存脚本"
                            >
                                <Save className="h-3.5 w-3.5" />
                            </button>
                        </div>
                    </div>
                    <input
                        type="text"
                        value={scriptName}
                        onChange={(e) => setScriptName(e.target.value)}
                        placeholder="脚本名称（保存时使用）"
                        className="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-zinc-300 placeholder-zinc-600 focus:border-emerald-400/40 focus:outline-none"
                    />
                    <div className="rounded-xl border border-white/10">
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
                </div>

                {/* Output */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => setActiveTab("run")}
                            className={`rounded-md px-3 py-1 text-xs transition ${
                                activeTab === "run"
                                    ? "bg-emerald-500/10 text-emerald-200"
                                    : "text-zinc-500 hover:text-zinc-300"
                            }`}
                        >
                            运行结果
                        </button>
                        <button
                            type="button"
                            onClick={() => setActiveTab("env")}
                            className={`rounded-md px-3 py-1 text-xs transition ${
                                activeTab === "env"
                                    ? "bg-cyan-500/10 text-cyan-200"
                                    : "text-zinc-500 hover:text-zinc-300"
                            }`}
                        >
                            环境变量
                        </button>
                    </div>
                    <div className="h-80 overflow-auto rounded-xl border border-white/10 bg-black/30 p-4 font-mono text-sm">
                        {running ? (
                            <div className="flex h-full items-center justify-center">
                                <Loader2 className="h-6 w-6 animate-spin text-zinc-500" />
                            </div>
                        ) : error ? (
                            <div className="text-red-400 whitespace-pre-wrap">
                                {error}
                            </div>
                        ) : activeTab === "run" && result ? (
                            <div className="space-y-4">
                                {result.stdout && (
                                    <div>
                                        <div className="mb-1 text-xs text-zinc-500">
                                            stdout
                                        </div>
                                        <div className="text-emerald-200 whitespace-pre-wrap">
                                            {result.stdout}
                                        </div>
                                    </div>
                                )}
                                {result.stderr && (
                                    <div>
                                        <div className="mb-1 text-xs text-zinc-500">
                                            stderr
                                        </div>
                                        <div className="text-amber-200 whitespace-pre-wrap">
                                            {result.stderr}
                                        </div>
                                    </div>
                                )}
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
                        ) : activeTab === "env" && envResult ? (
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
                                    <p>点击"运行"或按 Ctrl+Enter</p>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
