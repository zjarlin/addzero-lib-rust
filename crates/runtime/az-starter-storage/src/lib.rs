//! 文件中心系统插件，管理上传下载能力与插件包仓库。
//!
//! 本 crate 作为 addzero 系统插件体系中的「文件中心」入口，
//! 通过 [`az_plugin_macros::az_starter`] 宏自动向全局插件注册中心注册自身，
//! 提供包仓库管理页面，展示插件工件的存储架构与元数据索引方式。
//!
//! ## 核心功能
//!
//! - **包仓库页面**：以表格方式展示 PostgreSQL 元数据存储与本地文件/Git 发布归档的分工
//! - **系统菜单贡献**：注册到「系统插件」分区，菜单项「包仓库」，排序权重 60
//! - **插件工件管理**：支持 `.wasm` 业务插件上传，元数据写入 PG，二进制走文件/Git 归档
//! - **跨平台兼容**：兼容 web 与 desktop 两种运行环境
//!
//! ## 关键类型
//!
//! - [`StorageStarter`] — 实现 [`az_plugin_registry::PluginStarter`] 的插件描述结构体
//! - [`register_storage`] — `#[az_starter]` 标记的注册入口函数
//!
//! ## 用法
//!
//! 通常不直接调用本 crate 的公开接口；宿主应用通过
//! [`az_system_starters::link_all`] 统一链接所有系统 starter，
//! 由插件注册中心在运行时发现并组装菜单与页面。

use az_plugin_contract::{
    PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage, TableRow,
    TableSchema,
};
use az_plugin_macros::az_starter;
use az_plugin_registry::PluginStarter;

pub fn ensure_linked() {}

struct StorageStarter;

impl PluginStarter for StorageStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "storage".to_string(),
            name: "文件中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "管理上传下载能力与插件包仓库。".to_string(),
            tags: vec!["system".to_string(), "storage".to_string()],
            icon: Some("folder".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "包仓库".to_string(),
                page_id: "packages".to_string(),
                order: 60,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "packages".to_string(),
                title: "包仓库".to_string(),
                subtitle: "业务插件以 .wasm 工件上传，元数据写入 PostgreSQL，二进制走本地文件或 Git 发布归档。"
                    .to_string(),
                schema: PageSchema::Table(TableSchema {
                    columns: vec!["目录".to_string(), "内容".to_string(), "说明".to_string()],
                    rows: vec![
                        TableRow {
                            cells: vec![
                                "PostgreSQL".to_string(),
                                "插件元数据".to_string(),
                                "页面、菜单、维护者、安装命令和资源索引".to_string(),
                            ],
                        },
                        TableRow {
                            cells: vec![
                                "本地文件 / Git 发布".to_string(),
                                "*.wasm / CLI 资源".to_string(),
                                "运行时二进制、脚本文件对象与发布归档".to_string(),
                            ],
                        },
                    ],
                    empty_message: "暂无包仓库信息。".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        }
    }
}

#[az_starter]
pub fn register_storage() -> Box<dyn PluginStarter> {
    Box::new(StorageStarter)
}
