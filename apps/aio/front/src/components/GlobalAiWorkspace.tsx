import { useEffect, useMemo, useRef, useState } from "react";
import { matchPath, useLocation, useNavigate } from "react-router-dom";
import { Bot, Loader2, Route, Send, Sparkles, Trash2 } from "lucide-react";
import { getApiBaseUrl } from "@addzero/api-client";
import {
    Badge,
    Button,
    ScrollArea,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
    Sheet,
    SheetContent,
    SheetDescription,
    SheetHeader,
    SheetTitle,
    SheetTrigger,
    Textarea,
} from "@addzero/ui";
import {
    AI_WORKSPACE_SEED_EVENT,
    type AiWorkspaceSeedDetail,
} from "../lib/ai-workspace";

type ChatRole = "user" | "assistant";

interface ChatMessage {
    role: ChatRole;
    content: string;
    createdAt: string;
}

interface StoredThreadState {
    material: string;
    messages: ChatMessage[];
}

interface ChatResponseDto {
    provider: AiProviderKind;
    model: string;
    message: {
        role: string;
        content: string;
    };
}

type AiProviderKind = "open_ai" | "anthropic" | "gemini";

interface AiProviderConfigDto {
    provider: AiProviderKind;
    label: string;
    base_url?: string | null;
    default_model: string;
    enabled: boolean;
    api_key_configured: boolean;
    updated_at?: string | null;
}

interface PageProfile {
    title: string;
    focus: string;
    draftPlaceholder: string;
    promptPlaceholder: string;
    actions: string[];
}

const PANEL_OPEN_KEY = "aio.global-ai-workspace.open";
const THREAD_PREFIX = "aio.global-ai-workspace.thread:";
const PROVIDER_KEY = "aio.global-ai-workspace.provider";

