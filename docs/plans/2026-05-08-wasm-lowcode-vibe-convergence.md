# `aio` Wasm Plugin / Lowcode / Vibe Convergence

## 1. Conclusion

可以收敛，而且应该收敛。

但正确的收敛目标不是“再加一个 AI 按钮”，而是把当前三条线统一成一套页面模型：

- `WASM plugin` 负责交付页面能力包
- `lowcode canvas` 负责页面结构、拖拽布局、事件绑定
- `vibe coding` 负责生成和修改组件定义、页面树、动作脚本

最终目标应是：

- 页面以 `canvas document` 作为正式描述
- 组件分成两类：
  - `host built-in`：平台内建、拖拽即用、受控 props
  - `plugin generated`：由在线 vibe 生成，并被注册为新的组件或页面片段
- `button` 不是特殊入口，而是普通组件；它的 `onClick` 行为可以绑定：
  - 固定动作
  - HTTP / DB / event action
  - AI 生成的脚本动作
  - 打开“在线 vibe 编辑器”继续生成组件或动作

## 2. Existing Pieces

仓库里已经有可复用的三块基础：

- `crates/runtime/plugin-contract/src/lib.rs`
  这里定义了插件页面、菜单、marketplace、shell snapshot。当前的 `PageSchema` 还是结果页 DSL，偏展示，不够承载画布编辑。
- `crates/apps/lowcode/src/schema.rs`
  这里已经有 `LayoutSchema`、`ComponentNode`、`GridArea`、`EventBindingRecord`。这其实已经是画布页的核心文档模型。
- `apps/aio/backend/src/services/vibe_coding.rs`
  这里已经有在线启动 coding session 的后端入口。它现在是“开一个终端会话”，还不是“围绕画布节点做定向生成”。

这说明不需要再造第四套模型，应该直接以 `az-lowcode` 为页面结构核心。

## 3. Target Shape

建议把插件页面收敛成两层：

1. `PluginDescriptor`
   负责插件元数据、菜单、能力声明、页面入口。
2. `PluginPageSource`
   负责页面的真实内容来源。

`PluginPageSource` 建议分成三种：

- `builtin_schema`
  保留当前 `table/form/detail/board/markdown/graph`，用于简单展示型页面
- `canvas_document`
  指向 `LayoutSchema + EventBinding + component registry snapshot`
- `remote_runtime`
  预留给更重型的 wasm 前端运行时页面

这样做的原因：

- 简单展示页不需要强行进画布
- 真正需要拖拽、编排、AI 生成的页面统一走 `canvas_document`
- 未来如果某些 wasm 插件要自己带更复杂 runtime，也不阻塞

## 4. Component Model

组件体系应明确拆成三层：

- `built-in component`
  平台写死组件。比如按钮、输入框、表格、tabs、badge、统计卡、树、文件列表。
- `generated component`
  由 vibe 生成的受控组件定义。不是任意 React 代码直接进生产，而是先落为平台可校验的组件包。
- `composed block`
  多个组件组合成一个可复用 block，例如“搜索表格工作台”“审批详情头部”“笔记编辑区”。

`button` 这类控件本身不该是“AI 按钮”，而应该支持可配置 action：

- `navigate`
- `submit_form`
- `open_dialog`
- `emit_event`
- `run_http`
- `run_script`
- `open_vibe_task`

其中 `open_vibe_task` 才是你说的“在线 vibe coding 的那种按钮”。

也就是说：

- 按钮本身还是写死受控组件
- 但它可以打开一个带上下文的 vibe session
- 这个 vibe session 可以针对当前页面、当前节点、当前 selection 生成代码或 schema

## 5. Recommended Architecture

### ADR-001: `LayoutSchema` becomes the primary editable page document

- Status: Proposed
- Decision:
  对所有需要拖拽编辑的插件页面，正式页面结构统一使用 `az-lowcode::LayoutSchema`
- Consequences:
  `az-plugin-contract` 需要增加 `canvas_document` 页类型，现有展示型 schema 作为轻量快捷页保留

### ADR-002: Vibe output must target schema first, code second

- Status: Proposed
- Decision:
  在线 vibe 默认先生成：
  - `ComponentNode.props`
  - `EventBinding`
  - `LayoutSchema` subtree
  - 受控组件模板
  而不是直接生成任意散装前端代码
- Consequences:
  输出更可校验、可回滚、可 diff、可权限控制

### ADR-003: Generated components must be packaged as plugin assets

- Status: Proposed
- Decision:
  vibe 生成出来的新组件，不直接混入 host 源码，而是进入插件自己的组件注册表或开发态 plugin bundle
- Consequences:
  平台核心壳不会因为一次 AI 生成就被污染；组件归属、版本、回滚都清楚

## 6. Product Flow

建议用户操作流收敛成这样：

1. 在插件市场安装一个业务插件
2. 打开插件页面时，页面实际是一个 `canvas document`
3. 左侧是组件面板：
   - 平台写死组件
   - 当前插件自带组件
   - 当前插件历史生成组件
4. 中间是画布拖拽区
5. 右侧是 props / event / data binding 面板
6. 顶部有两个 AI 入口：
   - `Generate Block`
   - `Generate Component`
7. 每个组件也有局部入口：
   - `AI edit props`
   - `AI rewrite action`
   - `AI turn selection into reusable component`

这就把“拖拽写死组件 + vibe 出来的组件”统一到一个工作流里了。

## 7. What Should Not Be Done

下面这些做法看起来快，但会把系统做散：

- 不要把在线 vibe 只做成 console 页里一个孤立终端
- 不要让 vibe 直接改 host 前端源码作为主路径
- 不要把 wasm 插件页继续长期限定在 `table/form/detail/board/markdown/graph`
- 不要让“AI 按钮”成为一个特殊 hardcode 分支；它应该是 action system 的一个动作类型
- 不要同时维护“插件 DSL 页面”和“低代码页面”两套正式页面模型

## 8. Minimal Migration Plan

### Phase 1: Contract convergence

- 在 `az-plugin-contract` 中新增 `canvas_document`
- `ResolvedPage` 返回页面来源和编辑能力元数据
- 保留现有 `PageSchema` 以兼容旧页面

### Phase 2: Canvas runtime page

- 在 `aio-front` 增加 `plugin canvas renderer/editor`
- 先只支持 built-in components
- 先不开放任意生成组件代码

### Phase 3: Action system

- 把 `EventBindingRecord::HandlerType` 扩成真正可用于前端的 action model
- 增加 `open_vibe_task`
- 按钮、卡片、列表项统一复用这套 action binding

### Phase 4: Vibe-assisted generation

- 让 vibe session 接收更强上下文：
  - current page schema
  - selected node
  - available component registry
  - target slot / target grid area
- 输出目标先限定为：
  - subtree schema
  - props patch
  - action patch

### Phase 5: Generated component packaging

- 允许把稳定的 AI 生成 block 升级成插件内可复用组件
- 生成结果进入插件资产目录，再由 wasm plugin runtime 注册

## 9. Immediate Next Step

如果按这个方向推进，下一步最值钱的不是继续改市场页，而是：

1. 先给 `az-plugin-contract::PageSchema` 增加 `canvas_document`
2. 把 `az-lowcode::LayoutSchema` 作为插件页载荷接进去
3. 在 `aio-front` 先做一个“只读 canvas renderer + 右侧 props 面板”
4. 再加一个“对选中节点启动 vibe session”的入口

这样改动顺序最稳，也最容易验证。
