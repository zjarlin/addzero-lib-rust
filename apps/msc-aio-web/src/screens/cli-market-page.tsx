"use client";

import { useEffect, useMemo, useState } from "react";

import type {
  CliInstallMethod,
  CliMarketCatalog,
  CliMarketEntry,
  CliMarketEntryUpsert,
  CliMarketImportJob,
  CliMarketImportJobDetail,
} from "@addzero/api-client";

import { api } from "@/lib/api";
import { Button, Callout, Card, DataTable, Field, Input, PageHeader, SectionTitle, Textarea } from "@/components/ui";

function entryToUpsert(entry?: CliMarketEntry | null): CliMarketEntryUpsert {
  return {
    id: entry?.id ?? null,
    slug: entry?.slug ?? "",
    status: entry?.status ?? "draft",
    source_type: entry?.source_type ?? "manual",
    entry_kind: entry?.entry_kind ?? "cli",
    vendor_name: entry?.vendor_name ?? "",
    latest_version: entry?.latest_version ?? "",
    homepage_url: entry?.homepage_url ?? "",
    repo_url: entry?.repo_url ?? "",
    docs_url: entry?.docs_url ?? "",
    entry_point: entry?.entry_point ?? "",
    category_code: entry?.category_code ?? "",
    tags: entry?.tags ?? [],
    locales:
      entry?.locales ?? [
        {
          locale: "zh_cn",
          display_name: "",
          summary: "",
          description_md: "",
          install_guide_md: "",
          docs_summary: "",
          requires_text: "",
          install_command: "",
        },
        {
          locale: "en_us",
          display_name: "",
          summary: "",
          description_md: "",
          install_guide_md: "",
          docs_summary: "",
          requires_text: "",
          install_command: "",
        },
      ],
    install_methods:
      entry?.install_methods ?? [
        {
          id: null,
          platform: "cross_platform",
          installer_kind: "custom",
          package_id: "",
          command_template: "",
          validation_note: "",
          priority: 100,
        },
      ],
    doc_refs: entry?.doc_refs ?? [],
    raw: {},
  };
}

