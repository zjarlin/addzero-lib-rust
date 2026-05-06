import { Shield, Users, Menu, Building2, BookOpen } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";

const modules = [
  { icon: Users, title: "用户管理", desc: "用户 CRUD + 角色授权" },
  { icon: Shield, title: "角色管理", desc: "角色 CRUD + 菜单授权" },
  { icon: Menu, title: "菜单管理", desc: "菜单树 CURD + 路由同步" },
  { icon: Building2, title: "部门管理", desc: "组织架构树" },
  { icon: BookOpen, title: "字典管理", desc: "字典组 + 字典项" },
];

export default function SystemPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">系统管理</h1>
        <p className="mt-1 text-muted-foreground">
          用户 / 角色 / 菜单 / 部门 / 字典
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">需要 PostgreSQL</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">
            系统管理模块依赖 PostgreSQL。设置 DATABASE_URL 环境变量后，以下功能将自动激活：
          </p>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {modules.map((m) => (
              <div
                key={m.title}
                className="flex items-center gap-3 rounded-lg border p-3"
              >
                <m.icon className="h-5 w-5 text-muted-foreground" />
                <div>
                  <p className="text-sm font-medium">{m.title}</p>
                  <p className="text-xs text-muted-foreground">{m.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
