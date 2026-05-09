import { useEffect, useMemo, useState } from "react";
import {
    Blocks,
    Bot,
    BrainCircuit,
    Cable,
    Database,
    Loader2,
    MonitorSmartphone,
    PlaySquare,
    TerminalSquare,
    Workflow,
} from "lucide-react";
import { getApiBaseUrl } from "@az/api-client";

type RuntimeStatus = "online" | "pending" | "offline";

interface CapabilityCardProps {
    title: string;
    detail: string;
    status: RuntimeStatus;
    icon: React.ReactNode;
}

export default function Dashboard() {
    const [skillsCount, setSkillsCount] = useState<number | null>(null);
    const [apiReady, setApiReady] = useState<boolean | null>(null);

    useEffect(() => {
        const baseUrl = getApiBaseUrl();

        fetch(`${baseUrl}/api/admin/session`, { credentials: "include" })
            .then((res) => setApiReady(res.ok))
            .catch(() => setApiReady(false));

        fetch(`${baseUrl}/api/skills`, { credentials: "include" })
            .then((res) => {
                if (!res.ok) {
                    throw new Error(`HTTP ${res.status}`);
                }
                return res.json();
            })
            .then((data: unknown[]) => setSkillsCount(data.length))
            .catch(() => setSkillsCount(0));
    }, []);

    const capabilityCards = useMemo(
        () => [
            {
                title: "多脚本运行时",
                detail: "统一托管 Rhai、Python、TypeScript、Bash",
                status: "pending" as RuntimeStatus,
                icon: <TerminalSquare className="h-4 w-4" />,
            },
            {
                title: "Vibe Coding",
                detail: "自然语言生成脚本、流程和工程模板",
                status: "pending" as RuntimeStatus,
                icon: <BrainCircuit className="h-4 w-4" />,
            },
            {
                title: "WASM 插件",
                detail: "面向热插拔与隔离的插件运行层",
                status: "pending" as RuntimeStatus,
                icon: <Blocks className="h-4 w-4" />,
            },
            {
                title: "双端外壳",
                detail: "Web + Tauri 共享同一工作台与后端内核",
                status:
                    apiReady === null
                        ? ("pending" as RuntimeStatus)
                        : apiReady
                          ? ("online" as RuntimeStatus)
                          : ("offline" as RuntimeStatus),
                icon: <MonitorSmartphone className="h-4 w-4" />,
            },
        ],
        [apiReady],
    );

    return (
        <div className="space-y-8">
            <section className="grid gap-6 xl:grid-cols-[1.5fr_0.9fr]">
                <div className="overflow-hidden rounded-lg border bg-card">
                    <div className="border-b px-6 py-5">
                        <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                            <Cable className="h-3.5 w-3.5" />
                            AIO Workbench
                        </div>
                        <h1 className="mt-3 text-3xl font-semibold tracking-tight">
                            Web + Desktop unified shell for scripted runtime and AI orchestration
                        </h1>
                        <p className="mt-3 max-w-3xl text-sm text-muted-foreground">
                            这里先收敛平台外壳，不把它误做成 IDE 或普通后台。当前目标是把工作台骨架、
                            模块边界、运行入口和后续扩展位固定下来。
                        </p>
                    </div>

                    <div className="grid gap-0 md:grid-cols-3">
                        <SignalBlock
                            label="API 状态"
                            value={
                                apiReady === null ? (
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                ) : apiReady ? (
                                    "Ready"
                                ) : (
                                    "Offline"
                                )
                            }
                            detail="Axum platform gateway"
                        />
                        <SignalBlock
                            label="插件与技能"
                            value={
                                skillsCount === null ? (
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                ) : (
                                    String(skillsCount)
                                )
                            }
                            detail="已装载能力定义"
                        />
                        <SignalBlock
                            label="当前阶段"
                            value="Shell"
                            detail="先完成双端平台骨架"
                        />
                    </div>
                </div>

                <div className="rounded-lg border bg-card p-5">
                    <div className="flex items-center gap-2 text-sm font-medium">
                        <Workflow className="h-4 w-4 text-muted-foreground" />
                        平台主线
                    </div>
                    <div className="mt-4 space-y-3">
                        <RoadmapStep
                            title="1. 工作台壳子"
                            detail="统一导航、登录入口、平台总览、一级能力域"
                            active
                        />
                        <RoadmapStep
                            title="2. 运行时落位"
                            detail="脚本执行、任务编排、插件生命周期和权限边界"
                        />
                        <RoadmapStep
                            title="3. AI 编排"
                            detail="接入对话、生成脚本、生成流程、生成 CLI 模板"
                        />
                        <RoadmapStep
                            title="4. 产出层"
                            detail="导出脚本工程、CLI 脚手架、模板项目"
                        />
                    </div>
                </div>
            </section>

            <section className="grid gap-4 lg:grid-cols-2 2xl:grid-cols-4">
                {capabilityCards.map((item) => (
                    <CapabilityCard key={item.title} {...item} />
                ))}
            </section>

            <section className="grid gap-6 xl:grid-cols-[1.15fr_0.85fr]">
                <div className="rounded-lg border bg-card">
                    <div className="border-b px-5 py-4">
                        <h2 className="text-base font-semibold">能力域</h2>
                        <p className="mt-1 text-sm text-muted-foreground">
                            一级导航按平台能力拆分，不按零散页面堆叠。
                        </p>
                    </div>
                    <div className="grid gap-0 md:grid-cols-2">
                        <DomainTile
                            icon={<PlaySquare className="h-4 w-4" />}
                            title="脚本引擎"
                            detail="Rhai / Python / TS / Bash 统一调度"
                        />
                        <DomainTile
                            icon={<Bot className="h-4 w-4" />}
                            title="AI 编排"
                            detail="Vibe Coding、提示词、脚本生成、任务生成"
                        />
                        <DomainTile
                            icon={<Blocks className="h-4 w-4" />}
                            title="插件系统"
                            detail="WASM 组件、热加载、扩展点、隔离运行"
                        />
                        <DomainTile
                            icon={<Database className="h-4 w-4" />}
                            title="资源与知识"
                            detail="知识、配置、对象存储、市场与系统元数据"
                        />
                    </div>
                </div>

                <div className="rounded-lg border bg-card p-5">
                    <h2 className="text-base font-semibold">当前壳子约束</h2>
                    <div className="mt-4 space-y-3 text-sm text-muted-foreground">
                        <ConstraintItem text="不是 IDE，首屏不围绕代码编辑器，而是围绕平台工作台。" />
                        <ConstraintItem text="桌面端只是 Tauri 外壳，核心能力必须仍然由统一后端提供。" />
                        <ConstraintItem text="前端继续走 React + 主流组件生态，不额外引入小众 UI 体系。" />
                        <ConstraintItem text="PostgreSQL、插件、AI、任务流都作为能力域接入，不在壳子层硬编码业务。" />
                    </div>
                </div>
            </section>
        </div>
    );
}

