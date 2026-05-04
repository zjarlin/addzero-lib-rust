# `msc-aio` Blueprint

## 1. Problem Statement

`msc-aio` 不是单点工具，而是一套把个人知识资产、同步任务、自动化入口和命令能力统一收口的平台。它解决的问题有三个：

- 自己的知识库资产分散在目录、Markdown、脚本和临时工具里，缺少统一管理面
- 很多能力只能在网页或代码里调用，无法自然脚本化，也无法沉淀成稳定接口
- 数据当前分散在文件、内存和临时导入链路里，没有统一的正式存储与演进路径

## 2. Vision

`msc-aio` 的目标是做成一套 “Memory / Sync / CLI All-in-One” 工作台：

- Next.js Admin 负责管理知识资产、同步任务、配置和系统状态
- Tauri Desktop 壳负责加载同一套管理前端并开放桌面能力
- Axum 负责暴露 REST API、执行任务入口、认证与服务编排
- CLI 负责让所有核心能力天然可脚本化、可批处理、可自动化
- PostgreSQL `msc_aio` 作为唯一正式持久化源，承接全部业务数据

## 3. Architecture

### 3.1 Runtime Shape

- `PostgreSQL (msc_aio)`
  知识资产、分类、标签、来源、同步状态、任务历史、审计日志、系统配置、CLI 操作记录都正式进入 PG
- `Workspace domain crates`
  承载知识库、同步、导入导出、任务、资产管理等核心领域逻辑
- `Axum backend`
  负责 REST API、任务触发、认证授权、OpenAPI 暴露、后台编排
- `CLI`
  与 REST 使用同一套操作定义，默认面向脚本和自动化调用
- `Next.js admin`
  作为管理界面，用于浏览资产、查看状态、触发任务和管理配置
- `Tauri desktop`
  加载同一套静态导出前端，通过 localhost plugin 保持 loopback HTTP / cookie 合同一致

### 3.2 Source-of-Truth Rule

- 文件系统目录、Markdown 文件、构建期嵌入和内存实现都不是正式数据源
- 它们只承担导入、开发调试或降级演示角色
- 一旦进入正式流程，数据必须落进 `msc_aio`

## 4. Key Decisions

### ADR-001: Use PostgreSQL `msc_aio` as the system of record

- Status: Accepted
- Context:
  当前 admin 里已经存在构建期导入和内存实现，这适合快速原型，但不适合长期演进
- Decision:
  将 `msc_aio` 作为唯一正式持久化数据库
- Consequences:
  所有知识资产、同步任务与配置都必须设计表结构、迁移与仓储边界

### ADR-002: Use `axum + Next.js + Tauri` as the delivery pair

- Status: Accepted
- Context:
  后台需要同时具备浏览器管理面和稳定服务接口
- Decision:
  后端使用 Axum，前端管理台使用 Next.js App Router，桌面端使用 Tauri 壳加载同一套前端
- Consequences:
  领域逻辑不能写死在 UI 或路由层，必须下沉到 crate 级服务

### ADR-003: REST and CLI must come from the same operation definition

- Status: Accepted
- Context:
  手写 REST 和手写 CLI 两套接口会持续漂移
- Decision:
  以同一套操作定义或 contract 作为来源，同时挂到 Axum 路由和 CLI 子命令
- Consequences:
  CLI 不是后补物，而是与 API 同级的一等交付面

### ADR-004: Imports are pipelines, not persistence

- Status: Accepted
- Context:
  当前 Rust Markdown 资料已经能从目录导入到 admin 展示，但仍是编译期路径
- Decision:
  保留目录扫描作为 ingest pipeline，把结果写入 PG，而不是长期停留在 build-time embed
- Consequences:
  需要设计 ingest job、source registry、去重策略、内容 hash 与更新策略

## 5. Module Planning

建议按“大功能一模块”推进：

- `knowledge`
  知识资产、文档、标签、分类、目录来源、内容摘要、检索元数据
- `sync`
  外部来源同步、文件导入、状态机、冲突处理、重试策略
- `task`
  后台任务、执行历史、调度、失败恢复、批处理入口
- `config`
  系统配置、目录来源、连接信息、行为开关
- `audit`
  操作日志、变更记录、调用追踪
- `cli`
  命令面、参数绑定、输出格式、脚本友好行为

每个模块都应包含：

- PG schema / migration
- domain service
- API surface
- CLI surface
- admin scene

## 6. Immediate Next Steps

1. 先定义 `msc_aio` 的基础 schema
2. 把知识库笔记从 build-time embed 改成 ingest to PG
3. 把现有 `InMemorySkillsApi` 换成正式 PG 实现
4. 抽出共享 operation contract，作为 REST + CLI 的同源定义
5. 在 admin 中接 PG 读模型，而不是直接消费临时数据

## 7. Non-Goals Right Now

- 不急着把系统拆成微服务
- 不急着先做复杂权限模型
- 不先做多数据库混合持久化
- 不把文件系统继续当正式存储层长期保留
