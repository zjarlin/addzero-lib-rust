"use client";

import { Button, Card, DataTable, PageHeader, SectionTitle } from "@/components/ui";

const packageRows = [
  {
    slug: "raycast-macos",
    software_title: "Raycast",
    version: "1.81.0",
    platform: "macOS",
    format: "dmg",
    status: "已归档",
    source: "手动收口",
  },
  {
    slug: "uv-cross-platform",
    software_title: "uv",
    version: "0.7.x",
    platform: "CrossPlatform",
    format: "installer",
    status: "待校验",
    source: "CLI 市场联动",
  },
  {
    slug: "docker-desktop",
    software_title: "Docker Desktop",
    version: "4.41.x",
    platform: "macOS",
    format: "dmg",
    status: "整理中",
    source: "下载站",
  },
];

export function PackagesPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="下载与安装"
        subtitle="旧页面依赖编译期 catalog；迁移阶段先保留为前端静态读模型，后续再收口成正式 PG/API。"
      />
      <Card>
        <SectionTitle
          title="安装包视图"
          detail="当前页只迁 route surface，不延续 Dioxus 编译期 embed。"
          actions={<Button disabled>暂不编辑</Button>}
        />
        <DataTable
          columns={["Slug", "Title", "Version", "Platform", "Format", "Status", "Source"]}
          rows={packageRows.map((item) => [
            item.slug,
            item.software_title,
            item.version,
            item.platform,
            item.format,
            item.status,
            item.source,
          ])}
        />
      </Card>
    </div>
  );
}
