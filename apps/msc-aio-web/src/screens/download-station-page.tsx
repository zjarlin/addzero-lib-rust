"use client";

import { useEffect, useState } from "react";

import type { StorageBrowseResultDto } from "@addzero/api-client";

import { api } from "@/lib/api";
import { bytesLabel, timeLabel } from "@/lib/utils";
import { Button, Callout, Card, DataTable, Field, Input, PageHeader, SectionTitle } from "@/components/ui";

async function fileToBytes(file: File) {
  return Array.from(new Uint8Array(await file.arrayBuffer()));
}

export function DownloadStationPage() {
  const [result, setResult] = useState<StorageBrowseResultDto | null>(null);
  const [prefix, setPrefix] = useState("");
  const [newFolder, setNewFolder] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);

  const browse = async (nextPrefix = prefix) => {
    try {
      const nextResult = await api.browseFiles({ prefix: nextPrefix });
      setResult(nextResult);
      setPrefix(nextResult.current_prefix);
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void browse("");
  }, []);

  return (
    <div className="space-y-6">
      <PageHeader
        title="下载站"
        subtitle="浏览、上传、分享和删除都通过 `/api/admin/storage/files/*`，前端不扫描本地目录。"
        actions={
          <div className="flex flex-wrap gap-2">
            <Button onClick={() => void browse("")}>根目录</Button>
            <Button
              onClick={() => {
                if (result?.parent_prefix !== undefined && result?.parent_prefix !== null) {
                  void browse(result.parent_prefix);
                }
              }}
              disabled={!result?.parent_prefix}
            >
              上一级
            </Button>
          </div>
        }
      />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <Card>
        <SectionTitle
          title={result?.bucket ?? "msc-aio"}
          detail={result ? `当前前缀：${result.current_prefix || "/"}` : "正在连接 MinIO…"}
        />
        <div className="mb-4 grid gap-4 md:grid-cols-[1fr_auto]">
          <Input
            data-command-search="true"
            value={prefix}
            onChange={(event) => setPrefix(event.target.value)}
            placeholder="assets/installers/"
          />
          <Button onClick={() => void browse(prefix)}>浏览</Button>
        </div>
        <div className="grid gap-4 md:grid-cols-[1fr_auto]">
          <Input
            value={newFolder}
            onChange={(event) => setNewFolder(event.target.value)}
            placeholder="新增目录名"
          />
          <Button
            onClick={async () => {
              try {
                await api.createFolder({ parent_prefix: prefix, relative_path: newFolder });
                setNewFolder("");
                await browse(prefix);
              } catch (error) {
                setFeedback(error instanceof Error ? error.message : "创建失败");
              }
            }}
          >
            新建目录
          </Button>
        </div>
      </Card>

      <Card>
        <SectionTitle title="上传文件" />
        <input
          type="file"
          multiple
          onChange={async (event) => {
            const files = Array.from(event.target.files ?? []);
            if (!files.length) return;
            try {
              const payloadFiles = await Promise.all(
                files.map(async (file) => ({
                  file_name: file.name,
                  content_type: file.type || null,
                  bytes: await fileToBytes(file),
                })),
              );
              await api.uploadFiles({ prefix, files: payloadFiles });
              await browse(prefix);
            } catch (error) {
              setFeedback(error instanceof Error ? error.message : "上传失败");
            }
          }}
        />
      </Card>

      <Card>
        <SectionTitle title="目录" detail={`${result?.folders.length ?? 0} 个子目录`} />
        <DataTable
          columns={["Name", "Prefix", "Objects", "Size", "Action"]}
          rows={(result?.folders ?? []).map((folder) => [
            folder.name,
            folder.prefix,
            String(folder.object_count),
            bytesLabel(folder.size_bytes),
            <div key={folder.prefix} className="flex gap-2">
              <Button onClick={() => void browse(folder.prefix)}>打开</Button>
              <Button
                tone="danger"
                onClick={async () => {
                  try {
                    await api.deleteFolder({ prefix: folder.prefix });
                    await browse(prefix);
                  } catch (error) {
                    setFeedback(error instanceof Error ? error.message : "删除失败");
                  }
                }}
              >
                删除
              </Button>
            </div>,
          ])}
        />
      </Card>

      <Card>
        <SectionTitle title="文件" detail={`${result?.files.length ?? 0} 个对象`} />
        <DataTable
          columns={["Name", "Type", "Size", "Updated", "Action"]}
          rows={(result?.files ?? []).map((file) => [
            file.name,
            file.content_type,
            bytesLabel(file.size_bytes),
            timeLabel(file.last_modified),
            <div key={file.object_key} className="flex gap-2">
              <Button
                onClick={async () => {
                  try {
                    const shared = await api.shareFile({
                      object_key: file.object_key,
                      expiration_seconds: 24 * 3600,
                    });
                    window.open(shared.presigned_url, "_blank");
                  } catch (error) {
                    setFeedback(error instanceof Error ? error.message : "分享失败");
                  }
                }}
              >
                分享
              </Button>
              <Button
                tone="danger"
                onClick={async () => {
                  try {
                    await api.deleteFile({ object_key: file.object_key });
                    await browse(prefix);
                  } catch (error) {
                    setFeedback(error instanceof Error ? error.message : "删除失败");
                  }
                }}
              >
                删除
              </Button>
            </div>,
          ])}
        />
      </Card>
    </div>
  );
}
