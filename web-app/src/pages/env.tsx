import { useCallback, useEffect, useState } from "react";
import { Download, Loader2, Save } from "lucide-react";
import Editor from "@monaco-editor/react";
import { getApiBaseUrl } from "@addzero/api-client";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Badge } from "../components/ui/badge";

interface EnvVars {
  vars: Record<string, unknown>;
  stdout: string;
  stderr: string;
}

const PLACEHOLDER = `// Rhai 环境变量配置
// 所有顶级 let 绑定都会导出为环境变量

let DB_URL = "postgres://localhost:5432/aio";
let PORT = 8787;
let FEATURES = #{
    ai: true,
    python: false,
    bash: true,
};
let CACHE_TTL_SECS = 3600;
`;

export default function EnvPage() {
  const [source, setSource] = useState(PLACEHOLDER);
  const [configName, setConfigName] = useState("env");
  const [savedConfigs, setSavedConfigs] = useState<string[]>([]);
  const [result, setResult] = useState<EnvVars | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const baseUrl = getApiBaseUrl();

  const fetchConfigs = useCallback(async () => {
    try {
      const res = await fetch(`${baseUrl}/api/scripts`, { credentials: "include" });
      if (res.ok) setSavedConfigs(await res.json());
    } catch { /* ignore */ }
  }, [baseUrl]);

  useEffect(() => { fetchConfigs(); }, [fetchConfigs]);

  async function evalEnv() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${baseUrl}/api/engine/rhai/eval-env`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ source, vars: {} }),
      });
      if (!res.ok) throw new Error(await res.text());
      setResult(await res.json());
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function saveConfig() {
    try {
      await fetch(`${baseUrl}/api/scripts`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: configName.trim() || "env", source }),
      });
      fetchConfigs();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Save failed");
    }
  }

  async function loadConfig(name: string) {
    try {
      const res = await fetch(`${baseUrl}/api/scripts/${encodeURIComponent(name)}`, {
        credentials: "include",
      });
      if (!res.ok) throw new Error("Not found");
      const data = await res.json();
      setSource(data.source);
      setConfigName(data.name);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Load failed");
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">环境变量</h1>
        <p className="mt-1 text-muted-foreground">
          用 Rhai 脚本定义环境变量，求值后导出为键值对
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-[14rem_1fr_1fr]">
        {/* Saved configs */}
        <div className="space-y-3">
          <h2 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
            配置
          </h2>
          <div className="space-y-1">
            {savedConfigs.map((name) => (
              <button
                key={name}
                type="button"
                className="block w-full rounded-lg px-3 py-2 text-left text-sm transition hover:bg-accent"
                onClick={() => loadConfig(name)}
              >
                {name}.rhai
              </button>
            ))}
          </div>
        </div>

        {/* Editor */}
        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <Input
              value={configName}
              onChange={(e) => setConfigName(e.target.value)}
              placeholder="配置名称"
              className="h-8 text-xs"
            />
            <Button size="sm" variant="outline" onClick={saveConfig}>
              <Save className="h-3.5 w-3.5" />
            </Button>
            <Button size="sm" onClick={evalEnv} disabled={loading}>
              {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Download className="h-3.5 w-3.5" />}
              求值
            </Button>
          </div>
          <div className="rounded-xl border overflow-hidden">
            <Editor
              height="20rem"
              defaultLanguage="rust"
              theme="vs-dark"
              value={source}
              onChange={(v) => setSource(v ?? "")}
              options={{
                fontSize: 13,
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on",
                tabSize: 2,
                automaticLayout: true,
              }}
            />
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        {/* Evaluated Vars */}
        <div className="space-y-3">
          <h2 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
            导出变量
          </h2>
          <div className="space-y-2">
            {result ? (
              Object.keys(result.vars).length === 0 ? (
                <p className="text-sm text-muted-foreground">无导出变量</p>
              ) : (
                Object.entries(result.vars).map(([key, value]) => (
                  <Card key={key}>
                    <CardHeader className="pb-2">
                      <CardTitle className="text-sm font-mono text-emerald-400">
                        {key}
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <code className="text-xs break-all">
                        {typeof value === "object"
                          ? JSON.stringify(value, null, 2)
                          : String(value)}
                      </code>
                    </CardContent>
                  </Card>
                ))
              )
            ) : (
              <p className="text-sm text-muted-foreground">
                点击"求值"查看导出变量
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
