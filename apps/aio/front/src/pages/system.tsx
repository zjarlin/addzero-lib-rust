import { useEffect, useMemo, useState } from "react";
import {
    ArrowRight,
    BookOpen,
    Bot,
    Building2,
    CheckCircle2,
    ClipboardList,
    LockKeyhole,
    Menu,
    RotateCw,
    Search,
    Shield,
    Sparkles,
    Users,
} from "lucide-react";
import {
    Badge,
    Button,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
    Input,
    Textarea,
    cn,
} from "@addzero/ui";
import { emitAiWorkspaceSeed } from "../lib/ai-workspace";

type ModuleId =
    | "users"
    | "roles"
    | "navigation"
    | "organization"
    | "metadata"
    | "security";

interface SystemModule {
    id: ModuleId;
    title: string;
    desc: string;
    scope: string;
    icon: typeof Shield;
    checklist: string[];
    suggestedPrompt: string;
}

interface ModuleWorkbenchState {
    notes: string;
    checked: string[];
    completed: boolean;
}

interface SystemWorkbenchState {
    activeModuleId: ModuleId;
    search: string;
    modules: Record<ModuleId, ModuleWorkbenchState>;
}

const STORAGE_KEY = "aio.system.workbench.state";

const modules: SystemModule[] = [
    {
        id: "users",
        icon: Users,
        title: "用户管理",
        desc: "管理员、操作者、服务身份与会话边界。",
        scope: "账号生命周期、登录态、服务身份、审计主体",
        checklist: [
            "梳理用户类型和最小字段",
            "定义会话失效与恢复规则",
            "明确服务身份的权限边界",
        ],
        suggestedPrompt: "把用户管理模块整理成角色边界、状态流转和风险点。",
    },
    {
        id: "roles",
        icon: Shield,
        title: "角色管理",
        desc: "能力授权、资源访问和模块级权限模型。",
        scope: "角色模板、能力矩阵、页面访问、动作级授权",
        checklist: [
            "列出角色模板和职责边界",
            "梳理资源访问矩阵",
            "补齐动作级权限与审计要求",
        ],
        suggestedPrompt: "把角色管理整理成权限矩阵和需要补的边界条件。",
    },
    {
        id: "navigation",
        icon: Menu,
        title: "导航管理",
        desc: "平台工作台的信息架构与菜单树。",
        scope: "主轴上下文树、侧轴菜单树、可见性规则、排序",
        checklist: [
            "梳理主轴 domain 划分",
            "确认左侧菜单树的层级关系",
            "补齐隐藏、排序和权限规则",
        ],
        suggestedPrompt: "把导航管理整理成二维上下文树和菜单治理规则。",
    },
    {
        id: "organization",
        icon: Building2,
        title: "组织上下文",
        desc: "团队、空间、环境或租户范围。",
        scope: "组织层级、空间隔离、环境范围、租户策略",
        checklist: [
            "明确组织层级和上下文切换",
            "梳理租户/环境隔离边界",
            "补齐共享与继承规则",
        ],
        suggestedPrompt: "把组织上下文整理成层级结构、隔离规则和切换路径。",
    },
    {
        id: "metadata",
        icon: BookOpen,
        title: "字典与元数据",
        desc: "状态枚举、模板类型、运行时标签。",
        scope: "状态字典、模板分类、标签体系、运行元数据",
        checklist: [
            "整理统一字典和枚举来源",
            "定义模板和标签约束",
            "补齐运行时元数据归档方式",
        ],
        suggestedPrompt: "把字典与元数据整理成枚举、标签和运行期元信息结构。",
    },
    {
        id: "security",
        icon: LockKeyhole,
        title: "安全边界",
        desc: "脚本权限、外部调用和凭证策略。",
        scope: "脚本能力边界、外部调用审批、密钥托管、审计轨迹",
        checklist: [
            "梳理脚本权限等级",
            "定义外部调用审批链路",
            "补齐凭证保管和审计要求",
        ],
        suggestedPrompt: "把安全边界整理成权限等级、凭证策略和审计要求。",
    },
];