const PAGE_RULES: Array<{
    pattern: string;
    end?: boolean;
    profile: PageProfile;
}> = [
    {
        pattern: "/",
        profile: {
            title: "平台总览",
            focus: "汇总首页信号、异常、待办和跨域推进主线",
            draftPlaceholder:
                "把首页看到的指标、问题、待办或观察贴进来，我帮你整理成结构化结论。",
            promptPlaceholder: "例如：把这些首页信息整理成今天的执行清单。",
            actions: [
                "把当前信息整理成今日待办",
                "按模块归并异常信号",
                "提炼需要推进的三条主线",
            ],
        },
    },
    {
        pattern: "/assets/notes",
        profile: {
            title: "笔记",
            focus: "归并笔记、提炼标签、去重并形成可执行结论",
            draftPlaceholder:
                "贴入原始笔记、会议记录、剪贴内容或零散想法，我会帮你整理成主题结构。",
            promptPlaceholder: "例如：把这些笔记整理成标签、摘要和后续动作。",
            actions: [
                "整理成主题清单和标签",
                "合并重复内容并提炼结论",
                "输出可执行待办和负责人假设",
            ],
        },
    },
    {
        pattern: "/assets/packages",
        profile: {
            title: "安装包",
            focus: "整理发布包、版本状态、安装目标和缺口",
            draftPlaceholder:
                "贴入版本记录、发布备注、包清单或安装反馈，我帮你压成发布视图。",
            promptPlaceholder: "例如：把这些安装包数据整理成发布清单和风险点。",
            actions: [
                "整理成发布清单",
                "按版本和目标机器分组",
                "列出缺失校验和风险项",
            ],
        },
    },
    {
        pattern: "/assets/dotfiles",
        profile: {
            title: "dotfiles",
            focus: "整理配置差异、同步计划、回滚点和机器画像",
            draftPlaceholder:
                "贴入配置 diff、主机差异、同步日志或恢复需求，我帮你整理成变更计划。",
            promptPlaceholder: "例如：把这些配置差异整理成同步步骤和回滚点。",
            actions: [
                "整理成同步计划",
                "按机器归并配置差异",
                "生成恢复和回滚步骤",
            ],
        },
    },
    {
        pattern: "/console",
        profile: {
            title: "脚本控制台",
            focus: "整理运行输出、错误信息和下一步修复动作",
            draftPlaceholder:
                "贴入 stdout、stderr、脚本片段或 vibe 会话输出，我帮你压成排障结论。",
            promptPlaceholder: "例如：根据这些输出帮我定位问题并列出下一步。",
            actions: [
                "归并报错并定位根因",
                "整理运行输出为结论",
                "给出下一步验证命令",
            ],
        },
    },
    {
        pattern: "/env",
        profile: {
            title: "环境与配置",
            focus: "整理配置项、依赖关系、变更顺序和风险提示",
            draftPlaceholder:
                "贴入配置项、环境变量、接口返回或变更需求，我帮你整理成变更方案。",
            promptPlaceholder: "例如：把这些配置整理成可执行的修改步骤。",
            actions: [
                "整理成配置变更步骤",
                "提炼依赖关系和风险",
                "输出验收检查表",
            ],
        },
    },
    {
        pattern: "/market",
        end: false,
        profile: {
            title: "WASM 插件市场",
            focus: "整理插件需求、安装决策、能力差异和验收路径",
            draftPlaceholder:
                "贴入插件说明、安装反馈、能力清单或目标场景，我帮你整理成选型建议。",
            promptPlaceholder: "例如：帮我把这些插件信息整理成选型对比。",
            actions: [
                "整理成插件选型对比",
                "提炼安装和验收步骤",
                "列出能力缺口和后续实现",
            ],
        },
    },
    {
        pattern: "/prototype/wasm-studio",
        profile: {
            title: "WASM Studio 原型",
            focus: "整理低代码画布、组件编排和在线 vibe 生成思路",
            draftPlaceholder:
                "贴入画布需求、组件想法、交互流程或插件设想，我帮你整理成原型结构。",
            promptPlaceholder: "例如：把这个 studio 需求整理成组件树和实现步骤。",
            actions: [
                "整理成组件树和分区结构",
                "提炼画布交互与生成流程",
                "输出原型到正式实现的拆解",
            ],
        },
    },
    {
        pattern: "/system",
        end: false,
        profile: {
            title: "系统设置",
            focus: "整理系统配置、权限边界和后台操作步骤",
            draftPlaceholder:
                "贴入系统设置项、权限规则或后台操作需求，我帮你整理成可执行流程。",
            promptPlaceholder: "例如：把这些系统项整理成配置方案和检查表。",
            actions: [
                "整理成后台操作流程",
                "提炼权限和边界条件",
                "输出变更后的验收清单",
            ],
        },
    },
    {
        pattern: "/apps/:instanceSlug/:pageId",
        profile: {
            title: "插件实例页",
            focus: "整理插件实例数据、页面字段、操作流程和页面问题",
            draftPlaceholder:
                "贴入实例页字段、业务数据、控件反馈或页面目标，我帮你整理成结构化结果。",
            promptPlaceholder: "例如：把这个实例页数据整理成字段结构和操作建议。",
            actions: [
                "整理当前实例页的数据结构",
                "归并页面问题和操作路径",
                "输出给插件作者的修订建议",
            ],
        },
    },
];

function readPanelOpen() {
    if (typeof window === "undefined") {
        return false;
    }
    return window.localStorage.getItem(PANEL_OPEN_KEY) === "1";
}

function readThreadState(key: string): StoredThreadState {
    if (typeof window === "undefined") {
        return { material: "", messages: [] };
    }
    try {
        const raw = window.localStorage.getItem(`${THREAD_PREFIX}${key}`);
        if (!raw) {
            return { material: "", messages: [] };
        }
        const parsed = JSON.parse(raw) as StoredThreadState;
        return {
            material: parsed.material ?? "",
            messages: Array.isArray(parsed.messages) ? parsed.messages : [],
        };
    } catch {
        return { material: "", messages: [] };
    }
}

function writeThreadState(key: string, state: StoredThreadState) {
    if (typeof window === "undefined") {
        return;
    }
    window.localStorage.setItem(
        `${THREAD_PREFIX}${key}`,
        JSON.stringify(state),
    );
}

