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
