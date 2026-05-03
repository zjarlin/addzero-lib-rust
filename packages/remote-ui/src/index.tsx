"use client";

import clsx from "clsx";
import { MonitorPlay, PlugZap, ShieldAlert } from "lucide-react";
import type { ReactNode } from "react";

export interface RemoteDevice {
  id: string;
  name: string;
  platform: string;
  role: string;
  status: string;
  notes?: string | null;
}

export interface RemoteStatusChip {
  label: string;
  emphasis?: boolean;
}

export interface RemoteAction {
  label: string;
  tone?: "primary" | "neutral" | "danger";
}

export interface RemotePermissionNotice {
  title: string;
  body: string;
  bullets: string[];
  ctaPrimary: string;
  ctaSecondary: string;
}

export interface RemoteStageModel {
  title: string;
  subtitle: string;
  actions: RemoteAction[];
  statusChips: RemoteStatusChip[];
  placeholderTitle: string;
  placeholderBody: string;
  permissionNotice?: RemotePermissionNotice | null;
}

export interface RemoteSessionSummary {
  state: string;
  latestFrame?: {
    width: number;
    height: number;
    sequence: number;
  } | null;
  clipboard?: {
    content: string;
  } | null;
  pendingTransfer?: {
    fileName: string;
    totalBytes: number;
    chunkIndex: number;
    chunkCount: number;
  } | null;
}

export function RemoteShell({
  title,
  sidebar,
  stage,
  detail,
}: {
  title: string;
  sidebar: ReactNode;
  stage: ReactNode;
  detail: ReactNode;
}) {
  return (
    <div className="min-h-screen bg-[#060816] text-zinc-100">
      <div className="mx-auto grid min-h-screen max-w-[1700px] gap-4 px-4 py-4 xl:grid-cols-[18rem_minmax(0,1fr)_22rem]">
        <aside className="rounded-2xl border border-cyan-400/10 bg-cyan-950/10 p-4">
          <div className="mb-6">
            <p className="text-xs uppercase tracking-[0.28em] text-cyan-300/60">
              Addzero Remote
            </p>
            <h1 className="mt-2 text-2xl font-semibold text-white">{title}</h1>
          </div>
          {sidebar}
        </aside>
        <main className="rounded-2xl border border-white/10 bg-zinc-950/80 p-4 md:p-6">
          {stage}
        </main>
        <aside className="rounded-2xl border border-white/10 bg-zinc-950/80 p-4">
          {detail}
        </aside>
      </div>
    </div>
  );
}

