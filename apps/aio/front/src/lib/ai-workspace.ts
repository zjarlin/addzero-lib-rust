export const AI_WORKSPACE_SEED_EVENT = "aio:ai-workspace-seed";
export const AI_WORKSPACE_PANEL_EVENT = "aio:ai-workspace-panel";

export interface AiWorkspaceSeedDetail {
    path?: string;
    material?: string;
    prompt?: string;
    open?: boolean;
    navigate?: boolean;
    appendMaterial?: boolean;
    resetMessages?: boolean;
}

export interface AiWorkspacePanelDetail {
    open?: boolean;
    toggle?: boolean;
}

export interface PageProfile {
    title: string;
    focus: string;
    draftPlaceholder: string;
    promptPlaceholder: string;
    actions: string[];
}

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

export function emitAiWorkspaceSeed(detail: AiWorkspaceSeedDetail) {
    if (typeof window === "undefined") {
        return;
    }
    window.dispatchEvent(
        new CustomEvent<AiWorkspaceSeedDetail>(AI_WORKSPACE_SEED_EVENT, {
            detail,
        }),
    );
}

export function emitAiWorkspacePanel(detail: AiWorkspacePanelDetail) {
    if (typeof window === "undefined") {
        return;
    }
    window.dispatchEvent(
        new CustomEvent<AiWorkspacePanelDetail>(AI_WORKSPACE_PANEL_EVENT, {
            detail,
        }),
    );
}

function matchRoutePattern(pathname: string, pattern: string, end = true) {
    const cleanPath = pathname.split("?")[0].replace(/\/+$/, "") || "/";
    const cleanPattern = pattern.replace(/\/+$/, "") || "/";
    const pathParts = cleanPath === "/" ? [] : cleanPath.slice(1).split("/");
    const patternParts =
        cleanPattern === "/" ? [] : cleanPattern.slice(1).split("/");

    if (end && pathParts.length !== patternParts.length) {
        return false;
    }
    if (!end && pathParts.length < patternParts.length) {
        return false;
    }

    return patternParts.every((part, index) => {
        const pathPart = pathParts[index];
        return Boolean(pathPart) && (part.startsWith(":") || part === pathPart);
    });
}

export function pageProfileForPath(pathname: string): PageProfile {
    return (
        PAGE_RULES.find((rule) =>
            matchRoutePattern(pathname, rule.pattern, rule.end ?? true),
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
