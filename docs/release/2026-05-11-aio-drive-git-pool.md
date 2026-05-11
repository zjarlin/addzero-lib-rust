# AIO Drive Git Pool / Finder / Docs

这次发布包含三类交付：

## 1. Drive 后端与同步

- AIO Drive 默认后端切到 Git Pool
- 当前登录用户的 Drive 归属改为 API key owner 模型
- `aio drive ls`、`queue`、`conflict` 进入正式 CLI
- Git Pool 控制仓库和内容池仓库支持双向同步

## 2. Finder 集成修复

- 修复 Finder 右键托管后本地状态可能被 daemon 覆盖的问题
- `state.json` 写入增加跨进程锁
- Finder 扩展完成后按真实状态重算图标，不再盲刷绿色
- `DESIGN.md` 等新托管文件现在能稳定落本地状态并同步进 Git Pool

## 3. 文档与发布入口

- 新增 `apps/drive/backend/README.md`
- 补齐 `xiaoeyu.config.json`
- GitHub Pages 文档构建链恢复可用
- 小鳄鱼文档现在会收录 root README、crate README 和 app README

## 验证

- `cargo test -p az-drive-agent`
- `cargo build -p az-drive-app`
- `aio drive host /Users/zjarlin/DESIGN.md`
- `aio drive ls`

