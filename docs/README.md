# 文档目录

`docs/` 现在只保留纯文档内容，不再内置 Node、Docusaurus、pnpm 或前端构建链。

当前约定：

- `docs/plans/*.md`：规划与设计文档
- `docs/prototypes/*.html`：可直接打开的原型文件
- `docs/*.md`：补充说明

使用方式：

- Markdown 直接阅读
- HTML 原型直接用浏览器打开
- 如果后续要重新接站点生成器，放到仓库外层工具链处理，不再把前端包管理器带回这个仓库

## GitHub Pages / 小鳄鱼

这个仓库的文档站通过 `.github/workflows/deploy-docs.yml` 构建，配置文件是根目录的：

- [xiaoeyu.config.json](/Users/zjarlin/workspace/addzero-lib-rust/xiaoeyu.config.json)

当前会自动收录：

- 根目录 `README.md`
- `crates/**/README.md`
- `apps/**/README.md`

## 巡检文档

- [crates 中文注释巡检](./crates-chinese-comment-audit.md)

这意味着 AIO Drive 的 CLI、Git Pool、Finder 文档会直接进入文档站搜索入口，而不需要单独维护一套手写 sidebar。

## 自定义域名 / Tunnel

如果只是想让 GitHub Pages 地址更好记，优先顺序建议是：

1. 直接给 GitHub Pages 配自定义域名
2. 如果你已经稳定使用 `cloudflared tunnel`，再把自定义域名入口代理到静态站

不建议把 `github.com` 仓库页面本身作为长期反代目标。文档适合静态化，仓库 UI 不适合做主入口。