export function DeviceCard({ device, active }: { device: RemoteDevice; active?: boolean }) {
  return (
    <article
      className={clsx(
        "rounded-xl border p-4 transition",
        active
          ? "border-cyan-300/40 bg-cyan-400/10"
          : "border-white/10 bg-white/5 hover:border-white/20",
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-semibold text-white">{device.name}</div>
          <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
            {device.platform} · {device.role}
          </div>
        </div>
        <span
          className={clsx(
            "rounded-full px-2 py-1 text-[11px] uppercase tracking-[0.18em]",
            device.status.toLowerCase() === "online"
              ? "bg-emerald-500/10 text-emerald-200"
              : "bg-zinc-700 text-zinc-300",
          )}
        >
          {device.status}
        </span>
      </div>
      {device.notes ? <p className="mt-3 text-sm text-zinc-400">{device.notes}</p> : null}
    </article>
  );
}

export function RemoteStage({ model }: { model: RemoteStageModel }) {
  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div>
          <h2 className="text-2xl font-semibold text-white">{model.title}</h2>
          <p className="mt-2 max-w-2xl text-sm text-zinc-400">{model.subtitle}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          {model.actions.map((action) => (
            <button
              key={action.label}
              type="button"
              className={clsx(
                "rounded-md border px-3 py-2 text-sm transition",
                action.tone === "primary" && "border-cyan-300/40 bg-cyan-400/10 text-cyan-100",
                action.tone === "danger" && "border-red-400/40 bg-red-500/10 text-red-100",
                (!action.tone || action.tone === "neutral") &&
                  "border-white/10 bg-white/5 text-zinc-200 hover:bg-white/10",
              )}
            >
              {action.label}
            </button>
          ))}
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        {model.statusChips.map((chip) => (
          <span
            key={chip.label}
            className={clsx(
              "rounded-full px-3 py-1 text-xs uppercase tracking-[0.2em]",
              chip.emphasis
                ? "bg-cyan-400/15 text-cyan-100"
                : "bg-white/5 text-zinc-400",
            )}
          >
            {chip.label}
          </span>
        ))}
      </div>
      <section className="rounded-2xl border border-white/10 bg-gradient-to-br from-cyan-500/10 via-transparent to-emerald-400/10 p-8">
        <div className="flex min-h-[28rem] flex-col items-center justify-center rounded-2xl border border-dashed border-cyan-300/20 bg-black/30 text-center">
          <MonitorPlay className="mb-4 text-cyan-200/70" size={42} />
          <h3 className="text-xl font-semibold text-white">{model.placeholderTitle}</h3>
          <p className="mt-3 max-w-xl text-sm text-zinc-400">{model.placeholderBody}</p>
        </div>
      </section>
      {model.permissionNotice ? (
        <section className="rounded-2xl border border-amber-300/20 bg-amber-400/5 p-5">
          <div className="flex items-start gap-3">
            <ShieldAlert className="mt-0.5 text-amber-200" size={20} />
            <div className="space-y-3">
              <div>
                <h3 className="text-base font-semibold text-white">
                  {model.permissionNotice.title}
                </h3>
                <p className="mt-1 text-sm text-zinc-300">{model.permissionNotice.body}</p>
              </div>
              <ul className="list-disc space-y-1 pl-5 text-sm text-zinc-400">
                {model.permissionNotice.bullets.map((bullet) => (
                  <li key={bullet}>{bullet}</li>
                ))}
              </ul>
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  className="rounded-md border border-amber-300/30 bg-amber-300/10 px-3 py-2 text-sm text-amber-100"
                >
                  {model.permissionNotice.ctaPrimary}
                </button>
                <button
                  type="button"
                  className="rounded-md border border-white/10 bg-white/5 px-3 py-2 text-sm text-zinc-200"
                >
                  {model.permissionNotice.ctaSecondary}
                </button>
              </div>
            </div>
          </div>
        </section>
      ) : null}
    </div>
  );
}

export function SessionPanel({
  summary,
  allowFiles,
}: {
  summary: RemoteSessionSummary | null;
  allowFiles?: boolean;
}) {
  return (
    <div className="space-y-5">
      <section className="rounded-xl border border-white/10 bg-white/5 p-4">
        <h2 className="text-sm font-semibold uppercase tracking-[0.2em] text-zinc-400">
          会话概览
        </h2>
        {summary ? (
          <div className="mt-4 space-y-4 text-sm text-zinc-300">
            <div>
              <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">状态</div>
              <div className="mt-1 text-lg font-semibold text-white">{summary.state}</div>
            </div>
            {summary.latestFrame ? (
              <div>
                <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">画面</div>
                <div className="mt-1">
                  {summary.latestFrame.width} × {summary.latestFrame.height} · frame #
                  {summary.latestFrame.sequence}
                </div>
              </div>
            ) : null}
            {summary.clipboard ? (
              <div>
                <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">剪贴板</div>
                <div className="mt-1">{summary.clipboard.content}</div>
              </div>
            ) : null}
            {allowFiles && summary.pendingTransfer ? (
              <div>
                <div className="mb-2 flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-zinc-500">
                  <PlugZap size={14} />
                  文件传输
                </div>
                <div className="rounded-lg border border-white/10 bg-black/20 p-3">
                  <div className="font-medium text-white">
                    {summary.pendingTransfer.fileName}
                  </div>
                  <div className="mt-1 text-xs text-zinc-400">
                    {summary.pendingTransfer.totalBytes} bytes · chunk{" "}
                    {summary.pendingTransfer.chunkIndex + 1}/
                    {summary.pendingTransfer.chunkCount}
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        ) : (
          <p className="mt-4 text-sm text-zinc-400">当前没有可展示的会话摘要。</p>
        )}
      </section>
    </div>
  );
}
