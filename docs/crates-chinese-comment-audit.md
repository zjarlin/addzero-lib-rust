# crates 中文注释巡检

巡检日期：2026-05-27 初检，2026-05-28 复检

本巡检覆盖 `crates/**/Cargo.toml`、各 crate 的 `src/lib.rs` 入口说明、公开宏边界和当前已经落地的 derive alias 使用方式。目标不是把源码刷满中文注释，而是把中文说明放在后续维护者真正会读、且不容易腐烂的位置。

## 巡检结论

- 当前 `crates` 下共有 91 个 Cargo manifest。
- 除 `crates/runtime/cli` 是二进制入口、没有常规 `src/lib.rs` 外，其余库 crate 均已有有效 crate 级说明入口。
- 其中 65 个库 crate 使用 `//!` 直接写 crate 级说明，25 个库 crate 使用 `#![doc = include_str!("../README.md")]` 复用 README 作为 rustdoc。
- `crates/**/*.rs` 当前没有未归档的 `TODO` / `FIXME` 标记。
- 最值得补中文说明的位置不是普通字段和显而易见的派生，而是公开宏、wire/code enum 约定、插件注册边界、协议/持久化边界、以及会被多个 crate 复用的错误和上下文类型。

## 2026-05-28 复检快照

本轮复检使用仓库当前 `crates/**/*.rs` 做机械扫描，只把结果当作“找补点索引”，不直接等同于文档质量结论。README 注入型 rustdoc、模块级 `//!`、宏展开后的公开项，以及字段级文档不会完全反映在 item-level 命中数里。

| 指标 | 数量 | 说明 |
| --- | ---: | --- |
| Cargo manifest | 91 | 来自 `find crates -name Cargo.toml`。 |
| inline crate doc | 65 | `src/lib.rs` 中存在 `//!` 入口说明。 |
| README 注入 rustdoc | 25 | `#![doc = include_str!("../README.md")]`。 |
| 二进制入口 | 1 | `crates/runtime/cli`，没有常规 `src/lib.rs`。 |
| 公开 item 粗扫 | 2688 | 匹配 `pub struct/enum/trait/fn/type/const/static/mod` 和 `#[macro_export]`。 |
| 已有 item rustdoc | 1926 | 公开 item 前 8 行内存在 `///`。 |
| 中文 item rustdoc | 777 | item rustdoc 中含中文字符。 |

### 优先补注释队列

| 优先级 | crate | 依据 | 建议动作 |
| --- | --- | --- | --- |
| P1 | `az-music` | 已补一轮 item-level 中文 rustdoc，HTTP 配置、网易云搜索/歌词、Suno token、任务轮询和 wire DTO 边界已明确。 | 后续新增平台或异步接口时同步补第三方协议兼容说明。 |
| P1 | `az-mqtt` | 已补一轮 item-level 中文 rustdoc，QoS、TLS、后台轮询线程、接收超时和自动断开边界已明确。 | 后续新增 MQTT 5 能力或异步 API 时同步补协议兼容说明。 |
| P1 | `az-email` | 已补一轮 item-level 中文 rustdoc，SMTP 配置、消息构建、sender 注入和默认 sender 边界已明确。 | 后续新增 provider 或发送策略时同步补 `EmailSenderKind` / `EmailSenderConfig` 说明。 |
| P1 | `az-yml`、`az-toml` | 已补一轮 item-level 中文 rustdoc，仍可继续补测试断言意图和 README 场景示例。 | 后续只在新增 API 或发现行为歧义时补充，避免重复注释。 |
| P1 | `az-software-catalog` | 已补一轮 item-level 中文 rustdoc，软件平台/安装器 wire code、DTO 字段、服务启动 seed 和保存归一化边界已明确。 | 后续新增安装器或资产库联动字段时同步补兼容说明和测试。 |
| P1 | `az-excel`、`az-knowledge` | 数据 crate 入口说明存在，但公开模型和操作仍缺少中文契约。 | 补 Excel 导入/导出语义、知识源扫描/正式持久化与临时扫描结果的区别。 |
| P1 | `az-ssh`、`az-remote-session`、`az-cli-repl` | 已补一轮 item-level 中文 rustdoc，SSH 会话/文件传输、REPL 解析、远程会话中继生命周期已明确。 | 后续新增交互输入、远程帧类型或连接复用能力时同步补失败边界。 |
| P1 | `az-aio-plugin-config-center`、`az-aio-plugin-edge-gateway` | 插件 crate 通过 README 注入入口，但注册规格和 toolbar/page 贡献边界还可继续中文化。 | 给 provider、route、toolbar action、page contribution 的公开常量和函数补简短 rustdoc。 |
| P2 | `gitdb`、`az-browser-automation` | 总体已补多轮中文说明，但仍有大量英文历史 rustdoc。 | 继续按模块切片替换核心公开项，不做全量机械翻译。 |