function readProviderSelection() {
    if (typeof window === "undefined") {
        return "";
    }
    return window.localStorage.getItem(PROVIDER_KEY) ?? "";
}

function pageProfileForPath(pathname: string): PageProfile {
    return (
        PAGE_RULES.find((rule) =>
            matchPath(
                {
                    path: rule.pattern,
                    end: rule.end ?? true,
                },
                pathname,
            ),
        )?.profile ?? {
            title: "当前页面",
            focus: "围绕当前页面整理数据、抽取结构并给出下一步动作",
            draftPlaceholder:
                "贴入当前页面相关的数据、原文或记录，我帮你整理成结构化结果。",
            promptPlaceholder: "例如：把这些内容整理成更适合执行的格式。",
            actions: [
                "整理成结构化清单",
                "提炼重点和缺口",
                "输出下一步执行动作",
            ],
        }
    );
}

function buildSystemPrompt(pathname: string, page: PageProfile, material: string) {
    const segments = [
        "你是 AIO 工作台里的数据整理助手。",
        `当前页面：${page.title}`,
        `当前路由：${pathname}`,
        `本页整理焦点：${page.focus}`,
        "你的职责是帮助用户把原始数据、笔记、配置、报错、列表或页面内容整理成更易执行的结构。",
        "默认优先给出结构化结果，然后再补一句到三句必要说明。",
        "如果用户输入是杂乱文本，优先输出：1. 分类结果 2. 关键结论 3. 下一步动作。",
        "如果适合，用列表、表格字段、分组标题、标签建议、验收清单或 JSON 草案。",
        "不要泛泛而谈，不要写长篇空话。",
    ];

    if (material.trim()) {
        segments.push("用户已经在整理材料区提供了本页上下文，回答时要优先利用这些材料。");
    }

    return segments.join("\n");
}

