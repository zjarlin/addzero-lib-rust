import { Brain, Database } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";

export default function KnowledgePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">知识库</h1>
        <p className="mt-1 text-muted-foreground">Knowledge Graph 知识图谱</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-lg">
              <Brain className="h-5 w-5" />
              Feed
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              知识图谱需要 PostgreSQL 连接。设置 DATABASE_URL 后自动激活。
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-lg">
              <Database className="h-5 w-5" />
              数据源
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              PG 就绪后，自动从文件系统 / Blinko / 软件目录索引知识节点。
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
