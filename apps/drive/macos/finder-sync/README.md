# AIO Drive Finder Sync

这是 AIO Drive 的原生 macOS Finder 扩展。

它提供三类能力：

- Finder 右键菜单 `AIO Drive 托管 / 取消托管`
- 当前登录用户托管文件的绿色图标
- 融合来的别人的托管文件的蓝色图标

真正的托管 / 取消托管逻辑并不在扩展里实现，而是委托给同仓库里的 `az-drive-app` CLI。这样 Finder、CLI、daemon 走的是同一套 Drive 能力。

## 当前行为

- 监听目录：`$HOME` 和 `/Volumes`
- 禁止从 Finder 直接托管整个家目录
- 右键菜单只会在真实状态下显示 `托管` 或 `取消托管`
- 托管完成后会按真实本地状态重新计算图标，不再盲目刷成功徽标

运行日志写入：

```text
~/Library/Logs/az-drive-finder-sync.log
```

## 安装

在仓库根目录执行：

```bash
apps/drive/macos/finder-sync/install.sh
```

脚本会：

- 编译 `az-drive-app`
- 编译 Finder App 与 `.appex`
- 重新签名
- 安装到 `/Applications/AIO Drive Finder.app`
- 重新注册 Finder 扩展
- 安装 Finder Quick Actions 作为右键兜底入口
- 重启 Finder

如果是下载 `.dmg` 安装：

- 先把 `AIO Drive Finder.app` 拖进 `Applications`
- 再手动打开一次 app
- app 会自动注册 Finder 扩展、安装 Quick Actions，并给出系统设置引导
- 完全磁盘访问不会自动弹窗，托管受保护路径时需要手动去系统设置里添加

## 调试

常见排查入口：

- 查看日志：`tail -f ~/Library/Logs/az-drive-finder-sync.log`
- 查看本地状态：`aio drive ls`
- 查看单文件状态：`aio drive status <path>`

如果 Finder 看起来和 CLI 状态不一致，优先检查：

- 本地 `state.json` 是否真的包含该文件
- daemon 是否正在运行
- 最近一次 Finder 扩展是否已经重装
