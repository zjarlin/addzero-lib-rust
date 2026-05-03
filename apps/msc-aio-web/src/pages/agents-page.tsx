"use client";

import { useRouter, useSearchParams } from "next/navigation";
import { useEffect, useMemo, useState } from "react";

import type { SkillDto } from "@addzero/api-client";

import { api } from "@/lib/api";
import { timeLabel, titleFromSkillName } from "@/lib/utils";
import { Button, Callout, Card, DataTable, Field, Input, PageHeader, SectionTitle, Textarea } from "@/components/ui";

export function AgentsPage() {
  const router = useRouter();
  const [skills, setSkills] = useState<SkillDto[]>([]);
  const [search, setSearch] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    setLoading(true);
    try {
      setSkills(await api.listSkills());
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const visibleSkills = useMemo(() => {
    const query = search.trim().toLowerCase();
    return skills.filter((skill) => {
      return (
        !query ||
        skill.name.toLowerCase().includes(query) ||
        skill.description.toLowerCase().includes(query) ||
        skill.keywords.some((keyword) => keyword.toLowerCase().includes(query))
      );
    });
  }, [search, skills]);

  return (
    <div className="space-y-6">
      <PageHeader
        title="Skill 资产"
        subtitle="保留 skills 目录的资产语义，但浏览器前端只消费 `/api/skills*`。"
        actions={
          <>
            <Button
              onClick={async () => {
                try {
                  const report = await api.syncSkills();
                  setFeedback(report.detail || `已同步 ${report.synced_count} 条`);
                  await refresh();
                } catch (error) {
                  setFeedback(error instanceof Error ? error.message : "同步失败");
                }
              }}
            >
              同步
            </Button>
            <Button tone="accent" onClick={() => router.push("/agents/editor")}>
              新建 Skill
            </Button>
          </>
        }
      />
      {feedback ? <Callout>{feedback}</Callout> : null}

      <Card>
        <SectionTitle title="目录" detail={loading ? "正在加载…" : `${visibleSkills.length} / ${skills.length} 条`} />
        <div className="mb-4">
          <Input
            data-command-search="true"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="搜索 Skill 名称、说明或关键词"
          />
        </div>
        <DataTable
          columns={["Name", "Keywords", "Source", "Updated", "Action"]}
          rows={visibleSkills.map((skill) => [
            <div key={`${skill.name}-name`}>
              <div className="font-medium text-white">{titleFromSkillName(skill.name)}</div>
              <div className="mt-1 text-xs text-zinc-500">{skill.name}</div>
            </div>,
            <div key={`${skill.name}-keywords`} className="flex flex-wrap gap-2">
              {skill.keywords.slice(0, 4).map((keyword) => (
                <span key={keyword} className="rounded-full bg-white/5 px-2 py-1 text-xs text-zinc-300">
                  {keyword}
                </span>
              ))}
            </div>,
            <span key={`${skill.name}-source`} className="text-zinc-300">
              {skill.source}
            </span>,
            <span key={`${skill.name}-updated`} className="text-zinc-400">
              {timeLabel(skill.updated_at)}
            </span>,
            <Button
              key={`${skill.name}-action`}
              onClick={() => router.push(`/agents/editor?name=${encodeURIComponent(skill.name)}`)}
            >
              打开
            </Button>,
          ])}
        />
      </Card>
    </div>
  );
}

export function AgentEditorPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const skillName = searchParams.get("name");
  const [form, setForm] = useState({
    name: "",
    keywords: "",
    description: "",
    body: "",
  });
  const [feedback, setFeedback] = useState<string | null>(null);
  const [loading, setLoading] = useState(Boolean(skillName));

  useEffect(() => {
    if (!skillName) {
      return;
    }
    void api
      .getSkill(skillName)
      .then((skill) => {
        if (!skill) {
          setFeedback(`未找到 ${skillName}`);
          return;
        }
        setForm({
          name: skill.name,
          keywords: skill.keywords.join(", "),
          description: skill.description,
          body: skill.body,
        });
      })
      .catch((error) => setFeedback(error instanceof Error ? error.message : "加载失败"))
      .finally(() => setLoading(false));
  }, [skillName]);

  return (
    <div className="space-y-6">
      <PageHeader
        title={skillName ? "编辑 Skill" : "新建 Skill"}
        subtitle="Agent 编辑页改成静态导出安全的查询参数路由，不再使用动态段。"
        actions={<Button onClick={() => router.push("/agents")}>返回列表</Button>}
      />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <Card>
        <SectionTitle title={loading ? "正在加载…" : "Skill 定义"} />
        <div className="grid gap-4 md:grid-cols-2">
          <Field label="name">
            <Input
              value={form.name}
              onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
            />
          </Field>
          <Field label="keywords">
            <Input
              value={form.keywords}
              onChange={(event) =>
                setForm((current) => ({ ...current, keywords: event.target.value }))
              }
            />
          </Field>
        </div>
        <div className="mt-4">
          <Field label="description">
            <Textarea
              rows={4}
              value={form.description}
              onChange={(event) =>
                setForm((current) => ({ ...current, description: event.target.value }))
              }
            />
          </Field>
        </div>
        <div className="mt-4">
          <Field label="body">
            <Textarea
              rows={18}
              value={form.body}
              onChange={(event) => setForm((current) => ({ ...current, body: event.target.value }))}
            />
          </Field>
        </div>
        <div className="mt-6 flex flex-wrap gap-2">
          <Button
            tone="accent"
            onClick={async () => {
              try {
                const saved = await api.upsertSkill({
                  name: form.name,
                  keywords: form.keywords
                    .split(",")
                    .map((item) => item.trim())
                    .filter(Boolean),
                  description: form.description,
                  body: form.body,
                });
                setFeedback(`已保存 ${saved.name}`);
                router.push(`/agents/editor?name=${encodeURIComponent(saved.name)}`);
              } catch (error) {
                setFeedback(error instanceof Error ? error.message : "保存失败");
              }
            }}
          >
            保存
          </Button>
          {skillName ? (
            <Button
              tone="danger"
              onClick={async () => {
                try {
                  await api.deleteSkill(skillName);
                  router.push("/agents");
                } catch (error) {
                  setFeedback(error instanceof Error ? error.message : "删除失败");
                }
              }}
            >
              删除
            </Button>
          ) : null}
        </div>
      </Card>
    </div>
  );
}
