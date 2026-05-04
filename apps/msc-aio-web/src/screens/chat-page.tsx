"use client";

import { useState } from "react";

import type { ChatMessageDto } from "@addzero/api-client";

import { api } from "@/lib/api";
import { Button, Card, Callout, PageHeader, SectionTitle, Textarea } from "@/components/ui";

export function ChatPage() {
  const [messages, setMessages] = useState<ChatMessageDto[]>([]);
  const [input, setInput] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  return (
    <div className="space-y-6">
      <PageHeader
        title="聊天工作台"
        subtitle="聊天页只调用 `/api/openai-chat/chat`，保持现有 cookie 与配置链路。"
      />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <Card>
        <SectionTitle title="会话" detail="当前为多轮文本对话。" />
        <div className="space-y-3">
          {messages.map((message, index) => (
            <div key={index} className="rounded-lg border border-white/10 bg-black/20 p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">
                {message.role === "user" ? "你" : "AI"}
              </div>
              <p className="mt-2 whitespace-pre-wrap text-sm text-zinc-200">{message.content}</p>
            </div>
          ))}
          {!messages.length ? (
            <div className="rounded-lg border border-dashed border-white/10 p-6 text-sm text-zinc-500">
              还没有消息，先发一条试试。
            </div>
          ) : null}
        </div>

        <div className="mt-6 space-y-3">
          <Textarea
            data-command-search="true"
            rows={6}
            value={input}
            onChange={(event) => setInput(event.target.value)}
            placeholder="直接开始聊天…"
          />
          <div className="flex flex-wrap gap-2">
            <Button
              tone="accent"
              disabled={pending || !input.trim()}
              onClick={async () => {
                const nextMessage = { role: "user", content: input.trim() };
                const nextMessages = [...messages, nextMessage];
                setMessages(nextMessages);
                setInput("");
                setPending(true);
                try {
                  const response = await api.runChat({ messages: nextMessages });
                  setMessages([...nextMessages, response.message]);
                } catch (error) {
                  setFeedback(error instanceof Error ? error.message : "聊天失败");
                } finally {
                  setPending(false);
                }
              }}
            >
              {pending ? "输出中…" : "发送"}
            </Button>
            <Button onClick={() => setMessages([])} disabled={!messages.length || pending}>
              清空会话
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
}
