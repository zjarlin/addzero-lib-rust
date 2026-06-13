# az-aio-plugin-drive-center

桌面端文件托管中心插件，提供目录托管、同步、冲突检测与同步队列管理功能。

## 功能

- **驱动快照**：加载当前驱动状态快照，展示所有根目录、已托管路径、跟踪项、冲突项及同步队列
- **目录托管与取消托管**：支持将本地路径（如 `~/.agents/skills`）注册到驱动进行托管，也可随时取消托管
- **同步调度**：触发一次完整的驱动同步周期，将本地变更推送到远端并拉取远程更新
- **队列重试**：对同步队列中的失败项进行批量重试
- **冲突展示**：列出当前活跃的文件冲突，包含冲突路径与发生时间
- **桌面插件注册**：通过 `az-desktop-plugin` + `az-desktop-plugin-registry` 自动注册为桌面端 Operations 域的页面

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-aio-plugin-drive-center = { path = "../az-aio-plugin-drive-center" }       # workspace 内部引用
# 或发布后：
# az-aio-plugin-drive-center = "0.1"                                          # crates.io 引用
```

## 用法

```rust,ignore
// 该 crate 作为桌面插件自动发现并注册，无需手动调用。
// 在桌面 shell 中，Drive Center 页面位于 Operations > Storage 下，
// 通过 "/drive" 路由访问。
//
// 所有操作通过插件 toolbar 按钮触发：
// - Refresh：重新加载驱动快照
// - Sync：执行一次同步周期
// - Retry Queue：重试队列中的失败项
// - Host Skills：托管 ~/.agents/skills
// - Unhost Skills：取消托管 ~/.agents/skills
```

## 依赖的 crates

- `az-desktop-plugin` — 桌面端插件 trait、宿主服务接口与上下文类型
- `az-desktop-plugin-registry` — 插件注册清单，通过 inventory 自动发现
- `gpui` — Zed 生态 UI 框架，用于渲染驱动面板