export default function GlobalAiWorkspace() {
    const location = useLocation();
    const navigate = useNavigate();
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const hidden = location.pathname === "/login";

    const page = useMemo(
        () => pageProfileForPath(location.pathname),
        [location.pathname],
    );
    const threadKey = useMemo(() => location.pathname, [location.pathname]);

    const [open, setOpen] = useState(readPanelOpen);
    const [material, setMaterial] = useState("");
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [prompt, setPrompt] = useState("");
    const [sending, setSending] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [providers, setProviders] = useState<AiProviderConfigDto[]>([]);
    const [provider, setProvider] = useState<string>(readProviderSelection);

    const threadKeyRef = useRef(threadKey);
    const endRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        threadKeyRef.current = threadKey;
    }, [threadKey]);

    useEffect(() => {
        const stored = readThreadState(threadKey);
        setMaterial(stored.material);
        setMessages(stored.messages);
        setPrompt("");
        setError(null);
    }, [threadKey]);

    useEffect(() => {
        writeThreadState(threadKey, {
            material,
            messages,
        });
    }, [material, messages, threadKey]);

    useEffect(() => {
        if (typeof window === "undefined") {
            return;
        }
        window.localStorage.setItem(PANEL_OPEN_KEY, open ? "1" : "0");
    }, [open]);

    useEffect(() => {
        if (typeof window === "undefined") {
            return;
        }
        window.localStorage.setItem(PROVIDER_KEY, provider);
    }, [provider]);

    useEffect(() => {
        const handler = (event: KeyboardEvent) => {
            if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "j") {
                event.preventDefault();
                setOpen((current) => !current);
            }
        };
        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    }, []);

    useEffect(() => {
        let cancelled = false;

        async function loadProviders() {
            try {
                const response = await fetch(`${baseUrl}/api/ai/providers`, {
                    credentials: "include",
                });
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }
                const data = (await response.json()) as AiProviderConfigDto[];
                if (cancelled) {
                    return;
                }
                setProviders(data);
                setProvider((current) => {
                    if (data.some((item) => item.provider === current)) {
                        return current;
                    }
                    return (
                        data.find((item) => item.enabled && item.api_key_configured)
                            ?.provider ??
                        data[0]?.provider ??
                        ""
                    );
                });
            } catch (err) {
                if (!cancelled) {
                    setError(
                        err instanceof Error
                            ? err.message
                            : "加载 AI provider 失败",
                    );
                }
            }
        }

        void loadProviders();
        return () => {
            cancelled = true;
        };
    }, [baseUrl]);

    useEffect(() => {
        if (open) {
            endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
        }
    }, [messages, open]);

    useEffect(() => {
        const handler = (event: Event) => {
            const detail = (event as CustomEvent<AiWorkspaceSeedDetail>).detail;
            if (!detail) {
                return;
            }

            const targetPath = detail.path ?? threadKeyRef.current;
            const stored = readThreadState(targetPath);
            const nextMaterial =
                detail.material == null
                    ? stored.material
                    : detail.appendMaterial && stored.material.trim()
                      ? `${stored.material}\n\n${detail.material}`
                      : detail.material;
            const nextMessages = detail.resetMessages ? [] : stored.messages;

            writeThreadState(targetPath, {
                material: nextMaterial,
                messages: nextMessages,
            });

            if (targetPath === threadKeyRef.current) {
                setMaterial(nextMaterial);
                if (detail.resetMessages) {
                    setMessages([]);
                }
                if (detail.prompt != null) {
                    setPrompt(detail.prompt);
                }
            }

            if (detail.navigate && targetPath !== location.pathname) {
                navigate(targetPath);
            }
            if (detail.open) {
                setOpen(true);
            }
        };

        window.addEventListener(AI_WORKSPACE_SEED_EVENT, handler);
        return () => window.removeEventListener(AI_WORKSPACE_SEED_EVENT, handler);
    }, [location.pathname, navigate]);

    async function sendMessage(prefill?: string) {
        const content = (prefill ?? prompt).trim();
        if (!content || sending) {
            return;
        }
        const activeProvider =
            providers.find((item) => item.provider === provider) ?? null;
        if (!activeProvider || !activeProvider.enabled || !activeProvider.api_key_configured) {
            setError("请先在环境与配置页启用并配置一个可用的 AI provider。");
            return;
        }

        const requestKey = threadKey;
        const requestMaterial = material;
        const requestPage = page;
        const userMessage: ChatMessage = {
            role: "user",
            content,
            createdAt: new Date().toISOString(),
        };
        const nextMessages = [...messages, userMessage];

        setMessages(nextMessages);
        setPrompt("");
        setSending(true);
        setError(null);
        writeThreadState(requestKey, {
            material: requestMaterial,
            messages: nextMessages,
        });

        try {
            const payload = {
                provider: activeProvider.provider,
                messages: [
                    {
                        role: "system",
                        content: buildSystemPrompt(
                            requestKey,
                            requestPage,
                            requestMaterial,
                        ),
                    },
                    ...(requestMaterial.trim()
                        ? [
                              {
                                  role: "user",
                                  content: `当前整理材料：\n${requestMaterial.trim()}`,
                              },
                          ]
                        : []),
                    ...nextMessages.map((message) => ({
                        role: message.role,
                        content: message.content,
                    })),
                ],
            };

            const response = await fetch(`${baseUrl}/api/ai/chat`, {
                method: "POST",
                credentials: "include",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const body = await response.text();
                throw new Error(body || `HTTP ${response.status}`);
            }

            const data = (await response.json()) as ChatResponseDto;
            const assistantMessage: ChatMessage = {
                role: "assistant",
                content: data.message.content.trim() || "没有返回可展示内容。",
                createdAt: new Date().toISOString(),
            };

            if (threadKeyRef.current === requestKey) {
                setMessages((current) => [...current, assistantMessage]);
            } else {
                writeThreadState(requestKey, {
                    material: requestMaterial,
                    messages: [...nextMessages, assistantMessage],
                });
            }
        } catch (err) {
            const nextError =
                err instanceof Error ? err.message : "发送给 AI 失败";
            setError(nextError);
        } finally {
            setSending(false);
        }
    }

    function clearCurrentThread() {
        setMessages([]);
        setPrompt("");
        setError(null);
        writeThreadState(threadKey, {
            material,
            messages: [],
        });
    }

    if (hidden) {
        return null;
    }

    return (
        <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger asChild>
                <Button
                    type="button"
                    size="sm"
                    className="fixed bottom-5 right-5 z-40 h-auto rounded-full border border-stone-900/10 bg-[#171915] px-4 py-3 text-stone-50 shadow-[0_18px_40px_rgba(23,25,21,0.22)]"
                >
                    <Bot className="h-4 w-4" />
                    <span>AI 整理</span>
                    <Badge className="rounded-full bg-stone-50 text-stone-900 hover:bg-stone-50">
                        {page.title}
                    </Badge>
                </Button>
            </SheetTrigger>
            <SheetContent
                side="right"
                className="flex h-full w-[min(46rem,100vw)] flex-col border-l border-stone-300 bg-[#f6f2e8] p-0 sm:max-w-[46rem]"
            >
                <SheetHeader className="border-b border-stone-300 bg-[#fbfaf4] px-5 py-5">
                    <div className="flex items-start justify-between gap-4">
                        <div className="space-y-2">
                            <div className="flex flex-wrap items-center gap-2 text-xs font-semibold uppercase tracking-[0.18em] text-stone-500">
                                <Sparkles className="h-3.5 w-3.5" />
                                AI Data Organizer
                                <Badge variant="outline" className="rounded-full border-stone-300">
                                    Cmd/Ctrl + J
                                </Badge>
                            </div>
                            <SheetTitle className="text-2xl text-stone-900">
                                {page.title}整理窗
                            </SheetTitle>
                            <SheetDescription className="max-w-2xl text-sm leading-6 text-stone-600">
                                这是一层全局工作台聊天窗。把当前页的数据、笔记、配置、报错、
                                列表或草稿贴进来，它会按本页上下文帮你整理。
                            </SheetDescription>
                        </div>
                        <div className="flex shrink-0 flex-col items-end gap-2 text-right">
                            <Badge variant="outline" className="rounded-full border-stone-300 bg-white/70 text-stone-700">
                                {page.focus}
                            </Badge>
                            <div className="flex items-center gap-1.5 text-xs text-stone-500">
                                <Route className="h-3.5 w-3.5" />
                                <span>{location.pathname}</span>
                            </div>
                        </div>
                    </div>
                </SheetHeader>

                <div className="grid shrink-0 gap-3 border-b border-stone-300 bg-[#f8f4ec] px-5 py-4">
                    <section className="rounded-2xl border border-stone-300 bg-white/80 px-4 py-4 shadow-sm">
                        <div className="flex flex-wrap items-center justify-between gap-3">
                            <div>
                                <div className="text-sm font-semibold text-stone-900">
                                    整理材料
                                </div>
                                <p className="mt-1 text-xs leading-5 text-stone-500">
                                    当前页的原始数据、笔记、输出或片段。这里的内容会随当前页面单独保存。
                                </p>
                            </div>
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={() => navigate("/env")}
                            >
                                模型配置
                            </Button>
                        </div>
                        <Textarea
                            value={material}
                            onChange={(event) => setMaterial(event.target.value)}
                            placeholder={page.draftPlaceholder}
                            className="mt-3 min-h-28 border-stone-300 bg-[#fffdf8] leading-6"
                        />
                        <div className="mt-3 flex flex-wrap items-center justify-between gap-3">
                            <div className="text-xs text-stone-500">
                                当前聊天直接走统一 AI 网关，不再使用旧的 OpenAI 专用接口。
                            </div>
                            <div className="w-full sm:w-[220px]">
                                <Select value={provider} onValueChange={setProvider}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="选择 Provider" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {providers.map((item) => (
                                            <SelectItem
                                                key={item.provider}
                                                value={item.provider}
                                            >
                                                {item.label}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                    </section>

                    <section className="flex flex-wrap gap-2">
                        {page.actions.map((action) => (
                            <Button
                                key={action}
                                type="button"
                                variant="outline"
                                size="sm"
                                className="rounded-full border-stone-300 bg-white/70"
                                onClick={() => setPrompt(action)}
                            >
                                {action}
                            </Button>
                        ))}
                        <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="rounded-full text-stone-500"
                            onClick={clearCurrentThread}
                        >
                            <Trash2 className="h-4 w-4" />
                            清空本页会话
                        </Button>
                    </section>
                </div>

                <ScrollArea className="min-h-0 flex-1 px-5 py-4">
                    <div className="space-y-4">
                        {messages.length === 0 ? (
                            <div className="rounded-3xl border border-dashed border-stone-300 bg-white/55 px-5 py-6 text-sm leading-7 text-stone-500">
                                <p className="font-medium text-stone-800">
                                    先把当前页材料贴进上面的整理区，再告诉 AI 你要怎么整理。
                                </p>
                                <p className="mt-2">
                                    它更擅长做归类、去重、提炼标签、压成表格字段、输出验收清单或下一步动作。
                                </p>
                            </div>
                        ) : null}

                        {messages.map((message, index) => {
                            const isUser = message.role === "user";
                            return (
                                <article
                                    key={`${message.createdAt}-${index}`}
                                    className={`flex ${isUser ? "justify-end" : "justify-start"}`}
                                >
                                    <div
                                        className={`max-w-[85%] rounded-[1.4rem] border px-4 py-3 shadow-sm ${
                                            isUser
                                                ? "border-stone-900 bg-[#171915] text-stone-50"
                                                : "border-stone-300 bg-white/90 text-stone-900"
                                        }`}
                                    >
                                        <div className="mb-2 flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] opacity-70">
                                            <span>{isUser ? "You" : "Assistant"}</span>
                                            <span>
                                                {new Date(message.createdAt).toLocaleTimeString(
                                                    "zh-CN",
                                                    {
                                                        hour: "2-digit",
                                                        minute: "2-digit",
                                                    },
                                                )}
                                            </span>
                                        </div>
                                        <div className="whitespace-pre-wrap text-sm leading-7">
                                            {message.content}
                                        </div>
                                    </div>
                                </article>
                            );
                        })}

                        {sending ? (
                            <div className="flex justify-start">
                                <div className="flex items-center gap-2 rounded-full border border-stone-300 bg-white/90 px-4 py-2 text-sm text-stone-600 shadow-sm">
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                    AI 正在整理
                                </div>
                            </div>
                        ) : null}

                        <div ref={endRef} />
                    </div>
                </ScrollArea>

                <div className="shrink-0 border-t border-stone-300 bg-[#fbfaf4] px-5 py-4">
                    {error ? (
                        <div className="mb-3 rounded-2xl border border-rose-300 bg-rose-50 px-4 py-3 text-sm text-rose-700">
                            {error}
                        </div>
                    ) : null}
                    <div className="rounded-[1.6rem] border border-stone-300 bg-white/85 p-3 shadow-sm">
                        <Textarea
                            value={prompt}
                            onChange={(event) => setPrompt(event.target.value)}
                            onKeyDown={(event) => {
                                if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                                    event.preventDefault();
                                    void sendMessage();
                                }
                            }}
                            placeholder={page.promptPlaceholder}
                            className="min-h-24 border-0 bg-transparent px-1 py-1 text-sm leading-7 shadow-none focus-visible:ring-0"
                        />
                        <div className="mt-2 flex flex-wrap items-center justify-between gap-3">
                            <div className="text-xs text-stone-500">
                                当前页会话与整理材料分页面持久化保存。
                            </div>
                            <div className="flex items-center gap-2">
                                <span className="text-xs text-stone-500">
                                    Cmd/Ctrl + Enter 发送
                                </span>
                                <Button
                                    type="button"
                                    onClick={() => void sendMessage()}
                                    disabled={sending || !prompt.trim()}
                                    className="rounded-full px-5"
                                >
                                    {sending ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <Send className="h-4 w-4" />
                                    )}
                                    发送
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </SheetContent>
        </Sheet>
    );
}
