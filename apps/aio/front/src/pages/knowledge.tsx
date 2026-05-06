import { Brain, Database, FileSearch, Link2 } from "lucide-react";

const sources = [
    "文件系统知识目录",
    "脚本模板与运行记录",
    "软件目录与资源元数据",
    "未来的对话上下文与向量索引",
];

export default function KnowledgePage() {
    return (
        <div className="space-y-8">
            <section className="rounded-lg border bg-card">
                <div className="border-b px-5 py-4">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <Brain className="h-3.5 w-3.5" />
                        Knowledge Workbench
                    </div>
                    <h1 className="mt-3 text-3xl font-semibold tracking-tight">
                        知识与记忆工作台
                    </h1>
                    <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                        平台需要一层正式的知识底座，承接索引、关系、来源追踪和后续 AI 上下文供应，
                        而不是只放一个“知识库页面”。
                    </p>
                </div>

                <div className="grid gap-0 md:grid-cols-3">
                    <Signal
                        icon={<Database className="h-4 w-4" />}
                        title="PostgreSQL"
                        detail="作为正式持久化源，承接知识节点、来源和索引元数据。"
                    />
                    <Signal
                        icon={<Link2 className="h-4 w-4" />}
                        title="来源映射"
                        detail="记录每个节点来自哪里，保持导入态和正式态可追踪。"
                    />
                    <Signal
                        icon={<FileSearch className="h-4 w-4" />}
                        title="检索供给"
                        detail="为脚本生成、流程执行和 AI 编排提供上下文素材。"
                    />
                </div>
            </section>

            <section className="grid gap-6 xl:grid-cols-[1fr_0.9fr]">
                <div className="rounded-lg border bg-card p-5">
                    <h2 className="text-base font-semibold">知识来源</h2>
                    <div className="mt-4 grid gap-3">
                        {sources.map((source) => (
                            <div key={source} className="rounded-md border px-4 py-3 text-sm">
                                {source}
                            </div>
                        ))}
                    </div>
                </div>

                <div className="rounded-lg border bg-card p-5">
                    <h2 className="text-base font-semibold">当前状态</h2>
                    <div className="mt-4 space-y-3 text-sm text-muted-foreground">
                        <p>PG 连通后，这里要展示知识同步批次、异常节点和索引覆盖率。</p>
                        <p>接下来不该只做“列表页”，而要做知识节点、来源、异常和维护动作的四块面板。</p>
                    </div>
                </div>
            </section>
        </div>
    );
}

function Signal({
    icon,
    title,
    detail,
}: {
    icon: React.ReactNode;
    title: string;
    detail: string;
}) {
    return (
        <div className="border-t px-5 py-4 first:border-t-0 md:border-t-0 md:border-l first:md:border-l-0">
            <div className="flex items-center gap-2 text-sm font-medium">
                <span className="text-muted-foreground">{icon}</span>
                {title}
            </div>
            <p className="mt-2 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}
