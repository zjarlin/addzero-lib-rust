"use client";

import {
  DeviceCard,
  RemoteShell,
  RemoteStage,
  SessionPanel,
  type RemoteDevice,
  type RemoteStageModel,
  type RemoteSessionSummary,
} from "@addzero/remote-ui";
import { useMemo, useState } from "react";

import { REMOTE_DESKTOP_API_BASE_URL } from "./lib/constants";

const devices: RemoteDevice[] = [
  {
    id: "relay-mac-mini",
    name: "Mac mini Relay",
    platform: "macOS",
    role: "viewer target",
    status: "online",
    notes: "当前行为级别保持与旧 mock 驱动界面一致，优先验证状态和桌面特权提示。",
  },
  {
    id: "studio-win",
    name: "Studio Windows",
    platform: "Windows",
    role: "operator target",
    status: "idle",
    notes: "作为后续真实 relay / host 控制面的占位入口。",
  },
  {
    id: "lab-linux",
    name: "Lab Linux",
    platform: "Linux",
    role: "headless host",
    status: "offline",
    notes: "展示 seeded 设备的列表结构、状态和备注层级。",
  },
];

const sessionByDevice: Record<string, RemoteSessionSummary> = {
  "relay-mac-mini": {
    state: "relay-ready",
    latestFrame: {
      width: 2560,
      height: 1440,
      sequence: 4821,
    },
    clipboard: {
      content: "ssh relay@edge-gw",
    },
    pendingTransfer: {
      fileName: "session-log-2026-05-03.txt",
      totalBytes: 73122,
      chunkIndex: 3,
      chunkCount: 7,
    },
  },
  "studio-win": {
    state: "awaiting-control",
    latestFrame: {
      width: 1920,
      height: 1080,
      sequence: 188,
    },
  },
  "lab-linux": {
    state: "offline",
  },
};

function buildStageModel(active: RemoteDevice, desktopMode: boolean): RemoteStageModel {
  return {
    title: active.name,
    subtitle:
      "React 版本先维持当前远程桌面 seeded/mock 行为级别，桌面端再通过 provider/config 暴露额外能力。",
    actions: [
      { label: "开始查看", tone: "primary" },
      { label: "刷新会话", tone: "neutral" },
      ...(desktopMode ? [{ label: "桌面权限向导", tone: "neutral" as const }] : []),
      { label: "断开会话", tone: "danger" },
    ],
    statusChips: [
      { label: active.status, emphasis: active.status === "online" },
      { label: desktopMode ? "desktop-mode" : "web-mode" },
      { label: `api ${REMOTE_DESKTOP_API_BASE_URL}` },
    ],
    placeholderTitle: "Remote Stage Placeholder",
    placeholderBody:
      "真实视频帧、输入注入和 relay 信令还没有接入前，这里先保留可验证的信息密度、布局和权限提示。",
    permissionNotice: desktopMode
      ? {
          title: "桌面能力通过统一壳子开放",
          body: "Tauri wrapper 不再分叉独立 UI，只通过配置暴露系统级权限、剪贴板和文件传输能力。",
          bullets: [
            "屏幕录制 / 辅助功能授权由桌面壳子触发。",
            "Loopback HTTP contract 与 Web 前端保持一致。",
            "设备权限提示不内嵌到布局层，而是由远程模块决定是否展示。",
          ],
          ctaPrimary: "打开权限设置",
          ctaSecondary: "查看能力矩阵",
        }
      : {
          title: "Web viewer 先保持现有限制",
          body: "浏览器端先只保留查看、状态检查和会话上下文，不假设额外本地特权。",
          bullets: [
            "不在 Web 端暴露桌面系统级快捷操作。",
            "保持与现有 mock 驱动行为相同的功能级别。",
            "后续若接入真实 control plane，再从 REST/WS 合同推进。",
          ],
          ctaPrimary: "查看接入计划",
          ctaSecondary: "浏览 API 合同",
        },
  };
}

export function RemoteDesktopPage() {
  const [activeId, setActiveId] = useState(devices[0]?.id ?? "");
  const [desktopMode, setDesktopMode] = useState(false);
  const active = useMemo(
    () => devices.find((device) => device.id === activeId) ?? devices[0],
    [activeId],
  );

  const stage = buildStageModel(active, desktopMode);
  const summary = sessionByDevice[active.id] ?? null;

  return (
    <RemoteShell
      title="Remote Desktop"
      sidebar={
        <div className="space-y-3">
          <button
            type="button"
            onClick={() => setDesktopMode((value) => !value)}
            className="w-full rounded-xl border border-cyan-300/20 bg-cyan-400/10 px-3 py-2 text-left text-sm text-cyan-50 transition hover:bg-cyan-400/15"
          >
            {desktopMode ? "切回 Web Viewer" : "切到 Desktop Wrapper"}
          </button>
          {devices.map((device) => (
            <button
              key={device.id}
              type="button"
              className="w-full text-left"
              onClick={() => setActiveId(device.id)}
            >
              <DeviceCard device={device} active={device.id === active.id} />
            </button>
          ))}
        </div>
      }
      stage={<RemoteStage model={stage} />}
      detail={<SessionPanel summary={summary} allowFiles={desktopMode} />}
    />
  );
}
