"use client";

import { useEffect, useMemo, useState } from "react";

import type { KnowledgeNoteDto, SyncReportDto } from "@addzero/api-client";

import { api } from "@/lib/api";
import { markdownTitle, timeLabel } from "@/lib/utils";
import { Button, Card, Callout, PageHeader, SectionTitle, StatGrid, Textarea } from "@/components/ui";

export function DashboardPage() {
  const [notes, setNotes] = useState<KnowledgeNoteDto[]>([]);
  const [skillsStatus, setSkillsStatus] = useState<SyncReportDto | null>(null);
  const [draft, setDraft] = useState("# 今日整理\n\n");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void Promise.all([api.listNotes(), api.skillStatus()])
      .then(([nextNotes, status]) => {
        setNotes(nextNotes);
        setSkillsStatus(status);
      })
      .catch((error) => {
        setFeedback(error instanceof Error ? error.message : "加载失败");
      })
      .finally(() => setLoading(false));
  }, []);

  const latestNotes = useMemo(
    () =>
      [...notes]
        .sort((left, right) => right.updated_at.localeCompare(left.updated_at))
        .slice(0, 5),
    [notes],
  );

  return (
    <div className="space-y-6">
      <PageHeader
        title="笔记工作台"
        subtitle="总览页不再混合 Dioxus 本地状态；它只展示 PG 笔记、Skill 同步状态和快速捕获入口。"
        actions={
          <Button
            tone="accent"
            onClick={async () => {
              try {
                const saved = await api.saveNote({
                  relative_path: `dashboard/${Date.now()}.md`,
                  body: draft,
                });
                setNotes((current) => [saved, ...current]);
                setFeedback(`已保存：${saved.title}`);
              } catch (error) {
                setFeedback(error instanceof Error ? error.message : "保存失败");
              }
            }}
          >
            快速保存
          </Button>
        }
      />

      {feedback ? <Callout>{feedback}</Callout> : null}

      <StatGrid
        items={[
          {
            label: "Knowledge Notes",
            value: loading ? "…" : String(notes.length),
            detail: "PG 笔记条目",
          },
          {
            label: "Skill Sync",
            value: skillsStatus ? String(skillsStatus.synced_count) : "—",
            detail: skillsStatus?.status ?? "等待状态",
          },
          {
            label: "PG Status",
            value: skillsStatus?.pg_online ? "Online" : "Offline",
            detail: skillsStatus?.fs_root ?? "—",
          },
          {
            label: "Draft",
            value: markdownTitle(draft),
            detail: "当前快速捕获标题",
          },
        ]}
      />

      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <Card>
          <SectionTitle title="最近笔记" detail="按更新时间倒序展示，保持总览页可扫描。" />
          <div className="space-y-3">
            {latestNotes.map((note) => (
              <div key={note.source_path} className="rounded-lg border border-white/10 bg-black/20 p-4">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <div className="font-medium text-white">{note.title}</div>
                    <div className="mt-1 text-xs uppercase tracking-[0.18em] text-zinc-500">
                      {note.relative_path}
                    </div>
                  </div>
                  <span className="text-xs text-zinc-500">{timeLabel(note.updated_at)}</span>
                </div>
                <p className="mt-3 text-sm text-zinc-400">{note.preview}</p>
              </div>
            ))}
            {!latestNotes.length && (
              <div className="rounded-lg border border-dashed border-white/10 p-6 text-sm text-zinc-500">
                还没有可展示的笔记。
              </div>
            )}
          </div>
        </Card>

        <Card>
          <SectionTitle
            title="快速捕获"
            detail="保留 Markdown source + live preview 的双栏工作方式，但先用 React 实现。"
          />
          <div className="grid gap-4 lg:grid-cols-2">
            <Textarea
              data-command-search="true"
              rows={16}
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="直接记录今天要沉淀的内容…"
            />
            <div className="markdown-preview rounded-lg border border-white/10 bg-black/20 p-4">
              <pre className="whitespace-pre-wrap font-inherit text-sm text-zinc-300">{draft}</pre>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
