"use client";

import { Card, DataTable, PageHeader, SectionTitle } from "@/components/ui";

const auditRows = [
  ["09:12", "审计任务结束", "策略审计通过，没有新增风险项", "system"],
  ["08:41", "权限模板变更", "P-09 删除了一条遗留白名单", "Chen"],
  ["昨天 18:20", "发布完成", "release-0426 已部署到 production", "Luna"],
];

export function AuditPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="审计日志"
        subtitle="现阶段仍保持静态时间线读模型；等正式审计域 API 成熟后再切换。"
      />
      <Card>
        <SectionTitle title="最近日志" />
        <DataTable columns={["When", "Title", "Detail", "Actor"]} rows={auditRows} />
      </Card>
    </div>
  );
}
