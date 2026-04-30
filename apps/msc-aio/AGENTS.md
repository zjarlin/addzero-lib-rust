# AGENTS.md

## Local Admin Convention

- 当前 `dioxus-admin` 默认遵循仓库根 `AGENTS.md` 里的 admin shell、navigation、data 约定。
- 导航模型采用 `双轴上下文（Bi-Axial Context）`：
- 顶栏承载 `主轴上下文树（domain axis）`
- 左栏承载 `侧轴上下文树（module axis）`
- 内容区渲染 `(主轴节点, 侧轴节点)` 的二维交点
- 不把顶栏等同于页面 tab，也不把左栏等同于全局路由全集；左栏应只显示当前主轴下的子树。
- provider 术语优先使用 `domain`、`context axis`、`context tree`，避免继续把泛化分组叫成 `scene`。
- 对知识库场景，默认子树文案使用 `笔记`、`软件`、`安装包`。
