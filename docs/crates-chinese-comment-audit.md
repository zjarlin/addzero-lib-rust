# crates 中文注释巡检

巡检日期：2026-05-27

本巡检覆盖 `crates/**/Cargo.toml`、各 crate 的 `src/lib.rs` 入口说明、公开宏边界和当前已经落地的 derive alias 使用方式。目标不是把源码刷满中文注释，而是把中文说明放在后续维护者真正会读、且不容易腐烂的位置。

## 巡检结论

- 当前 `crates` 下共有 91 个 Cargo manifest。
- 除 `crates/runtime/az-cli` 是二进制入口、没有常规 `src/lib.rs` 外，其余库 crate 均已有有效 crate 级说明入口。
- 其中 65 个库 crate 使用 `//!` 直接写 crate 级说明，25 个库 crate 使用 `#![doc = include_str!("../README.md")]` 复用 README 作为 rustdoc。
- `crates/**/*.rs` 当前没有未归档的 `TODO` / `FIXME` 标记。
- 最值得补中文说明的位置不是普通字段和显而易见的派生，而是公开宏、wire/code enum 约定、插件注册边界、协议/持久化边界、以及会被多个 crate 复用的错误和上下文类型。

## 注释分层标准

### crate 入口

每个库 crate 至少保留一种入口说明：

- `//!`：适合短小库或核心契约，直接说明 crate 负责什么、不负责什么。
- `#![doc = include_str!("../README.md")]`：适合 README 已经承载安装、示例、约束的 crate，避免 README 和 rustdoc 双份漂移。

入口说明优先回答三件事：

- 这个 crate 的正式职责是什么。
- 它和相邻 crate 的边界是什么。
- 调用方最常用的入口类型、函数或宏是什么。

### 公开 API

公开的 `pub struct`、`pub enum`、`pub trait`、`pub fn`、`#[macro_export]` 应优先用 `///` 写中文 rustdoc。重点不是解释字段字面含义，而是解释行为契约：

- 错误类型说明何时返回，是否保留源错误链。
- 协议 DTO 说明 wire 格式、命名约定和兼容风险。
- 注册宏说明注册发生在编译期还是运行时，调用方需要额外做什么。
- code enum 说明 `code()`、`as_str()`、`Display`、serde wire value 之间的关系。

### 局部 `//` 注释

局部注释只写“为什么”，不写“做什么”。适合以下情况：

- 安全性、平台差异、性能绕行、协议兼容、迁移边界。
- 测试中的关键断言意图。
- 临时兼容层的移除条件。

不建议给普通 getter、字段赋值、显而易见的 match 分支补中文行注释。

## 分组巡检

| 分组 | crate 数 | 当前状态 | 后续重点 |
| --- | ---: | --- | --- |
| `api` | 10 | 契约和外部 API crate 普遍已有中文 crate 说明，部分 item-level rustdoc 仍是英文。 | wire 格式、认证边界、外部服务失败语义继续补中文。 |
| `apps` | 5 | 多数通过 README 注入 rustdoc，适合文档站收录。 | 插件职责、admin 路由挂载和 provider 边界保持 README 优先。 |
| `config` | 2 | TOML/YAML 都有较完整中文入口说明。 | 错误分支和环境变量替换约束补到公开 API。 |
| `core` | 18 | 基础工具 crate 基本已有中文说明；`az-derive-aliases` 用 README 承载 alias 全量清单。 | 宏 alias 只补功能层说明，不为每个机械 derive 重复造长注释。 |
| `data` | 12 | 数据、DDL、SQL、持久化 crate 已有中文入口说明。 | AST、迁移、PG 正式数据源边界继续用 crate/module doc 表达。 |
| `network` | 13 | 协议、远程、邮件、临时邮箱等边界说明较完整；部分自动化/代理 crate 通过 README 承载。 | 外部服务、鉴权、协议脱敏、不可自动化边界需要保持中文可审计。 |
| `runtime` | 22 | admin、脚本、Drive agent、starter 等 crate 已形成中文入口说明；`az-cli` 是二进制入口。 | 插件注册宏和 CLI 操作定义是最高价值补注释点。 |
| `storage` | 5 | Drive store 和 gitdb 相关 crate 通过 README 或模块文档承载。 | typestate、shard、对象存储/元数据分层应补设计型中文说明。 |
| `text` | 2 | 通过 README 注入 rustdoc。 | 公开转换规则和编码约定应在 README/rustdoc 中同步。 |
| `ui` | 2 | 通过 README 注入 rustdoc。 | 组件 Props、交互契约和可访问性约束优先写在公开组件上。 |

## 本轮高价值补点

### 已补

- `az-admin-plugin-registry`：公开注册宏补中文 rustdoc，明确 `domain`、`branch`、`page`、`root page` 和 starter 插件链接保活入口的语义。
- `az-cli-market-contract`：给 CLI 市场 wire enum、关键 DTO 和 base64 artifact 编解码入口补中文 rustdoc，明确 `code()` / serde wire value 的兼容边界。
- `az-config-center-contract`：给 Shell 组件契约补中文 rustdoc，区分 `ShellComponentKind` 的 wire value 和 `Display` 分组名。
- `az-remote-protocol`：给协议错误、流类型、握手帧、文件/视频块和 `ControlFrame` 补中文 rustdoc，强调 relay token 脱敏和 JSON wire contract。
- `az-browser-automation`：将 OpenAI 手工记录步骤相关公开说明改为中文，明确 step id、`Display` 和 `strum(message)` 的职责分层。
- `docs/README.md`：增加本巡检文档入口，方便文档站收录。

### 继续建议

- `az-derive-aliases`：README 已经是中文 alias 清单，后续只在新增 alias 时维护“功能型分层”，不要再按 struct/enum 或业务语义拆出大量平行宏。
- `az-browser-automation`：后续新增记录步骤时继续用 `strum(message = "...")` 承载长说明，避免手写 match 和注释分离。
- `az-cli-market-contract`、`az-config-center-contract`：后续扩展字段或新增 wire enum 时同步补中文 rustdoc 和兼容性测试。
- `az-drive-store`、`gitdb`、`toasty-driver-gitdb`：存储层需要用模块级中文说明固定对象层、元数据层、shard 层边界。
- `az-remote-protocol`、`az-wasm-plugin-api`：协议 crate 不要靠每个字段行注释堆信息，优先维护 crate/module 级“帧类型、兼容性、脱敏、安全边界”说明。

## 不建议补注释的位置

- 纯机械 derive alias 调用，如 `#[apply(plain_eq)]`、`#[apply(serde_eq)]`，除非该类型的语义边界不明显。
- SQL/DDL AST、协议帧 payload enum、远程输入事件等结构型 enum，不应为了“统一 code enum”而改 wire 形状或加误导性 code 注释。
- 隐藏宏 helper 的展开细节不应写成公开教程；公开说明放在 README 和对外宏 rustdoc。
- 测试里不需要解释每个 `assert_eq!` 字面含义，只补关键断言为什么守住某个协议或兼容边界。

## 后续执行顺序

1. 新增 crate 时先补 `//!` 或 README 注入，保证文档站和 rustdoc 同源。
2. 公开 API 出现跨 crate 调用前补中文 `///`，尤其是错误、协议、注册宏、trait。
3. 修改 wire enum 或 serde 命名时同步补测试，并在文档说明兼容风险。
4. 只在代码意图无法通过命名表达时写局部 `//` 注释。
5. 每轮巡检至少跑 `git diff --check`，涉及 rustdoc 示例时再跑对应 crate 的 `cargo test`。
