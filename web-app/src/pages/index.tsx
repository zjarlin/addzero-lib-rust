import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Brain, FolderOpen, Terminal, Zap, Loader2 } from "lucide-react";
import { getApiBaseUrl } from "@addzero/api-client";

export default function Dashboard() {
    const navigate = useNavigate();
    const [skillsCount, setSkillsCount] = useState<number | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const baseUrl = getApiBaseUrl();
        fetch(`${baseUrl}/api/skills`, { credentials: "include" })
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json();
            })
            .then((data: unknown[]) => setSkillsCount(data.length))
            .catch((err) => setError(err.message));
    }, []);

    return (
        <div className="space-y-6">
            <div>
                <h1 className="text-3xl font-bold tracking-tight text-white">
                    AIO Platform
                </h1>
                <p className="mt-1 text-zinc-400">
                    Web + 桌面双端一体化脚本化运行平台
                </p>
            </div>

            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <StatCard
                    icon={<Brain className="h-5 w-5" />}
                    label="Skills"
                    value={
                        error ? (
                            "—"
                        ) : skillsCount === null ? (
                            <Loader2 className="h-5 w-5 animate-spin" />
                        ) : (
                            String(skillsCount)
                        )
                    }
                    detail="已加载技能"
                />
                <StatCard
                    icon={<Terminal className="h-5 w-5" />}
                    label="脚本引擎"
                    value="1"
                    detail="Rhai 已就绪"
                />
                <StatCard
                    icon={<Zap className="h-5 w-5" />}
                    label="插件运行时"
                    value="✓"
                    detail="WASM 插件就绪"
                />
                <StatCard
                    icon={<FolderOpen className="h-5 w-5" />}
                    label="资源"
                    value="—"
                    detail="已部署资源"
                />
            </div>

            <div className="rounded-xl border border-white/10 bg-white/5 p-6">
                <h3 className="text-lg font-semibold text-white">快速入口</h3>
                <div className="mt-4 grid gap-4 sm:grid-cols-2">
                    <QuickAction
                        icon={<Terminal className="h-5 w-5" />}
                        title="脚本控制台"
                        detail="在线编写和运行 Rhai 脚本"
                        onClick={() => navigate("/console")}
                    />
                    <QuickAction
                        icon={<Brain className="h-5 w-5" />}
                        title="管理 Skills"
                        detail="查看和管理技能定义"
                        onClick={() => navigate("/skills")}
                    />
                </div>
            </div>
        </div>
    );
}

function StatCard({
    icon,
    label,
    value,
    detail,
}: {
    icon: React.ReactNode;
    label: string;
    value: React.ReactNode;
    detail: string;
}) {
    return (
        <div className="rounded-xl border border-white/10 bg-white/5 p-5">
            <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium text-zinc-400">{label}</h3>
                <span className="text-zinc-500">{icon}</span>
            </div>
            <div className="mt-2 text-2xl font-bold text-white">{value}</div>
            <p className="mt-1 text-xs text-zinc-500">{detail}</p>
        </div>
    );
}

function QuickAction({
    icon,
    title,
    detail,
    onClick,
}: {
    icon: React.ReactNode;
    title: string;
    detail: string;
    onClick?: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="flex items-center gap-4 rounded-lg border border-white/10 bg-white/5 p-4 text-left transition hover:border-white/20 hover:bg-white/10"
        >
            <span className="text-zinc-400">{icon}</span>
            <div>
                <div className="font-medium text-white">{title}</div>
                <div className="text-sm text-zinc-500">{detail}</div>
            </div>
        </button>
    );
}
