# addzero-lib-rust Issues

> 共 36 个 issues

## 🔴 未完成（27）

| # | Issue | 标签 |
|---|-------|------|
| [#45](https://github.com/zjarlin/addzero-lib-rust/issues/45) | [msc-aio] 🟡 FilesScene 命名冲突 + Knowledge 域过载 refactor | refactor, msc-aio, convention |
| [#44](https://github.com/zjarlin/addzero-lib-rust/issues/44) | [msc-aio] 🟡 阻塞 I/O 伪装 async — logo_storage / package_storage | performance, msc-aio |
| [#43](https://github.com/zjarlin/addzero-lib-rust/issues/43) | [msc-aio] 🟡 LocalBoxFuture 类型别名在 8 个 service 文件中重复定义 | refactor, msc-aio |
| [#42](https://github.com/zjarlin/addzero-lib-rust/issues/42) | [msc-aio] 🟡 scenes/ 术语不符合 AGENTS.md 约定，应改为 domains/ | refactor, msc-aio, convention |
| [#41](https://github.com/zjarlin/addzero-lib-rust/issues/41) | [msc-aio] 🔴 签名失败时静默放行 — issue_cookie 可发出空签名 cookie | security, msc-aio |
| [#40](https://github.com/zjarlin/addzero-lib-rust/issues/40) | [msc-aio] 🔴 默认凭证 admin/admin + 硬编码 session secret | security, msc-aio |
| [#39](https://github.com/zjarlin/addzero-lib-rust/issues/39) | [msc-aio] 🔴 每次请求新建 PG 连接池 — asset_graph/knowledge_graph/cli_market | bug, performance, msc-aio |
| [#38](https://github.com/zjarlin/addzero-lib-rust/issues/38) | [Refactor] network/core crate 中 mutex/rwlock expect 应改为 graceful recovery | bug, network, core |
| [#37](https://github.com/zjarlin/addzero-lib-rust/issues/37) | [Test] network crate 普遍缺少纯函数单元测试 | enhancement, network |
| [#36](https://github.com/zjarlin/addzero-lib-rust/issues/36) | [Chore] 统一 workspace 级 forbid(unsafe_code) | enhancement, core |
| [#35](https://github.com/zjarlin/addzero-lib-rust/issues/35) | [Refactor] dioxus-admin: knowledge_graph 5 处 expect + 缺少 Error Boundary | bug, priority-high, web |
| [#34](https://github.com/zjarlin/addzero-lib-rust/issues/34) | [Fix] addzero-browser-automation: Chrome 子进程泄漏 | bug, network |
| [#33](https://github.com/zjarlin/addzero-lib-rust/issues/33) | [Fix] addzero-curl: CurlExecutor::default() panic if TLS init fails | bug, network |
| [#32](https://github.com/zjarlin/addzero-lib-rust/issues/32) | [Fix] addzero-curl: Regex 每次调用重编译（性能问题） | bug, priority-high, network |
| [#29](https://github.com/zjarlin/addzero-lib-rust/issues/29) | [Doc] crates/core: 12 个 crate 公共 API 缺少文档注释 | documentation, priority-high |
| [#19](https://github.com/zjarlin/addzero-lib-rust/issues/19) | [Impl] addzero-cte — #1 子任务 | feature, impl |
| [#15](https://github.com/zjarlin/addzero-lib-rust/issues/15) | [Plan] Rust 化执行路线图 — 7阶段 ~55 crate | feature, priority-high |
| [#14](https://github.com/zjarlin/addzero-lib-rust/issues/14) | [Feature] 配置管理增强：表达式求值 + 热重载 + 加密 | feature, config |
| [#13](https://github.com/zjarlin/addzero-lib-rust/issues/13) | [Feature] 国产数据库方言：达梦 + 人大金仓 | feature, database, priority-high, dialect |
| [#12](https://github.com/zjarlin/addzero-lib-rust/issues/12) | [Feature] KBox 本地工具箱 + KCloud 云同步 | feature, toolchain, storage |
| [#11](https://github.com/zjarlin/addzero-lib-rust/issues/11) | [Feature] Rust 宏等价 KCP：i18n + Builder + Spread | feature, macro, kcp |
| [#10](https://github.com/zjarlin/addzero-lib-rust/issues/10) | [Feature] 音乐平台扩展：QQ音乐 + 网易云 + Suno | feature, api, music |
| [#7](https://github.com/zjarlin/addzero-lib-rust/issues/7) | [Feature] Web 框架中间件：全局错误处理 + 请求日志 + 验证 | feature, web, spring |
| [#5](https://github.com/zjarlin/addzero-lib-rust/issues/5) | [Feature] ORM 扩展：动态数据源 + 审计字段 + 多租户 | feature, database, orm |
| [#4](https://github.com/zjarlin/addzero-lib-rust/issues/4) | [Feature] Modbus 协议支持：RTU/TCP/MQTT + 代码生成 | feature, priority-high, iot, modbus |
| [#2](https://github.com/zjarlin/addzero-lib-rust/issues/2) | [Feature] 代码生成器：proc-macro 等价 KSP/APT/LSI 架构 | feature, priority-high, codegen |
| [#1](https://github.com/zjarlin/addzero-lib-rust/issues/1) | [Feature] 数据库支持：DDL生成器 + SQL工具链 | feature, database, priority-high |

## ✅ 已完成（9）

| # | Issue | 标签 |
|---|-------|------|
| [#31](https://github.com/zjarlin/addzero-lib-rust/issues/31) | [Fix] addzero-reflection::ExpiringCache 构造 panic + expect on Mutex | bug, priority-high, core |
| [#30](https://github.com/zjarlin/addzero-lib-rust/issues/30) | [Fix] addzero-tree::build_tree 死代码 + 文档误导 | bug, priority-high, core |
| [#22](https://github.com/zjarlin/addzero-lib-rust/issues/22) | [Impl] addzero-ddl-generator — #1 子任务 | feature, impl |
| [#21](https://github.com/zjarlin/addzero-lib-rust/issues/21) | [Impl] addzero-sql — #1 子任务 | feature, impl |
| [#20](https://github.com/zjarlin/addzero-lib-rust/issues/20) | [Impl] addzero-database-model — #1 子任务 | feature, impl |
| [#9](https://github.com/zjarlin/addzero-lib-rust/issues/9) | [Feature] AI/LLM 集成：Chat 模型抽象 + Embedding + TTS | feature, priority-high, ai |
| [#8](https://github.com/zjarlin/addzero-lib-rust/issues/8) | [Feature] IoT/嵌入式：STM32 Bootloader + 串口通信 | feature, iot, embedded |
| [#6](https://github.com/zjarlin/addzero-lib-rust/issues/6) | [Feature] 跨平台基础工具补全：JSON/Tree/Error/Regex | feature, core, kmp |
| [#3](https://github.com/zjarlin/addzero-lib-rust/issues/3) | [Feature] 网络API集成：天气/视频/翻译/支付/OCR | feature, network, api |
