"use client";

import { useEffect, useState } from "react";

import type { BrandingSettingsDto, BrandingSettingsUpdate, StoredLogoDto } from "@addzero/api-client";

import { api } from "@/lib/api";
import { Button, Callout, Card, Field, Input, PageHeader, SectionTitle, Select, Textarea } from "@/components/ui";

async function fileToBytes(file: File) {
  return Array.from(new Uint8Array(await file.arrayBuffer()));
}

export function SettingsPage() {
  const [settings, setSettings] = useState<BrandingSettingsUpdate>({
    site_name: "MSC_AIO",
    logo_source: "app_icon",
    logo: null,
    brand_copy: "",
    header_badge: "",
  });
  const [storedLogo, setStoredLogo] = useState<StoredLogoDto | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    void api
      .getBrandingSettings()
      .then((value: BrandingSettingsDto) => {
        setSettings({
          site_name: value.site_name,
          logo_source: value.logo_source,
          logo: value.logo,
          brand_copy: value.brand_copy,
          header_badge: value.header_badge,
        });
        setStoredLogo(value.logo);
      })
      .catch((error) => setFeedback(error instanceof Error ? error.message : "加载失败"));
  }, []);

  return (
    <div className="space-y-6">
      <PageHeader
        title="系统设置"
        subtitle="系统设置先把品牌配置正式接到 REST；安全、默认值和存储信息保留为显式面板。"
      />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <div className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr]">
        <Card>
          <SectionTitle title="品牌与站点" actions={<Button tone="accent" onClick={async () => {
            const saved = await api.saveBrandingSettings({ ...settings, logo: storedLogo });
            setStoredLogo(saved.logo);
            setFeedback("已保存品牌设置");
          }}>保存</Button>} />
          <div className="grid gap-4 md:grid-cols-2">
            <Field label="site_name">
              <Input value={settings.site_name} onChange={(event) => setSettings({ ...settings, site_name: event.target.value })} />
            </Field>
            <Field label="logo_source">
              <Select
                value={settings.logo_source}
                onChange={(event) => setSettings({ ...settings, logo_source: event.target.value as BrandingSettingsUpdate["logo_source"] })}
              >
                <option value="app_icon">app_icon</option>
                <option value="custom_upload">custom_upload</option>
                <option value="text_only">text_only</option>
              </Select>
            </Field>
          </div>
          <div className="mt-4 grid gap-4">
            <Field label="brand_copy">
              <Textarea rows={5} value={settings.brand_copy} onChange={(event) => setSettings({ ...settings, brand_copy: event.target.value })} />
            </Field>
            <Field label="header_badge">
              <Input value={settings.header_badge} onChange={(event) => setSettings({ ...settings, header_badge: event.target.value })} />
            </Field>
            <Field label="upload_logo" detail={storedLogo ? storedLogo.relative_path : "未上传"}>
              <input
                type="file"
                accept="image/*"
                onChange={async (event) => {
                  const file = event.target.files?.[0];
                  if (!file) return;
                  const uploaded = await api.uploadLogo({
                    file_name: file.name,
                    content_type: file.type || null,
                    bytes: await fileToBytes(file),
                  });
                  setStoredLogo(uploaded);
                  setSettings((current) => ({ ...current, logo_source: "custom_upload" }));
                }}
              />
            </Field>
          </div>
        </Card>
        <div className="space-y-6">
          <Card>
            <SectionTitle title="安全与认证" />
            <ul className="space-y-2 text-sm text-zinc-300">
              <li>认证仍由 Axum session cookie 负责。</li>
              <li>前端不在 Next 侧读取服务端 cookie，也不引入 BFF。</li>
              <li>桌面端通过 Tauri localhost window 访问同一 HTTP 合约。</li>
            </ul>
          </Card>
          <Card>
            <SectionTitle title="默认值" />
            <ul className="space-y-2 text-sm text-zinc-300">
              <li>主页默认仍指向 `/dashboard`。</li>
              <li>命令搜索入口统一响应 `Cmd/Ctrl + K`。</li>
              <li>主轴 domain 与侧轴 menu 均由 provider 填充。</li>
            </ul>
          </Card>
          <Card>
            <SectionTitle title="对象存储" />
            <ul className="space-y-2 text-sm text-zinc-300">
              <li>浏览器工作台只依赖 `/api/admin/storage/files/*`。</li>
              <li>文件上传按 base64 bytes DTO 走现有 MinIO service。</li>
            </ul>
          </Card>
        </div>
      </div>
    </div>
  );
}
