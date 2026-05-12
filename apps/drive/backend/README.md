# AIO Drive

`AIO Drive` 把“本地文件 + Git 仓库池”包装成一个双向同步云盘。

当前默认后端已经不是 `PG + MinIO`，而是 **Git Pool**：

- 不要求独立业务服务器
- 不要求 PostgreSQL / MinIO 常驻
- 默认直接复用系统 `git`
- 可用 GitHub、Gitea、NAS bare repo、本地 bare repo 作为远端

旧版 `PG + MinIO` Drive 后端已经删除。`aio drive` / `az-drive-app` 只保留 Git Pool。

对用户暴露的主入口是 `aio drive ...`。Finder 右键菜单和状态图标只是这个 CLI 的图形外壳。

## 现在的模型

- `aio login` 成功后会为当前登录用户准备 `drive-default` API key。
- Drive 归属绑定 **API key owner 用户**，不再对外暴露 `space` 概念。
- 当前登录用户自己的托管文件显示绿色托管图标。
- 来自 `trusted_api_keys` 融合过来的别人的托管文件显示蓝色对勾图标。

## Git Pool 结构

默认本地根目录：

```text
~/.aio/drive-git-pool
```

目录职责：

- `control/`：索引、pool 列表、挂载信息、同步队列、冲突、`.aioignore`
- `pools/<name>/`：内容对象和 pool index

默认配置文件：

```text
~/.config/aio/drive.toml
```

## 常用命令

登录并准备当前用户 Drive 凭证：

```bash
aio reg --username <name> --password-stdin
aio login --username <name> --password-stdin
aio whoami
aio key list
```

初始化 Git Pool：

```bash
aio drive pool init --control-remote <control-repo-url>
aio drive pool add main <pool-repo-url>
aio drive backend status
```

如果希望 pool 容量打满后自动扩容到新的本地 bare repo：

```bash
aio drive pool init \
  --control-remote <control-repo-url> \
  --auto-pool-root ~/.aio/drive-auto-pools \
  --auto-pool-prefix auto
aio drive pool add main <pool-repo-url> --max-size 8gb
```

开启后，当现有可写 pool 的 `used_bytes + 新对象大小` 超过 `max_size_bytes`，Drive 会在
`auto_pool_root` 下自动创建类似 `auto-0001.git`、`auto-0002.git` 的 bare repo，并把新对象写入新仓库。

如果希望对象字节逐步迁移到 `gitdb` 的多 repo shard，而控制仓和同步队列还留在当前 Drive 元数据层：

```bash
aio drive pool init \
  --control-remote <control-repo-url> \
  --object-backend gitdb \
  --gitdb-object-root ~/.aio/drive-gitdb-objects \
  --gitdb-object-max-shard-size 8gb \
  --gitdb-object-shard-prefix shard
aio drive backend status
```

这条路径下：

- 新对象会进入 `gitdb` 管理的 `shard-0001`、`shard-0002` 等 Git repo
- `git_pool` 的内容池可以逐步停止新增
- Drive 元数据、冲突、挂起项、同步队列仍保留在当前控制仓，方便渐进迁移

托管与查看：

```bash
aio drive host ~/.agents/skills
aio drive host /Library/SomeFolder
aio drive unhost ~/.agents/skills/tmp
aio drive ls
aio drive ls --all
aio drive ls --ignored
aio drive ls --format json
```

同步与冲突：

```bash
aio drive sync
aio drive queue list
aio drive queue retry
aio drive conflict list
aio drive conflict resolve --id <conflict-id> --keep-local
```

## 跟踪规则

- 显式托管的文件会直接进入 `hosted`
- 托管父目录后，后续新增的子文件会在同步时自动发现
- `.gitignore` 会天然生效
- 子项单独取消托管时，会写入 `.aioignore` 形成跨设备黑名单
- 家目录路径统一折叠成 `$HOME/...`
- 非家目录绝对路径如 `/Library/...` 会保留绝对路径身份

## 队列与冲突

当前同步已经带有两套耐久化机制：

- `sync-queue`：上传、下载、远端物化任务会持久化，失败后可重试
- `conflicts` + `suspended`：冲突文件不会直接覆盖本地，会生成冲突副本并挂起

这意味着：

- daemon 重启后，队列状态不会丢
- 冲突不会静默吞掉
- 失败项可以单独重试，不需要整盘重扫

## Finder 集成

macOS Finder 扩展位于：

- [apps/drive/macos/finder-sync/README.md](/Users/zjarlin/workspace/addzero-lib-rust/apps/drive/macos/finder-sync/README.md)

它负责：

- 右键菜单 `AIO Drive 托管 / 取消托管`
- 绿色托管图标
- 蓝色融合图标
- 同步中 / 错误图标

实际托管逻辑仍由 `az-drive-app` / `aio drive` 执行。

## 文档入口

这个 README 会被仓库的小鳄鱼文档站自动收录。部署成功后，GitHub Pages 会把它和相关 README 聚合成一个静态文档站。

文档站构建配置见：

- [xiaoeyu.config.json](/Users/zjarlin/workspace/addzero-lib-rust/xiaoeyu.config.json)
- [docs/README.md](/Users/zjarlin/workspace/addzero-lib-rust/docs/README.md)

## 自定义短域名 / Tunnel

可以做，但推荐分两种情况看：

1. 静态文档站：
   优先用 **GitHub Pages 自定义域名**，不需要 tunnel，结构更简单。
2. 你已经有 `cloudflared tunnel`：
   可以在本机起一个很薄的反向代理，把 GitHub Pages 或本地生成的静态站挂到自定义域名上，再通过 tunnel 暴露。

不建议把 `github.com` 的登录页、仓库 UI、鉴权流量直接当主站去反代。那样链路复杂、缓存和头部策略难控，收益也不高。最稳的是：

- 文档内容做成静态站
- tunnel 只代理你自己的静态入口
