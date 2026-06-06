# AZ AIO Sync Plugin Requirements

## 背景

目标是在 `apps/az-aio/plugins` 里实现一个对标坚果云、OneDrive 的文件同步插件。同步必须支持两台或多台设备之间的低延迟文件内容同步，文本文件采用行级 CRDT 合并，不能用简单的“最后写入覆盖”。

当前事实：

- 已存在 Rust CRDT 库：`crates/text/az-line-crdt`。
- 已新增插件核心 crate：`apps/az-aio/plugins/features/sync`。
- 当前 `features/sync` 已提供插件描述、API 合约、默认目录建模、CRDT 同步核心和 Finder 状态 JSON 生成。
- 当前还没有完整 daemon、真实文件监听、真实 WebSocket handler、远端部署服务或双机端到端同步。

## 代码位置

插件代码必须放在：

````text
/Users/zjarlin/aio/workspace/zjarlin/addzero-lib-rust/apps/az-aio/plugins
不要写到其他 workspace。尤其不要把插件实现误放到 `crates/apps`
已存在的 Finder Sync 参考实现：

```text
/Users/zjarlin/aio/workspace/zjarlin/addzero-lib-rust/apps/drive/macos/finder-sync
````

Finder 扩展当前只读取本地状态文件，不负责同步协议本身。

## 用户目标

最终要能做到：

1. 本机启动同步客户端。
2. 远端或另一台电脑启动同步客户端/服务端。
3. 默认同步目录为当前用户家目录下：

```text
~/az-sync
```

4. 用户无需手动配置即可同步默认目录。
5. 本机 `touch ~/az-sync/a.txt`，另一台机器能很快出现该文件。
6. 本机追加内容，另一台机器能很快看到变更。
7. 另一台机器删除 3 个字符，本机也能很快同步。
8. 已存在目录也要支持，例如两台机器都有：

```text
~/.agents/skills
```

9. 同步相对家目录的路径，而不是假设两台机器绝对路径相同。
10. 需要读取设备名和设备 home 目录，跨平台建模。
11. 同步速度要快，传输层优先 WebSocket。
12. macOS 上尽量支持 Finder 原生同步状态徽标；如果纯 jar 做不到就不做，但 Rust/native 插件应预留状态文件桥接。

## 核心设计要求

### 路径模型

同步路径必须以用户 home 为边界：

```text
device_name = 当前设备名
home_dir = 当前设备家目录
relative_path = 相对 home 的路径，例如 az-sync/a.txt 或 .agents/skills/foo/SKILL.md
local_path = home_dir + relative_path
```

禁止把 `/Users/zjarlin/...` 当作跨设备固定路径。两台设备用户名可能一样，也可能不一样。

### 默认目录

默认同步根：

```text
relative_path = az-sync
local_path = ~/az-sync
```

首次启动应自动创建或纳入该目录。用户不配置也能工作。

### 文本同步

文本文件必须通过 `az-line-crdt`：

- 初始导入：从本地文本生成 CRDT 文档。
- 本地全文变化：用行级更新 API，例如 `LineCrdtDocument::apply_text_by_line`。
- 精确字符变化：可用 `insert_text`、`delete_text`、`splice_text`。
- 传输内容：CRDT snapshot 或 incremental update。
- 远端导入：`import_snapshot` 或 `import_update`。
- 合并策略：CRDT 收敛，不能覆盖远端内容。

最低验收用例：

1. A 写入 `one\ntwo\nthree`。
2. B 收到后内容一致。
3. B 删除 `two` 三个字符。
4. A 收到后内容变为 `one\n\nthree`。
5. A、B 最终内容一致。

### 二进制文件

第一阶段可以先不做复杂二进制 CRDT。可按对象存储或整文件哈希同步设计：

- 小文件可整文件传输。
- 大文件要分块，避免把服务器内存打爆。
- MinIO 可作为大对象/分块内容存储。
- PostgreSQL 只存元数据、索引、版本、水位和设备状态。

### 冲突要求

文本 CRDT 不应产生传统冲突副本。并发编辑需要收敛。
不可接受：

- 简单 last-write-wins 覆盖文本文件。
- 每次全文上传导致远端覆盖本地未同步编辑。
- 绝对路径相同才同步。

## 传输与服务端

### 推荐角色

安装包不必分客户端/服务端。建议同一个二进制通过配置或启动参数区分：

```text
az-aio sync serve
az-aio sync agent
```

也可以默认 agent 连接远端 sync server。

### WebSocket

需要一个低延迟通道：

```text
GET /api/sync/ws
```

建议消息类型：

- `hello`：上报设备名、home、同步 roots。
- `update`：发送 CRDT snapshot/update。
- `ack`：确认某文件某版本已收到。
- `request-snapshot`：本地缺失基线时请求快照。
- `heartbeat`：保活和延迟检测。
- `error`：协议错误。

当前 `features/sync/src/contracts.rs` 已定义 `SyncWireMessage`，可继续扩展。

### HTTP API

当前插件已声明这些 API 合约，后续后端要真正实现：

```text
GET  /api/sync/status
GET  /api/sync/files
POST /api/sync/roots
POST /api/sync/files/apply-text
POST /api/sync/files/delete-text
POST /api/sync/files/import-update
GET  /api/sync/ws
GET  /api/sync/finder/status
POST /api/sync/finder/refresh
```

### 远端域名

用户希望后续把服务器上的 `端口` 通过 Cloudflare 反代为：

```text
https://sync.addzero.site
```

当前没有真实服务在远端 `端口` 运行。后续实现者必须部署并验证。

## 索引与存储

### 本地索引

本地必须维护同步索引，不能每次全量扫描大目录后阻塞：

- 文件相对路径
- 文件类型
- 本地 mtime / size
- 内容 hash
- CRDT snapshot/version
- 已发送/已确认版本水位
- 状态：synced/syncing/error/deleted/shared

本地索引建议放在：

```text
~/.config/addzero/sync/index.db
```

可用 SQLite 或轻量 KV。不要把索引塞到被同步目录里。

### 服务器索引

PostgreSQL 存：

- device
- sync_root
- file_record
- crdt_update_log 或版本水位
- object metadata
- session/connection

MinIO 存：

- 大文件分块
- 二进制文件对象
- 必要的 CRDT snapshot blob

### 大文件和大量文件

必须避免：

- 服务端一次加载全量文件内容。
- 每次扫描全库。
- 每个文件变化都立刻全量上传。
- 单 WebSocket 消息发送超大 blob。

要求：

- 文件监听事件合并和去抖。
- 大文件分块上传。
- 服务器端分页列文件。
- CRDT update 只传增量。
- 快照只在缺失基线或压缩日志时发送。

## 文件监听

本地 agent 需要监听同步 roots：

- macOS：FSEvents 或 `notify` crate。
- Linux：inotify 或 `notify` crate。
- Windows：ReadDirectoryChangesW 或 `notify` crate。

事件处理：

1. 监听创建、修改、删除、重命名。
2. 对短时间重复事件去抖。
3. 文件仍在写入时延迟读取。
4. 读取文件后计算 hash，hash 未变则不生成 update。
5. 文本文件进 CRDT，同步 update。
6. 二进制文件走对象同步。

## Finder 状态集成

现有 macOS Finder Sync 扩展读取状态文件候选：

```text
$AZ_DRIVE_STATE
~/Library/Application Support/addzero/drive/state.json
~/.config/addzero/drive/state.json
```

现有扩展关注的 JSON 字段：

```json
{
  "hosted": [
    {
      "local_path": "/Users/name/az-sync/a.txt",
      "space_id": "main",
      "root_alias": "default",
      "relative_path": "az-sync/a.txt"
    }
  ],
  "hosted_roots": [
    {
      "local_path": "/Users/name/az-sync",
      "space_id": "main",
      "root_alias": "default",
      "relative_path": "az-sync"
    }
  ]
}
```

当前 `features/sync/src/finder_status.rs` 已能生成兼容结构。

徽标 ID：

- `hosted`
- `shared`
- `busy`
- `error`

注意：现有 Finder 扩展主要根据 `hosted` / `hosted_roots` 判断绿色或共享状态。`busy` / `error` 当前主要是操作期间动态设置；如果要持久展示 busy/error，需要扩展 Objective-C 状态读取逻辑。

## 安全要求

最低要求：

- WebSocket 必须认证，不能裸连后随意写文件。
- 设备注册要有 token 或 key。
- 服务端必须按用户/空间隔离数据。
- 客户端只能写同步 root 内路径。
- 服务端不能接受 `../`、绝对路径或 home 外路径。
- 远程 update 必须绑定设备、空间、相对路径和版本。
- 日志不能打印敏感 token。

当前本地插件核心只做模型和 CRDT 逻辑，不代表传输安全已完成。

## 当前已实现内容

已新增：

```text
apps/az-aio/plugins/features/sync
```

关键文件：

- `src/descriptor.rs`：插件描述、UI contribution、backend API contribution、默认设置。
- `src/contracts.rs`：HTTP/WebSocket DTO。
- `src/sync_model.rs`：设备、home 相对路径、默认 root、文件状态、CRDT envelope。
- `src/sync_engine.rs`：内存版 CRDT 同步核心。
- `src/finder_status.rs`：Finder Sync 兼容状态 JSON。
- `src/error.rs`：错误类型。

已接入：

- `apps/az-aio/plugins/Cargo.toml`
- `apps/az-aio/plugins/host/Cargo.toml`
- `apps/az-aio/plugins/host/src/lib.rs`
- `xtask/src/main.rs`

已验证命令：

```bash
cargo test -p az-aio-plugin-sync
cargo test -p az-aio-plugin-host
cargo test -p xtask
cargo run -p xtask -- az-platform plugin build sync
cargo run -p xtask -- az-platform plugin build-wasm sync
```

`build-wasm` 首次失败是因为本机没装 `wasm32-unknown-unknown`，执行下面命令后通过：

```bash
rustup target add wasm32-unknown-unknown
```

## 未实现内容

后续会话需要继续实现：

1. 真正的 `az-aio` 后端路由，把插件声明的 API 变成可请求的 handler。
2. 本地 daemon/agent。
3. 文件监听。
4. WebSocket 客户端和服务端。
5. PostgreSQL schema。
6. MinIO 大文件/对象同步。
7. 本地索引持久化。
8. 服务端部署到 `~/DockerCompose/az-sync`。
9. Cloudflare 反代 `sync.addzero.site`。
10. 双机真实端到端测试。

## 端到端验收脚本思路

测试至少要覆盖两台机器，不能只跑单测。

A 机器：

```bash
mkdir -p ~/az-sync
echo 'one
two
three' > ~/az-sync/a.txt
```

B 机器等待同步后：

```bash
cat ~/az-sync/a.txt
```

应输出：

```text
one
two
three
```

B 机器删除 `two`：

```bash
python3 - <<'PY'
from pathlib import Path
p = Path.home() / "az-sync" / "a.txt"
s = p.read_text()
p.write_text(s.replace("two", "", 1))
PY
```

A 机器等待同步后：

```bash
cat ~/az-sync/a.txt
```

应输出：

```text
one

three
```

还要测试已存在目录：

```bash
az-aio sync root add ~/.agents/skills
```

两台机器都已有内容时，初次同步必须做 CRDT/import/index reconcile，不能直接覆盖任一侧。

## 交付标准

可以认为完成的条件：

1. 本机和远端都有进程运行。
2. 远端 `localhost:端口` 或 `sync.addzero.site` API 可访问。
3. `/api/sync/status` 显示至少两个设备。
4. 默认 `~/az-sync` 无配置同步。
5. touch、追加、删除 3 字符都能双向同步。
6. `.agents/skills` 这种已有目录可以添加为同步 root。
7. 文本并发修改最终收敛。
8. 大文件不会导致服务端内存暴涨。
9. Finder 状态文件生成后，macOS Finder 扩展能看到托管状态。
