import {
    BookOpen,
    Building2,
    LockKeyhole,
    Menu,
    Shield,
    Users,
} from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@addzero/ui";

const modules = [
    { icon: Users, title: "用户管理", desc: "管理员、操作者、服务身份与会话边界" },
    { icon: Shield, title: "角色管理", desc: "能力授权、资源访问和模块级权限模型" },
    { icon: Menu, title: "导航管理", desc: "平台工作台的信息架构与菜单树" },
    { icon: Building2, title: "组织上下文", desc: "团队、空间、环境或租户范围" },
    { icon: BookOpen, title: "字典与元数据", desc: "状态枚举、模板类型、运行时标签" },
    { icon: LockKeyhole, title: "安全边界", desc: "脚本权限、外部调用和凭证策略" },
];

export default function SystemPage() {
    return (
        <div className="space-y-8">
            <Card>
                <CardHeader className="border-b">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <Shield className="h-3.5 w-3.5" />
                        System Workbench
                    </div>
                    <CardTitle className="mt-3 text-3xl tracking-tight">
                        系统工作台
                    </CardTitle>
                    <CardDescription className="mt-2 max-w-3xl text-sm">
                        系统层不是普通后台尾页。对于脚本平台，它要定义身份、权限、导航、元数据和运行边界。
                    </CardDescription>
                </CardHeader>

                <CardContent className="grid gap-3 p-5 sm:grid-cols-2 xl:grid-cols-3">
                    {modules.map((module) => (
                        <Card key={module.title} className="shadow-none">
                            <CardContent className="px-4 py-3">
                            <div className="flex items-center gap-2 text-sm font-medium">
                                <module.icon className="h-4 w-4 text-muted-foreground" />
                                {module.title}
                            </div>
                            <p className="mt-2 text-sm text-muted-foreground">{module.desc}</p>
                            </CardContent>
                        </Card>
                    ))}
                </CardContent>
            </Card>
        </div>
    );
}
