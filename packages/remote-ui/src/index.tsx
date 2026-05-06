"use client";

import { MonitorPlay, PlugZap, ShieldAlert } from "lucide-react";
import type { ReactNode } from "react";
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Separator,
  cn,
} from "@addzero/ui";

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
    <div className="dark min-h-screen bg-background text-foreground">
      <div className="mx-auto grid min-h-screen max-w-[1700px] gap-4 px-4 py-4 xl:grid-cols-[18rem_minmax(0,1fr)_22rem]">
        <Card className="border-primary/20 bg-card/80 shadow-none backdrop-blur">
          <CardHeader className="pb-4">
            <p className="text-xs uppercase tracking-[0.28em] text-muted-foreground">
              Addzero Remote
            </p>
            <CardTitle className="mt-2 text-2xl">{title}</CardTitle>
          </CardHeader>
          <CardContent>{sidebar}</CardContent>
        </Card>
        <Card className="min-w-0 bg-card/90 shadow-none">
          <CardContent className="p-4 md:p-6">
          {stage}
          </CardContent>
        </Card>
        <Card className="bg-card/80 shadow-none">
          <CardContent className="p-4">
          {detail}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export function DeviceCard({ device, active }: { device: RemoteDevice; active?: boolean }) {
  return (
    <Card
      className={cn(
        "transition shadow-none",
        active
          ? "border-primary/40 bg-primary/10"
          : "bg-card/70 hover:bg-accent/50",
      )}
    >
      <CardContent className="p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-semibold">{device.name}</div>
          <div className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
            {device.platform} · {device.role}
          </div>
        </div>
        <Badge
          variant={device.status.toLowerCase() === "online" ? "default" : "secondary"}
          className={cn(
            "rounded-full px-2 py-1 text-[11px] uppercase tracking-[0.18em]",
            device.status.toLowerCase() === "online"
              ? "bg-emerald-500/15 text-emerald-200"
              : "bg-muted text-muted-foreground",
          )}
        >
          {device.status}
        </Badge>
      </div>
      {device.notes ? (
        <p className="mt-3 text-sm text-muted-foreground">{device.notes}</p>
      ) : null}
      </CardContent>
    </Card>
  );
}

export function RemoteStage({ model }: { model: RemoteStageModel }) {
  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div>
          <h2 className="text-2xl font-semibold">{model.title}</h2>
          <p className="mt-2 max-w-2xl text-sm text-muted-foreground">{model.subtitle}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          {model.actions.map((action) => (
            <Button
              key={action.label}
              type="button"
              variant={
                action.tone === "danger"
                  ? "destructive"
                  : action.tone === "primary"
                    ? "default"
                    : "outline"
              }
            >
              {action.label}
            </Button>
          ))}
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        {model.statusChips.map((chip) => (
          <Badge
            key={chip.label}
            variant={chip.emphasis ? "default" : "secondary"}
            className={cn(
              "rounded-full px-3 py-1 text-xs uppercase tracking-[0.2em]",
              chip.emphasis
                ? "bg-primary/20 text-primary-foreground"
                : "bg-muted text-muted-foreground",
            )}
          >
            {chip.label}
          </Badge>
        ))}
      </div>
      <Card className="overflow-hidden border-primary/20 bg-gradient-to-br from-primary/10 via-card to-muted/30 shadow-none">
        <CardContent className="p-8">
        <div className="flex min-h-[28rem] flex-col items-center justify-center rounded-2xl border border-dashed border-primary/20 bg-background/70 text-center">
          <MonitorPlay className="mb-4 text-primary/70" size={42} />
          <h3 className="text-xl font-semibold">{model.placeholderTitle}</h3>
          <p className="mt-3 max-w-xl text-sm text-muted-foreground">{model.placeholderBody}</p>
        </div>
        </CardContent>
      </Card>
      {model.permissionNotice ? (
        <Card className="border-amber-500/30 bg-amber-500/10 shadow-none">
          <CardContent className="p-5">
          <div className="flex items-start gap-3">
            <ShieldAlert className="mt-0.5 text-amber-300" size={20} />
            <div className="space-y-3">
              <div>
                <h3 className="text-base font-semibold">
                  {model.permissionNotice.title}
                </h3>
                <p className="mt-1 text-sm text-muted-foreground">{model.permissionNotice.body}</p>
              </div>
              <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
                {model.permissionNotice.bullets.map((bullet) => (
                  <li key={bullet}>{bullet}</li>
                ))}
              </ul>
              <div className="flex flex-wrap gap-2">
                <Button type="button" variant="outline" className="border-amber-400/40 bg-amber-400/10 text-amber-100 hover:bg-amber-400/20 hover:text-amber-50">
                  {model.permissionNotice.ctaPrimary}
                </Button>
                <Button type="button" variant="outline">
                  {model.permissionNotice.ctaSecondary}
                </Button>
              </div>
            </div>
          </div>
          </CardContent>
        </Card>
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
      <Card className="shadow-none">
        <CardHeader className="pb-4">
        <h2 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
          会话概览
        </h2>
        </CardHeader>
        <CardContent>
        {summary ? (
          <div className="space-y-4 text-sm">
            <div>
              <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">状态</div>
              <div className="mt-1 text-lg font-semibold">{summary.state}</div>
            </div>
            {summary.latestFrame ? (
              <div>
                <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">画面</div>
                <div className="mt-1">
                  {summary.latestFrame.width} × {summary.latestFrame.height} · frame #
                  {summary.latestFrame.sequence}
                </div>
              </div>
            ) : null}
            {summary.clipboard ? (
              <div>
                <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">剪贴板</div>
                <div className="mt-1">{summary.clipboard.content}</div>
              </div>
            ) : null}
            {allowFiles && summary.pendingTransfer ? (
              <div>
                <Separator className="mb-4" />
                <div className="mb-2 flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
                  <PlugZap size={14} />
                  文件传输
                </div>
                <Card className="shadow-none">
                  <CardContent className="p-3">
                  <div className="font-medium">
                    {summary.pendingTransfer.fileName}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    {summary.pendingTransfer.totalBytes} bytes · chunk{" "}
                    {summary.pendingTransfer.chunkIndex + 1}/
                    {summary.pendingTransfer.chunkCount}
                  </div>
                  </CardContent>
                </Card>
              </div>
            ) : null}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">当前没有可展示的会话摘要。</p>
        )}
        </CardContent>
      </Card>
    </div>
  );
}
