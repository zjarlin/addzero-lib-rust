import { FolderTree, HardDrive, PackageOpen, ShieldCheck } from "lucide-react";

const assets = [
    {
        title: "对象存储",
        detail: "MinIO / S3 兼容资源池，承接文件、包和导出物。",
    },
    {
        title: "资源目录",
        detail: "统一维护脚本附件、模板产物、插件包和下载资产。",
    },
    {
        title: "访问策略",
        detail: "把分享链接、权限和生命周期放到正式资源层，而不是分散在业务页。",
    },
];

export default function StoragePage() {
    return (
        <div className="space-y-8">
            <section className="rounded-lg border bg-card">
                <div className="border-b px-5 py-4">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <HardDrive className="h-3.5 w-3.5" />
                        Asset Workbench
                    </div>
                    <h1 className="mt-3 text-3xl font-semibold tracking-tight">
                        资源与存储工作台
                    </h1>
                    <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
                        资源层要服务整个平台，不只是一个 MinIO 文件列表。它会承接脚本输入输出、
                        导出工程、插件包、共享链接和归档资产。
                    </p>
                </div>

                <div className="grid gap-0 md:grid-cols-3">
                    <Stat icon={<FolderTree className="h-4 w-4" />} title="目录管理" detail="统一组织脚本、插件、模板和导出物。" />
                    <Stat icon={<PackageOpen className="h-4 w-4" />} title="产物归档" detail="为 CLI 生成物和插件包保留正式发布位。" />
                    <Stat icon={<ShieldCheck className="h-4 w-4" />} title="访问控制" detail="分享、过期、权限与引用关系进入资源治理。" />
                </div>
            </section>

            <section className="rounded-lg border bg-card p-5">
                <h2 className="text-base font-semibold">资源层职责</h2>
                <div className="mt-4 grid gap-3 lg:grid-cols-3">
                    {assets.map((asset) => (
                        <div key={asset.title} className="rounded-lg border px-4 py-3">
                            <div className="text-sm font-medium">{asset.title}</div>
                            <p className="mt-2 text-sm text-muted-foreground">{asset.detail}</p>
                        </div>
                    ))}
                </div>
            </section>
        </div>
    );
}

function Stat({
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