function createModuleState(): ModuleWorkbenchState {
    return {
        notes: "",
        checked: [],
        completed: false,
    };
}

function createDefaultState(): SystemWorkbenchState {
    return {
        activeModuleId: modules[0].id,
        search: "",
        modules: {
            users: createModuleState(),
            roles: createModuleState(),
            navigation: createModuleState(),
            organization: createModuleState(),
            metadata: createModuleState(),
            security: createModuleState(),
        },
    };
}

function readState(): SystemWorkbenchState {
    if (typeof window === "undefined") {
        return createDefaultState();
    }

    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) {
            return createDefaultState();
        }

        const parsed = JSON.parse(raw) as Partial<SystemWorkbenchState>;
        const defaults = createDefaultState();
        return {
            activeModuleId:
                parsed.activeModuleId && defaults.modules[parsed.activeModuleId]
                    ? parsed.activeModuleId
                    : defaults.activeModuleId,
            search: parsed.search ?? "",
            modules: {
                users: {
                    ...defaults.modules.users,
                    ...(parsed.modules?.users ?? {}),
                },
                roles: {
                    ...defaults.modules.roles,
                    ...(parsed.modules?.roles ?? {}),
                },
                navigation: {
                    ...defaults.modules.navigation,
                    ...(parsed.modules?.navigation ?? {}),
                },
                organization: {
                    ...defaults.modules.organization,
                    ...(parsed.modules?.organization ?? {}),
                },
                metadata: {
                    ...defaults.modules.metadata,
                    ...(parsed.modules?.metadata ?? {}),
                },
                security: {
                    ...defaults.modules.security,
                    ...(parsed.modules?.security ?? {}),
                },
            },
        };
    } catch {
        return createDefaultState();
    }
}

function moduleSnapshot(module: SystemModule, state: ModuleWorkbenchState) {
    const completedChecklist = module.checklist.map((item) => {
        const checked = state.checked.includes(item) ? "已完成" : "待处理";
        return `- ${item}：${checked}`;
    });

    return [
        `${module.title}`,
        `范围：${module.scope}`,
        `说明：${module.desc}`,
        `模块状态：${state.completed ? "已完成" : "进行中"}`,
        "检查项：",
        ...completedChecklist,
        `备注：${state.notes.trim() || "无"}`,
    ].join("\n");
}

function systemOverview(state: SystemWorkbenchState) {
    return modules
        .map((module) => moduleSnapshot(module, state.modules[module.id]))
        .join("\n\n---\n\n");
}

async function copyText(text: string) {
    if (typeof navigator === "undefined" || !navigator.clipboard?.writeText) {
        throw new Error("当前环境不支持剪贴板写入");
    }
    await navigator.clipboard.writeText(text);
}

