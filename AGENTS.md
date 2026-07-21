# AGENTS.md

## Admin Shell Convention

- 对 admin / workbench 骨架，保持“无头 shell + provider 填充”的结构。
- `AppLayout`、topbar、left rail、right rail 不应直接硬编码业务按钮、菜单项或页面分支。
- provider 统一命名为 `AdminProvider`，由单一聚合 provider 暴露 shell 内容，例如 topbar、菜单树、right panel slot。
- 顶栏按钮回调与导航命令由 provider 提供，骨架层只负责渲染和触发。
- 左侧导航使用显式的树模型，不再依赖写死在骨架里的字符串分组或路由分支判断。
- 通用 admin 壳子沉淀到 `crates`，应用侧默认只保留 provider 实现和接入代码。
- provider 收集默认使用 `rudi` 的注册的编译期注册表；禁止再引入第二套 DI 轨道。
- 多模块开发按“大功能一模块”组织；文件粒度保持在人类可以轻松阅读的范围内，避免继续堆积成巨型文件。

## Admin Navigation Convention

- admin 工作面默认按 `双轴上下文（Bi-Axial Context）` 理解，不把顶栏简单理解成“场景切换”。
- 顶栏承载 `主轴上下文树`：用于切换业务域、产品壳、子系统根节点或大型路由组；它本身可以是树的压缩视图，而不要求永远是一排平铺 tab。
- 左侧栏承载 `侧轴上下文树`：用于展开当前主轴下的模块、对象、节点、子流程或子菜单。
- 内容区是 `(主轴节点, 侧轴节点)` 的二维上下文交点；头部和侧栏都可以是树，不强行把其中任一轴扁平化为单层菜单。
- 术语上优先使用 `domain`、`context axis`、`context tree`、`二维上下文`，避免把所有分组都泛化叫成 `scene`。

## Admin Data Convention

- admin 业务数据默认以 PostgreSQL 为唯一正式持久化源，遵循 `all in pg`。
- 内存实现、构建期嵌入、文件扫描结果只可作为临时开发态、导入态或降级态，不作为正式数据落点。
- 新增 admin 模块时，先定义 PG 中的数据模型、迁移和读写边界，再接页面。

## API And CLI Convention

- 后端默认采用 `axum + dioxus` 组合。
- REST API 与 CLI 必须来自同一套操作定义或 contract，不维护两套漂移的接口面。
- 新功能进入正式阶段时，同时考虑 API、CLI 与 admin 三个交付面，而不是只做其中一个。

## Change And Traceability Convention

- 对于已经可以被新实现平行替代的旧接口，默认直接删除旧接口、旧分支与兼容层，不以“保持兼容”为理由长期保留双轨实现。
- 兼容适配层只能作为明确例外存在；引入时必须同时写清保留理由、预期移除时机与风险边界，不能把临时过渡做成长期结构。
- 功能来自 `feature request` 或 issue 时，在合适的实现注释、模块注释或 rustdoc 中标注来源，例如 `#{123}`，保证后续可以追溯功能入口与决策出处。
- `#{issueId}` 标注应放在真正承载该功能语义的位置，避免机械重复刷满整个文件。
- 对外公开 API 默认补足正确、可读的 rustdoc，至少说明职责、关键约束与必要用法。
- 测试中的关键断言，优先补一行短注释说明断言意图，强调“为什么要验证这里”，而不只是重复断言字面含义。

## Rust Module Convention

- 仓库第一方目录名使用无发布前缀的短领域名，例如 `crates/network/addhost`、`apps/aio`、`plugins/linux`；目录名不携带 `az-`。
- Cargo 包名作为 crates.io 全局身份继续使用 `az-` 前缀，例如目录 `addhost` 中声明 `name = "az-addhost"`；不在发布阶段临时改写包名。
- 第一方源码文件必须按实际功能或领域概念命名，文件名本身应能回答“这里负责什么”；优先使用 `contract.rs`、`routes.rs`、`store.rs`、`client.rs`、`navigation.rs`、`conversion.rs` 等职责名。
- 禁止使用 `api.rs`、`common.rs`、`utils.rs`、`helpers.rs`、`misc.rs` 等无法表达具体职责的泛化文件名；只有外部代码生成器固定产出的文件可以保留生成器约定名称，并且必须位于 `generated/` 边界内。
- 当一个文件同时承载多个可独立描述的大功能时，应先按职责拆分模块，再分别命名；不要通过改成另一个宽泛名称掩盖巨型文件问题。
- 协议契约使用 `contract.rs`，HTTP 路由或处理器按职责使用 `routes.rs` / `controllers/`，持久化边界使用 `store.rs`，平台传输边界使用 `http.rs` / `storage.rs`；不要把这些不同职责重新汇总到单个门面文件。
- Rust 代码默认遵循 `2018+ file-based modules`，优先使用 `foo.rs` 作为模块入口，不回到 `foo/mod.rs`。
- 目录模块发现默认优先使用 `automod` 之类的显式模块收集方案，前提是目录本身保持“只放正式模块”的整洁边界。
- `src/` 下只放正式参与编译的模块文件；草稿、实验、废弃迁移稿不得继续放在 `src/` 模块树内。
- 不再维护“大一统 re-export 门面”文件，例如在 `services.rs` 中集中 `pub use` 整个子系统的 DTO、service、helper。
- 调用方默认直接依赖真实模块路径，例如 `crate::services::ai_chat::ChatRequestDto`，并借助 IDE 自动导包，而不是依赖扁平门面别名。
- 共享入口文件只保留真正的入口职责：模块发现、少量基础类型别名、必要的顶层编排；不要把它继续堆成隐式 API 广场。
- 对普通数据结构、上下文结构、配置结构，默认优先直接使用 `T { ... }` 结构体字面量构造，而不是继续补自定义 fluent builder / `with_xxx` 链式接口。
- 只有当类型不适合公开字段、需要强约束校验、必须隐藏内部不变量，或外部库生态已经明确要求 builder 模式时，才引入 `new(...)` / builder；否则按字面量初始化处理。

