import { FolderOpen, HardDrive } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";

export default function StoragePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">存储</h1>
        <p className="mt-1 text-muted-foreground">MinIO 对象存储文件管理</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-lg">
            <HardDrive className="h-5 w-5" />
            存储服务
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-4 rounded-lg border p-4">
            <FolderOpen className="h-8 w-8 text-muted-foreground" />
            <div>
              <p className="font-medium">MinIO 未连接</p>
              <p className="text-sm text-muted-foreground">
                配置 AIO_MINIO_ENDPOINT / AIO_MINIO_ACCESS_KEY / AIO_MINIO_SECRET_KEY 环境变量以启用对象存储
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