export default function SystemPage() {
    const [workbench, setWorkbench] = useState<SystemWorkbenchState>(readState);
    const [feedback, setFeedback] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(workbench));
    }, [workbench]);

    const filteredModules = useMemo(() => {
        const query = workbench.search.trim().toLowerCase();
        if (!query) {
            return modules;
        }
        return modules.filter((module) =>
            [module.title, module.desc, module.scope].some((part) =>
                part.toLowerCase().includes(query),
            ),
        );
    }, [workbench.search]);

    const activeModule =
        modules.find((module) => module.id === workbench.activeModuleId) ?? modules[0];
    const activeState = workbench.modules[activeModule.id];

    const completedCount = modules.filter(
        (module) => workbench.modules[module.id].completed,
    ).length;
    const checkedCount = modules.reduce(
        (count, module) => count + workbench.modules[module.id].checked.length,
        0,
    );
    const notesCount = modules.filter((module) =>
        workbench.modules[module.id].notes.trim(),
    ).length;

    function patchModule(
        moduleId: ModuleId,
        updater: (current: ModuleWorkbenchState) => ModuleWorkbenchState,
    ) {
        setWorkbench((current) => ({
            ...current,
            modules: {
                ...current.modules,
                [moduleId]: updater(current.modules[moduleId]),
            },
        }));
    }

    function selectModule(moduleId: ModuleId) {
        setWorkbench((current) => ({
            ...current,
            activeModuleId: moduleId,
        }));
        setFeedback(null);
        setError(null);
    }

    function toggleChecklist(item: string) {
        patchModule(activeModule.id, (current) => {
            const checked = current.checked.includes(item)
                ? current.checked.filter((entry) => entry !== item)
                : [...current.checked, item];
            return {
                ...current,
                checked,
            };
        });
    }

    function cycleModule() {
        const pool = filteredModules.length > 0 ? filteredModules : modules;
        const currentIndex = pool.findIndex(
            (module) => module.id === workbench.activeModuleId,
        );
        const nextIndex = currentIndex >= 0 ? (currentIndex + 1) % pool.length : 0;
        selectModule(pool[nextIndex].id);
    }

    async function handleCopyCurrent() {
        try {
            await copyText(moduleSnapshot(activeModule, activeState));
            setFeedback(`已复制 ${activeModule.title} 摘要`);
            setError(null);
        } catch (err) {
            setError(err instanceof Error ? err.message : "复制失败");
        }
    }

    async function handleCopyOverview() {
        try {
            await copyText(systemOverview(workbench));
            setFeedback("已复制系统工作台总览");
            setError(null);
        } catch (err) {
            setError(err instanceof Error ? err.message : "复制失败");
        }
    }

    function sendModuleToAi(module: SystemModule) {
        const state = workbench.modules[module.id];
        emitAiWorkspaceSeed({
            path: "/system",
            material: moduleSnapshot(module, state),
            prompt: module.suggestedPrompt,
            open: true,
        });
        setFeedback(`已把 ${module.title} 送到 AI 整理窗`);
        setError(null);
    }

    function sendOverviewToAi() {
        emitAiWorkspaceSeed({
            path: "/system",
            material: systemOverview(workbench),
            prompt: "把系统工作台这些模块整理成优先级、风险和下一步动作。",
            open: true,
        });
        setFeedback("已把系统总览送到 AI 整理窗");
        setError(null);
    }

    function toggleCompleted(moduleId: ModuleId) {
        patchModule(moduleId, (current) => ({
            ...current,
            completed: !current.completed,
        }));
    }

    function resetActiveModule() {
        patchModule(activeModule.id, () => createModuleState());
        setFeedback(`已重置 ${activeModule.title} 的本地工作状态`);
        setError(null);
    }

    return (
        <div className="space-y-6">
            <Card className="overflow-hidden">
                <CardHeader className="border-b bg-[#faf7ef]">
                    <div className="flex flex-wrap items-center justify-between gap-3">
                        <div>
                            <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                                <Shield className="h-3.5 w-3.5" />
                                System Workbench
                            </div>
                            <CardTitle className="mt-3 text-3xl tracking-tight">
                                系统工作台
                            </CardTitle>
                            <CardDescription className="mt-2 max-w-3xl text-sm">
                                这页现在不是静态说明。你可以在这里切模块、打检查项、写备注、
                                复制清单，或者直接把当前模块和总览送进 AI 整理窗。
                            </CardDescription>
                        </div>
                        <div className="flex flex-wrap gap-2">
                            <Button type="button" variant="outline" onClick={cycleModule}>
                                <ArrowRight className="h-4 w-4" />
                                下一模块
                            </Button>
                            <Button type="button" variant="outline" onClick={() => void handleCopyOverview()}>
                                <ClipboardList className="h-4 w-4" />
                                复制总览
                            </Button>
                            <Button type="button" onClick={sendOverviewToAi}>
                                <Bot className="h-4 w-4" />
                                总览交给 AI
                            </Button>
                        </div>
                    </div>
                </CardHeader>

                <CardContent className="grid gap-3 p-5 sm:grid-cols-2 xl:grid-cols-4">
                    <TopMetric
                        label="模块数"
                        value={String(modules.length)}
                        detail="系统治理面"
                    />
                    <TopMetric
                        label="已完成"
                        value={String(completedCount)}
                        detail="模块状态"
                    />
                    <TopMetric
                        label="已勾选"
                        value={String(checkedCount)}
                        detail="检查项"
                    />
                    <TopMetric
                        label="有备注"
                        value={String(notesCount)}
                        detail="本地整理"
                    />
                </CardContent>
            </Card>

            {feedback ? (
                <div className="rounded-2xl border border-emerald-300 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                    {feedback}
                </div>
            ) : null}
            {error ? (
                <div className="rounded-2xl border border-rose-300 bg-rose-50 px-4 py-3 text-sm text-rose-700">
                    {error}
                </div>
            ) : null}

            <section className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
                <Card>
                    <CardHeader className="border-b">
                        <div className="flex flex-wrap items-center justify-between gap-3">
                            <div>
                                <CardTitle className="text-lg">治理模块</CardTitle>
                                <CardDescription className="mt-1">
                                    左边选模块，右边直接整理工作状态。
                                </CardDescription>
                            </div>
                            <div className="relative w-full max-w-xs">
                                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                                <Input
                                    value={workbench.search}
                                    onChange={(event) =>
                                        setWorkbench((current) => ({
                                            ...current,
                                            search: event.target.value,
                                        }))
                                    }
                                    className="pl-9"
                                    placeholder="搜索模块、范围或说明"
                                />
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="grid gap-3 p-5 sm:grid-cols-2">
                        {filteredModules.length === 0 ? (
                            <div className="col-span-full rounded-3xl border border-dashed px-4 py-8 text-center text-sm text-muted-foreground">
                                当前筛选没有命中模块，清空关键字后可恢复完整列表。
                            </div>
                        ) : null}
                        {filteredModules.map((module) => {
                            const state = workbench.modules[module.id];
                            const active = module.id === activeModule.id;
                            const Icon = module.icon;
                            return (
                                <Card
                                    key={module.id}
                                    className={cn(
                                        "border transition-colors",
                                        active && "border-stone-900 bg-stone-950 text-stone-50",
                                    )}
                                >
                                    <CardContent className="space-y-4 px-4 py-4">
                                        <div className="flex items-start justify-between gap-3">
                                            <div>
                                                <div className="flex items-center gap-2 text-sm font-medium">
                                                    <Icon
                                                        className={cn(
                                                            "h-4 w-4",
                                                            active
                                                                ? "text-amber-300"
                                                                : "text-muted-foreground",
                                                        )}
                                                    />
                                                    {module.title}
                                                </div>
                                                <p
                                                    className={cn(
                                                        "mt-2 text-sm leading-6",
                                                        active
                                                            ? "text-stone-300"
                                                            : "text-muted-foreground",
                                                    )}
                                                >
                                                    {module.desc}
                                                </p>
                                            </div>
                                            <Badge
                                                variant={state.completed ? "default" : "outline"}
                                                className="rounded-full"
                                            >
                                                {state.completed ? "已完成" : "进行中"}
                                            </Badge>
                                        </div>

                                        <div
                                            className={cn(
                                                "rounded-2xl border px-3 py-3 text-xs leading-6",
                                                active
                                                    ? "border-stone-700 bg-stone-900 text-stone-300"
                                                    : "bg-muted/30 text-muted-foreground",
                                            )}
                                        >
                                            {module.scope}
                                        </div>

                                        <div className="flex flex-wrap gap-2">
                                            <Button
                                                type="button"
                                                size="sm"
                                                variant={active ? "secondary" : "outline"}
                                                onClick={() => selectModule(module.id)}
                                            >
                                                查看详情
                                            </Button>
                                            <Button
                                                type="button"
                                                size="sm"
                                                variant="outline"
                                                onClick={() => sendModuleToAi(module)}
                                            >
                                                <Sparkles className="h-4 w-4" />
                                                AI 整理
                                            </Button>
                                            <Button
                                                type="button"
                                                size="sm"
                                                variant="outline"
                                                onClick={() => toggleCompleted(module.id)}
                                            >
                                                <CheckCircle2 className="h-4 w-4" />
                                                {state.completed ? "撤销完成" : "标记完成"}
                                            </Button>
                                        </div>
                                    </CardContent>
                                </Card>
                            );
                        })}
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader className="border-b">
                        <div className="flex flex-wrap items-center justify-between gap-3">
                            <div>
                                <CardTitle className="text-lg">{activeModule.title}</CardTitle>
                                <CardDescription className="mt-1">
                                    {activeModule.scope}
                                </CardDescription>
                            </div>
                            <div className="flex flex-wrap gap-2">
                                <Button type="button" variant="outline" size="sm" onClick={() => void handleCopyCurrent()}>
                                    <ClipboardList className="h-4 w-4" />
                                    复制摘要
                                </Button>
                                <Button type="button" size="sm" onClick={() => sendModuleToAi(activeModule)}>
                                    <Bot className="h-4 w-4" />
                                    交给 AI
                                </Button>
                            </div>
                        </div>
                    </CardHeader>

                    <CardContent className="space-y-5 p-5">
                        <section className="space-y-3">
                            <div className="flex items-center justify-between gap-3">
                                <div className="text-sm font-medium">检查清单</div>
                                <Badge variant="outline" className="rounded-full">
                                    {activeState.checked.length}/{activeModule.checklist.length}
                                </Badge>
                            </div>
                            <div className="grid gap-2">
                                {activeModule.checklist.map((item) => {
                                    const checked = activeState.checked.includes(item);
                                    return (
                                        <button
                                            key={item}
                                            type="button"
                                            onClick={() => toggleChecklist(item)}
                                            className={cn(
                                                "flex items-center justify-between rounded-2xl border px-3 py-3 text-left text-sm transition-colors",
                                                checked
                                                    ? "border-emerald-400 bg-emerald-50 text-emerald-800"
                                                    : "border-border bg-background hover:bg-muted/40",
                                            )}
                                        >
                                            <span>{item}</span>
                                            <Badge
                                                variant={checked ? "default" : "outline"}
                                                className="rounded-full"
                                            >
                                                {checked ? "已勾选" : "待处理"}
                                            </Badge>
                                        </button>
                                    );
                                })}
                            </div>
                        </section>

                        <section className="space-y-3">
                            <div className="flex items-center justify-between gap-3">
                                <div className="text-sm font-medium">模块备注</div>
                                <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    onClick={resetActiveModule}
                                >
                                    <RotateCw className="h-4 w-4" />
                                    重置当前模块
                                </Button>
                            </div>
                            <Textarea
                                value={activeState.notes}
                                onChange={(event) =>
                                    patchModule(activeModule.id, (current) => ({
                                        ...current,
                                        notes: event.target.value,
                                    }))
                                }
                                placeholder="这里写当前模块的判断、边界、风险、临时结论或下一步动作。"
                                className="min-h-40 leading-7"
                            />
                        </section>

                        <section className="rounded-3xl border bg-muted/20 px-4 py-4">
                            <div className="text-sm font-medium">当前模块摘要预览</div>
                            <pre className="mt-3 whitespace-pre-wrap text-sm leading-7 text-muted-foreground">
                                {moduleSnapshot(activeModule, activeState)}
                            </pre>
                        </section>

                        <div className="flex flex-wrap gap-2">
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => toggleCompleted(activeModule.id)}
                            >
                                <CheckCircle2 className="h-4 w-4" />
                                {activeState.completed ? "撤销完成" : "标记本模块完成"}
                            </Button>
                            <Button type="button" variant="outline" onClick={cycleModule}>
                                <ArrowRight className="h-4 w-4" />
                                切到下一模块
                            </Button>
                            <Button type="button" onClick={() => sendModuleToAi(activeModule)}>
                                <Sparkles className="h-4 w-4" />
                                用 AI 继续整理
                            </Button>
                        </div>
                    </CardContent>
                </Card>
            </section>
        </div>
    );
}

function TopMetric({
    label,
    value,
    detail,
}: {
    label: string;
    value: string;
    detail: string;
}) {
    return (
        <div className="rounded-3xl border bg-background px-4 py-4 shadow-sm">
            <div className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {label}
            </div>
            <div className="mt-2 text-3xl font-semibold tracking-tight">{value}</div>
            <div className="mt-1 text-sm text-muted-foreground">{detail}</div>
        </div>
    );
}