function CapabilityCard({ title, detail, status, icon }: CapabilityCardProps) {
    const statusLabel =
        status === "online" ? "Ready" : status === "pending" ? "Planned" : "Blocked";
    const statusClass =
        status === "online"
            ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
            : status === "pending"
              ? "bg-amber-500/10 text-amber-600 dark:text-amber-400"
              : "bg-rose-500/10 text-rose-600 dark:text-rose-400";

    return (
        <div className="rounded-lg border bg-card p-5">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-medium">
                    <span className="text-muted-foreground">{icon}</span>
                    {title}
                </div>
                <span className={`rounded-md px-2 py-1 text-[11px] font-medium ${statusClass}`}>
                    {statusLabel}
                </span>
            </div>
            <p className="mt-3 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}

function SignalBlock({
    label,
    value,
    detail,
}: {
    label: string;
    value: React.ReactNode;
    detail: string;
}) {
    return (
        <div className="border-t px-6 py-4 md:border-l md:first:border-l-0 md:border-t-0">
            <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                {label}
            </div>
            <div className="mt-2 flex min-h-7 items-center text-2xl font-semibold">{value}</div>
            <div className="mt-1 text-sm text-muted-foreground">{detail}</div>
        </div>
    );
}

function RoadmapStep({
    title,
    detail,
    active = false,
}: {
    title: string;
    detail: string;
    active?: boolean;
}) {
    return (
        <div className="rounded-lg border px-4 py-3">
            <div className="flex items-center justify-between gap-3">
                <div className="text-sm font-medium">{title}</div>
                <span
                    className={`rounded-md px-2 py-1 text-[11px] font-medium ${
                        active
                            ? "bg-primary text-primary-foreground"
                            : "bg-muted text-muted-foreground"
                    }`}
                >
                    {active ? "Current" : "Queued"}
                </span>
            </div>
            <p className="mt-1 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}

function DomainTile({
    icon,
    title,
    detail,
}: {
    icon: React.ReactNode;
    title: string;
    detail: string;
}) {
    return (
        <div className="border-t px-5 py-4 odd:md:border-r first:border-t-0 nth-[2]:border-t-0">
            <div className="flex items-center gap-2 text-sm font-medium">
                <span className="text-muted-foreground">{icon}</span>
                {title}
            </div>
            <p className="mt-2 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}

function ConstraintItem({ text }: { text: string }) {
    return (
        <div className="rounded-md border bg-muted/30 px-3 py-2">
            {text}
        </div>
    );
}
