//! 插件系统的运行时内核，负责插件生命周期管理、服务注入和前端 Shell 数据聚合。
//!
//! 本 crate 提供 [`PlatformKernel`] 作为插件系统的统一入口，核心职责包括：
//! - 插件安装与实例管理：从目录安装插件、创建运行时实例、刷新目录
//! - Shell 数据聚合：组合当前用户信息、导航树、插件计数，生成 [`ShellSnapshot`]
//! - 市场数据聚合：合并系统插件与业务插件，生成 [`MarketplaceSnapshot`]
//! - 页面解析：根据 `plugin_id` + `page_id` 或 `instance_slug` + `page_id` 解析页面
//!
//! 服务注入基于 `shaku` 框架，预置五个核心服务接口：
//! - [`AuthProvider`]：当前用户身份与认证模式
//! - [`RbacService`]：权限校验
//! - [`DictionaryService`]：字典数据（笔记类型等）
//! - [`AuditService`]：审计日志种子条目
//! - [`StorageService`]：存储路径提示
//!
//! 开发环境提供默认实现：`DevAuthProvider`（admin/admin）、`AllowAllRbacService`（全放行）等。

use std::{