## Frontend And Backend Naming Convention

- 前端目录按 UI 语义命名：可复用 UI 单元放入 `components/`，可独立路由或工作面的页面放入 `screens/`；不要用泛化的 `api.rs` 或后端分层名称承载前端 UI。
- 前端 HTTP、浏览器存储和其他传输/平台边界按真实职责命名，例如 `http.rs`、`storage.rs`、`bootstrap.rs`；`api` 只用于明确的对外协议或操作定义。
- 后端目录按职责分层：HTTP 入口放入 `controllers/`，业务编排放入 `services/`，领域与持久化数据结构放入 `models/`。
- 不要为了形式上的分层增加只做一对一转发的 controller、service 或 model；文件归属以实际职责和人类可读性为准。

## Rust Error Convention

- Rust 运行时错误默认直接使用 `anyhow::Result<T>`，不再为每个 crate 定义独立 `Error` enum 和 `XxxResult<T>` alias。
- 删除只服务于 Rust 错误传递的手写错误格式；只有协议模型、HTTP 响应体、外部 schema、业务状态枚举等“数据模型”可以继续保留 `Error` 命名类型。
- 不再把 `thiserror` 作为默认依赖；只有确有数据模型或外部协议要求时才保留。
- 使用 `anyhow` 时应在 I/O、图片读取、模型加载等关键失败点补 `Context`，让错误链直接说明失败位置和操作对象。

### Recoverable Multi-Error Flow

- 参考 [A Novel Look at Error Handling in Rust](https://jtjlehi.github.io/2026/06/25/novel-rust-error-handling.html)，补充处理“内部发生错误，但仍可安全产出降级或部分结果”的场景；该模式不替代普通 `anyhow::Result` 错误传播。
- 先区分错误是否允许继续：结果已不可信、关键不变量被破坏或后续操作无意义时，继续返回 `anyhow::Result<T>` 并立即使用 `?` 早退；只有存在明确且领域安全的降级值时才允许继续执行。
- 需要“继续执行 + 向调用方报告错误”时，统一复用 `crates/core/error` 提供的 `az_error::diagnostics::Diagnostics`，并显式传入 `&mut Diagnostics`；禁止各 crate 重复实现收集器，也禁止使用全局变量、线程局部变量或其他隐藏错误通道。
- 诊断收集器默认保存带 `Context` 的 `anyhow::Error`，支持一次调用收集多个错误，并保持稳定、可定位的发生顺序；禁止只保留最后一个错误或静默覆盖先前错误。
- 使用 `Diagnostics::capture` 把失败记录后转换为 `Option<T>`，或使用 `Diagnostics::recover` 执行调用方显式提供的领域降级闭包，使业务 happy path 保持主导；不要在每个调用点复制大段 `match` 错误样板。
- 降级值必须具备明确的领域语义并在 API/rustdoc 中说明；禁止为让流程“跑完”而盲目使用 `.ok()`、`.unwrap_or_default()` 或任意占位值吞掉错误。
- 调用方应在阶段边界统一决定：接受部分结果时使用 `Diagnostics::iter` / `into_errors` 展示或记录全部诊断；拒绝部分结果时使用 `finish` / `into_result` 聚合失败返回；不要在深层函数中替调用方过早丢弃诊断或决定最终策略。
- 诊断收集器只覆盖一个清晰阶段或批处理范围，不跨越无关层级长期传递，也不把所有普通 `Result` API 改造成错误参数模式。
- 测试应同时验证降级结果、全部诊断及其关键上下文，并验证致命错误仍会立即终止流程，避免“可继续”模式掩盖真正失败。
