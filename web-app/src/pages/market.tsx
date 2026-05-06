import { Store, Package } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";

export default function MarketPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">CLI Market</h1>
        <p className="mt-1 text-muted-foreground">CLI 工具市场 — 安装 / 发布 / 导入 / 导出</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-lg">
            <Store className="h-5 w-5" />
            CLI 市场
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4 rounded-lg border p-4">
            <Package className="h-8 w-8 text-muted-foreground" />
            <div>
              <p className="font-medium">需要 PostgreSQL</p>
              <p className="text-sm text-muted-foreground">
                配置 DATABASE_URL 后自动激活 CLI 工具市场
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