export function CliMarketRegistryPage() {
  const [catalog, setCatalog] = useState<CliMarketCatalog | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [form, setForm] = useState<CliMarketEntryUpsert>(entryToUpsert());
  const [feedback, setFeedback] = useState<string | null>(null);
  const [installResult, setInstallResult] = useState<string | null>(null);

  const refresh = async () => {
    try {
      const nextCatalog = await api.cliCatalog();
      setCatalog(nextCatalog);
      if (!selectedId && nextCatalog.entries[0]) {
        setSelectedId(nextCatalog.entries[0].id);
        setForm(entryToUpsert(nextCatalog.entries[0]));
      }
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const selectedEntry = useMemo(
    () => catalog?.entries.find((entry) => entry.id === selectedId) ?? null,
    [catalog, selectedId],
  );

  return (
    <div className="space-y-6">
      <PageHeader
        title="CLI 市场"
        subtitle="注册表页直接对接 CLI market REST 接口，保留目录、编辑、安装和发布动作。"
        actions={<Button onClick={() => setForm(entryToUpsert())}>新建条目</Button>}
      />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <div className="grid gap-6 xl:grid-cols-[24rem_minmax(0,1fr)]">
        <Card>
          <SectionTitle title="注册表目录" detail={`${catalog?.entries.length ?? 0} 条`} />
          <div className="space-y-2">
            {catalog?.entries.map((entry) => (
              <button
                key={entry.id}
                type="button"
                onClick={() => {
                  setSelectedId(entry.id);
                  setForm(entryToUpsert(entry));
                  setInstallResult(null);
                }}
                className={`w-full rounded-lg border px-3 py-3 text-left ${
                  entry.id === selectedId
                    ? "border-emerald-400/40 bg-emerald-500/10"
                    : "border-white/10 bg-black/20"
                }`}
              >
                <div className="font-medium text-white">{entry.slug}</div>
                <div className="mt-1 text-xs text-zinc-500">{entry.vendor_name || "未填写 vendor"}</div>
              </button>
            ))}
          </div>
        </Card>
        <Card>
          <SectionTitle
            title={selectedEntry ? `编辑 ${selectedEntry.slug}` : "新建条目"}
            actions={
              <div className="flex flex-wrap gap-2">
                <Button
                  tone="accent"
                  onClick={async () => {
                    try {
                      const saved = await api.cliUpsert(form);
                      setFeedback(`已保存 ${saved.slug}`);
                      await refresh();
                      setSelectedId(saved.id);
                    } catch (error) {
                      setFeedback(error instanceof Error ? error.message : "保存失败");
                    }
                  }}
                >
                  保存
                </Button>
                {selectedId ? (
                  <>
                    <Button
                      onClick={async () => {
                        try {
                          const result = await api.cliInstall(selectedId, {});
                          setInstallResult([result.command, result.stdout, result.stderr].filter(Boolean).join("\n\n"));
                        } catch (error) {
                          setFeedback(error instanceof Error ? error.message : "安装失败");
                        }
                      }}
                    >
                      安装
                    </Button>
                    <Button
                      onClick={async () => {
                        try {
                          await api.cliPublish(selectedId);
                          setFeedback("已发布");
                          await refresh();
                        } catch (error) {
                          setFeedback(error instanceof Error ? error.message : "发布失败");
                        }
                      }}
                    >
                      发布
                    </Button>
                    <Button
                      tone="danger"
                      onClick={async () => {
                        try {
                          await api.cliArchive(selectedId);
                          setFeedback("已归档");
                          await refresh();
                        } catch (error) {
                          setFeedback(error instanceof Error ? error.message : "归档失败");
                        }
                      }}
                    >
                      归档
                    </Button>
                  </>
                ) : null}
              </div>
            }
          />
          <div className="grid gap-4 md:grid-cols-2">
            <Field label="slug">
              <Input value={form.slug} onChange={(event) => setForm({ ...form, slug: event.target.value })} />
            </Field>
            <Field label="vendor_name">
              <Input
                value={form.vendor_name}
                onChange={(event) => setForm({ ...form, vendor_name: event.target.value })}
              />
            </Field>
            <Field label="latest_version">
              <Input
                value={form.latest_version}
                onChange={(event) =>
                  setForm({ ...form, latest_version: event.target.value })
                }
              />
            </Field>
            <Field label="tags">
              <Input
                value={form.tags.join(", ")}
                onChange={(event) =>
                  setForm({
                    ...form,
                    tags: event.target.value
                      .split(",")
                      .map((item) => item.trim())
                      .filter(Boolean),
                  })
                }
              />
            </Field>
          </div>
          <div className="mt-4 grid gap-4 md:grid-cols-2">
            <Field label="homepage_url">
              <Input
                value={form.homepage_url}
                onChange={(event) => setForm({ ...form, homepage_url: event.target.value })}
              />
            </Field>
            <Field label="repo_url">
              <Input
                value={form.repo_url}
                onChange={(event) => setForm({ ...form, repo_url: event.target.value })}
              />
            </Field>
          </div>
          <div className="mt-4 grid gap-4 xl:grid-cols-2">
            {form.locales.map((locale, index) => (
              <div key={locale.locale} className="space-y-3 rounded-lg border border-white/10 p-4">
                <div className="text-sm font-semibold text-white">{locale.locale}</div>
                <Field label="display_name">
                  <Input
                    value={locale.display_name}
                    onChange={(event) => {
                      const locales = [...form.locales];
                      locales[index] = { ...locale, display_name: event.target.value };
                      setForm({ ...form, locales });
                    }}
                  />
                </Field>
                <Field label="summary">
                  <Textarea
                    rows={4}
                    value={locale.summary}
                    onChange={(event) => {
                      const locales = [...form.locales];
                      locales[index] = { ...locale, summary: event.target.value };
                      setForm({ ...form, locales });
                    }}
                  />
                </Field>
              </div>
            ))}
          </div>
          <div className="mt-4">
            <Field label="install_command">
              <Textarea
                rows={4}
                value={form.install_methods[0]?.command_template ?? ""}
                onChange={(event) => {
                  const methods: CliInstallMethod[] = [...form.install_methods];
                  methods[0] = {
                    ...(methods[0] ?? {
                      id: null,
                      platform: "cross_platform",
                      installer_kind: "custom",
                      package_id: "",
                      validation_note: "",
                      priority: 100,
                    }),
                    command_template: event.target.value,
                  };
                  setForm({ ...form, install_methods: methods });
                }}
              />
            </Field>
          </div>
          {installResult ? (
            <div className="mt-4 rounded-lg border border-white/10 bg-black/20 p-4 text-sm whitespace-pre-wrap text-zinc-200">
              {installResult}
            </div>
          ) : null}
        </Card>
      </div>
    </div>
  );
}

async function readFileAsBase64(file: File) {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = "";
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return btoa(binary);
}

export function CliMarketImportsPage() {
  const [jobs, setJobs] = useState<CliMarketImportJob[]>([]);
  const [detail, setDetail] = useState<CliMarketImportJobDetail | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  const refresh = async () => {
    try {
      setJobs(await api.cliJobs());
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <div className="space-y-6">
      <PageHeader title="CLI 导入任务" subtitle="上传 JSON/XLSX 走现有 import job 链路。" />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <Card>
        <SectionTitle title="上传导入文件" />
        <input
          type="file"
          accept=".json,.xlsx"
          onChange={async (event) => {
            const file = event.target.files?.[0];
            if (!file) return;
            try {
              const bytes_b64 = await readFileAsBase64(file);
              const result = await api.cliImport({
                file_name: file.name,
                bytes_b64,
                format: file.name.endsWith(".xlsx") ? "xlsx" : "json",
                mode: "native",
              });
              setDetail(result);
              await refresh();
            } catch (error) {
              setFeedback(error instanceof Error ? error.message : "导入失败");
            }
          }}
        />
      </Card>
      <Card>
        <SectionTitle title="最近任务" detail={`${jobs.length} 条`} />
        <DataTable
          columns={["File", "Format", "Mode", "Status", "Action"]}
          rows={jobs.map((job) => [
            job.file_name,
            job.format,
            job.mode,
            job.status,
            <Button
              key={job.id}
              onClick={async () => {
                const nextDetail = await api.cliJobDetail(job.id);
                setDetail(nextDetail);
              }}
            >
              查看
            </Button>,
          ])}
        />
      </Card>
      {detail ? (
        <Card>
          <SectionTitle title={detail.file_name} detail={detail.status} />
          <pre className="whitespace-pre-wrap text-sm text-zinc-300">{detail.result_summary}</pre>
        </Card>
      ) : null}
    </div>
  );
}

export function CliMarketDocsPage() {
  const [catalog, setCatalog] = useState<CliMarketCatalog | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    void api
      .cliCatalog()
      .then(setCatalog)
      .catch((error) => setFeedback(error instanceof Error ? error.message : "加载失败"));
  }, []);

  return (
    <div className="space-y-6">
      <PageHeader
        title="CLI 文档"
        subtitle="文档页展示 doc refs 与公开注册表导出链接，保持 route parity。"
      />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <Card>
        <SectionTitle title="公开导出" />
        <div className="flex flex-wrap gap-2">
          <Button onClick={() => window.open("/api/cli-market/public/registry.json", "_blank")}>
            registry.json
          </Button>
          <Button onClick={() => window.open("/api/cli-market/public/registry.xlsx", "_blank")}>
            registry.xlsx
          </Button>
        </div>
      </Card>
      <Card>
        <SectionTitle title="文档索引" detail={`${catalog?.entries.length ?? 0} 条`} />
        <DataTable
          columns={["Slug", "Docs URL", "Doc Ref", "Summary"]}
          rows={(catalog?.entries ?? []).map((entry) => [
            entry.slug,
            entry.docs_url || "—",
            entry.doc_refs[0]?.url || "—",
            entry.locales[0]?.docs_summary || entry.locales[0]?.summary || "—",
          ])}
        />
      </Card>
    </div>
  );
}