### 巡检判断

当前仓库不缺 crate 级入口说明，真正缺的是“跨 crate 会被调用的公开 API 契约”。后续每轮应选 2 到 4 个 crate 做小批量补注释，并同步跑对应 crate 的 `cargo fmt --check`、`git diff --check`，避免把注释巡检变成不可验证的大面积文本 churn。

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
| `core` | 17 | 基础工具 crate 基本已有中文说明；机械 derive alias 已删除，普通类型应直接写显式 `#[derive(...)]`。 | 新增宏时只保留真正的代码生成职责，避免用宏隐藏普通 derive 组合。 |
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
- `az-json`、`az-ai-chat`：续补 JSON 工具函数、聊天通用 trait、OpenAI 兼容客户端和 manifest 描述的中文 rustdoc。
- `gitdb` 存储组：续补 SQL AST、逻辑/物理计划、Volcano 执行算子、typestate 事务、Git 存储类型和行 blob 编解码的中文 rustdoc，明确 SQL 子集、执行模型、事务状态和 Git 路径安全边界。
- `az-serial`：补齐串口错误、端口句柄、配置枚举和帧解码器的中文 rustdoc，明确 `Baud0`、非阻塞 timeout、帧边界和 buffer overflow 语义。
- `az-proxy`：补齐订阅获取、订阅解析、代理节点类型、节点归一化和 TCP 测速的中文 rustdoc，明确 Clash YAML / URI / base64 订阅自动识别边界。
- `az-rustfs`：补齐 S3 兼容配置、凭证脱敏、对象元数据、客户端 trait、阻塞客户端、内存客户端、分片上传进度和断点续传状态的中文 rustdoc。
- `toasty-driver-gitdb`：补齐 Toasty 能力矩阵、driver URL、连接工作线程和 gitdb/toasty 错误转换边界的中文 rustdoc。
- `az-array`、`az-common`、`az-model`：补齐基础集合工具、本地日期时间工具、实体分页模型 trait 的中文 rustdoc，明确边界条件、时区语义、分页零基 offset 和空分页规则。
- `gitdb::catalog`：把 `DataType` 的 wire code、SQL 展示名和 `strum::EnumProperty` 元数据关系写进中文 rustdoc，替换手写 SQL 类型 match 表。
- `gitdb::storage::types`：保留 `TableName`、`RowKey`、`BranchName` 的业务校验，并显式实现 `as_str()` / `into_string()`。
- `az-str`：补齐字符串规格化、前后缀处理、命名转换、KMP 匹配、模板格式化、Markdown/HTML 提取、键值对解析和特殊字符转义等公开工具函数的中文 rustdoc，明确空值、UTF-8 字节偏移、宽松数值转换和上下文专用编码器边界。
- `az-creates`：补齐统一错误类型、`Creates` 门面、Maven Central 包装器、天眼查普通/华为云签名客户端和关键响应 DTO 的中文 rustdoc，明确外部 API 凭证边界、上游响应兼容、provider factory 依赖注入入口和错误链保留方式。
- `az-yml`、`az-toml`：补齐 YAML 路径查询、环境变量替换、Spring profile 激活、数据库配置提取、Version Catalog 解析/初始化/合并和 TOML 插入宏的中文 rustdoc，明确路径语法、默认值、脱敏、排序输出和非解析式文本插入边界。
- `az-email`：补齐 SMTP 配置、邮件消息、sender trait/factory、默认 sender、快捷发送函数和 `lettre::Message` 构建入口的中文 rustdoc，明确密码脱敏、TLS 选择、附件 IO/MIME 推断、进程级默认发送器和临时 sender 创建成本。
- `az-mqtt`：补齐 QoS、消息/订阅、连接配置、TLS 文件路径、后台轮询线程、接收超时、批量收集和显式断开的中文 rustdoc，明确 `rumqttc` 转换、证书/私钥脱敏、Last Will 校验和 Drop 清理边界。
- `az-music`：补齐 HTTP 配置、音乐客户端门面、网易云搜索/歌词/详情 API、搜索结果 DTO、Suno token 客户端、任务轮询和生成请求/任务模型的中文 rustdoc，明确第三方业务码、bearer token 脱敏、默认请求头、轮询完成状态和上游 wire 字段保留边界。
- `az-kiro-auth-support`：补齐 OIDC 配置、设备流程、token 轮询状态、身份/密码生成、验证码提取和不支持自动化能力的中文 rustdoc，明确 AWS Builder ID device flow、User-Agent、轮询终态和安全边界。
- `az-ssh`：补齐 SSH 错误、连接配置、认证材料脱敏、命令执行结果、会话方法和快捷函数的中文 rustdoc，明确 TCP/握手/认证/命令/SFTP 失败边界。
- `az-cli-repl`：补齐 REPL 参数类型、参数值、参数定义、错误、命令 trait、执行结果和单行解析入口的中文 rustdoc，明确 `code()`、`Display`、布尔别名、默认值和命令序号规则。
- `az-remote-session`：补齐远程会话中继结果、错误、运行时配置和服务方法的中文 rustdoc，明确进程内状态、设备注册、会话授权、剪贴板/视频帧/文件暂存和拒绝授权行为。
- `az-software-catalog`：补齐软件平台、安装器、安装方法、软件条目、目录响应、保存输入、主页元数据、草稿输入、错误和服务入口的中文 rustdoc，明确 `package` wire 兼容、标签/平台去重、空方法过滤和默认 seed 边界。

