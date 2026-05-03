"use client";

import { useEffect, useMemo, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import type { ChatMessageDto, KnowledgeNoteDto } from "@addzero/api-client";

import { api } from "@/lib/api";
import { markdownTitle } from "@/lib/utils";
import { Button, Callout, Card, Input, PageHeader, SectionTitle, Textarea } from "@/components/ui";

interface NoteDraft {
  sourcePath: string;
  relativePath: string;
  body: string;
}

function toDraft(note?: KnowledgeNoteDto | null): NoteDraft {
  if (!note) {
    return {
      sourcePath: "",
      relativePath: "",
      body: "# 新笔记\n\n",
    };
  }
  return {
    sourcePath: note.source_path,
    relativePath: note.relative_path,
    body: note.body,
  };
}

export function NotesPage() {
  const [notes, setNotes] = useState<KnowledgeNoteDto[]>([]);
  const [selectedPath, setSelectedPath] = useState("");
  const [draft, setDraft] = useState<NoteDraft>(toDraft());
  const [search, setSearch] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [assistantPrompt, setAssistantPrompt] = useState("");
  const [assistantMessages, setAssistantMessages] = useState<ChatMessageDto[]>([]);

  const refresh = async () => {
    try {
      const nextNotes = await api.listNotes();
      setNotes(nextNotes);
      if (!selectedPath && nextNotes[0]) {
        setSelectedPath(nextNotes[0].source_path);
        setDraft(toDraft(nextNotes[0]));
      }
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const visibleNotes = useMemo(() => {
    const query = search.trim().toLowerCase();
    return notes.filter((note) => {
      return (
        !query ||
        note.title.toLowerCase().includes(query) ||
        note.preview.toLowerCase().includes(query) ||
        note.relative_path.toLowerCase().includes(query)
      );
    });
  }, [notes, search]);

  return (
    <div className="space-y-6">
      <PageHeader
        title="笔记"
        subtitle="保留 source + preview 双栏，以及底部基于 OpenAI 兼容接口的整理助手。"
        actions={
          <Button
            onClick={() => {
              setSelectedPath("");
              setDraft(toDraft());
            }}
          >
            新建
          </Button>
        }
      />
      {feedback ? <Callout>{feedback}</Callout> : null}

      <div className="grid gap-6 xl:grid-cols-[20rem_minmax(0,1fr)]">
        <Card>
          <SectionTitle title="笔记列表" detail={`${visibleNotes.length} / ${notes.length} 条`} />
          <Input
            data-command-search="true"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="搜索标题、摘要或路径"
          />
          <div className="mt-4 space-y-2">
            {visibleNotes.map((note) => (
              <button
                key={note.source_path}
                type="button"
                onClick={() => {
                  setSelectedPath(note.source_path);
                  setDraft(toDraft(note));
                  setAssistantMessages([]);
                }}
                className={`w-full rounded-lg border px-3 py-3 text-left transition ${
                  selectedPath === note.source_path
                    ? "border-emerald-400/40 bg-emerald-500/10"
                    : "border-white/10 bg-black/20 hover:bg-white/5"
                }`}
              >
                <div className="font-medium text-white">{note.title}</div>
                <div className="mt-1 text-xs text-zinc-500">{note.relative_path}</div>
                <p className="mt-2 text-sm text-zinc-400">{note.preview}</p>
              </button>
            ))}
          </div>
        </Card>

        <div className="space-y-6">
          <Card>
            <SectionTitle
              title={markdownTitle(draft.body)}
              detail={draft.sourcePath || "新建状态，保存后写入 PG knowledge_documents"}
              actions={
                <div className="flex flex-wrap gap-2">
                  <Button
                    tone="accent"
                    onClick={async () => {
                      try {
                        const saved = await api.saveNote({
                          source_path: draft.sourcePath || null,
                          relative_path: draft.relativePath || `notes/${Date.now()}.md`,
                          body: draft.body,
                        });
                        setFeedback(`已保存：${saved.title}`);
                        await refresh();
                        setSelectedPath(saved.source_path);
                        setDraft(toDraft(saved));
                      } catch (error) {
                        setFeedback(error instanceof Error ? error.message : "保存失败");
                      }
                    }}
                  >
                    保存
                  </Button>
                  <Button
                    tone="danger"
                    disabled={!draft.sourcePath}
                    onClick={async () => {
                      if (!draft.sourcePath) return;
                      try {
                        await api.deleteNote({ source_path: draft.sourcePath });
                        setFeedback("已删除");
                        setDraft(toDraft());
                        setSelectedPath("");
                        await refresh();
                      } catch (error) {
                        setFeedback(error instanceof Error ? error.message : "删除失败");
                      }
                    }}
                  >
                    删除
                  </Button>
                </div>
              }
            />
            <div className="mb-4">
              <Input
                value={draft.relativePath}
                onChange={(event) =>
                  setDraft((current) => ({ ...current, relativePath: event.target.value }))
                }
                placeholder="notes/example.md"
              />
            </div>
            <div className="grid gap-4 xl:grid-cols-2">
              <Textarea
                rows={20}
                value={draft.body}
                onChange={(event) =>
                  setDraft((current) => ({ ...current, body: event.target.value }))
                }
              />
              <div className="markdown-preview rounded-lg border border-white/10 bg-black/20 p-4">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{draft.body}</ReactMarkdown>
              </div>
            </div>
          </Card>

          <Card>
            <SectionTitle title="整理助手" detail="将当前草稿作为上下文，调用现有聊天接口。" />
            <div className="space-y-3">
              {assistantMessages.map((message, index) => (
                <div key={index} className="rounded-lg border border-white/10 bg-black/20 p-3 text-sm">
                  <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">
                    {message.role}
                  </div>
                  <div className="mt-2 whitespace-pre-wrap text-zinc-200">{message.content}</div>
                </div>
              ))}
              <Textarea
                rows={5}
                value={assistantPrompt}
                onChange={(event) => setAssistantPrompt(event.target.value)}
                placeholder="例如：把这篇笔记整理成更清晰的大纲"
              />
              <Button
                tone="accent"
                disabled={!assistantPrompt.trim()}
                onClick={async () => {
                  try {
                    const seedMessages: ChatMessageDto[] = [
                      {
                        role: "system",
                        content: `当前笔记内容：\n\n${draft.body}`,
                      },
                      ...assistantMessages,
                      {
                        role: "user",
                        content: assistantPrompt,
                      },
                    ];
                    const response = await api.runChat({ messages: seedMessages });
                    setAssistantMessages([
                      ...assistantMessages,
                      { role: "user", content: assistantPrompt },
                      response.message,
                    ]);
                    setAssistantPrompt("");
                  } catch (error) {
                    setFeedback(error instanceof Error ? error.message : "请求失败");
                  }
                }}
              >
                生成建议
              </Button>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