### 继续建议

- 机械 derive alias 已删除；后续新增类型直接写显式 `#[derive(...)]`，不要再按 struct/enum 或业务语义拆出大量平行宏。
- `az-browser-automation`：后续新增记录步骤时继续用 `strum(message = "...")` 承载长说明，避免手写 match 和注释分离。
- `az-cli-market-contract`、`az-config-center-contract`：后续扩展字段或新增 wire enum 时同步补中文 rustdoc 和兼容性测试。
- `az-drive-store`、`gitdb`：存储层仍需继续用模块级中文说明固定对象层、元数据层、shard 层边界；`toasty-driver-gitdb` 的 driver 边界已补一轮。
- `az-remote-protocol`、`az-wasm-plugin-api`：协议 crate 不要靠每个字段行注释堆信息，优先维护 crate/module 级“帧类型、兼容性、脱敏、安全边界”说明。

## 不建议补注释的位置

- 纯机械 derive 组合本身不需要注释，除非该类型的语义边界不明显。
- SQL/DDL AST、协议帧 payload enum、远程输入事件等结构型 enum，不应为了“统一 code enum”而改 wire 形状或加误导性 code 注释。
- 隐藏宏 helper 的展开细节不应写成公开教程；公开说明放在 README 和对外宏 rustdoc。
- 测试里不需要解释每个 `assert_eq!` 字面含义，只补关键断言为什么守住某个协议或兼容边界。

## 后续执行顺序

1. 新增 crate 时先补 `//!` 或 README 注入，保证文档站和 rustdoc 同源。
2. 公开 API 出现跨 crate 调用前补中文 `///`，尤其是错误、协议、注册宏、trait。
3. 修改 wire enum 或 serde 命名时同步补测试，并在文档说明兼容风险。
4. 只在代码意图无法通过命名表达时写局部 `//` 注释。
5. 每轮巡检至少跑 `git diff --check`，涉及 rustdoc 示例时再跑对应 crate 的 `cargo test`。
6. 对 `az-rustfs`、`az-proxy` 这类 README 注入 rustdoc 的 crate，后续优先让 README 承载场景示例，源码 `///` 只固定 API 契约。
