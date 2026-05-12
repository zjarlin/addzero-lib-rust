//! 组织中心系统插件，维护部门结构、团队归属与责任人信息。
//!
//! 本 crate 作为 addzero 系统插件体系中的「组织中心」入口，
//! 通过 [`az_plugin_macros::az_starter`] 宏自动向全局插件注册中心注册自身，
//! 提供组织架构（部门树、上级关系、负责人、成员）的表格化管理页面。
//!
//! ## 核心功能
//!
//! - **部门管理页面**：以表格方式展示部门名称、上级部门、负责人与成员数
//! - **系统菜单贡献**：注册到「系统插件」分区，菜单项「部门管理」，排序权重 20
//! - **RBAC 基础维度**：组织树与责任域作为权限模型的基础数据维度
//! - **跨平台兼容**：兼容 web 与 desktop 两种运行环境
//!
//! ## 关键类型
//!
//! - [`OrganizationStarter`] — 实现 [`az_plugin_registry::PluginStarter`] 的插件描述结构体
//! - [`register_organization`] — `#[az_starter]` 标记的注册入口函数
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

struct OrganizationStarter;

impl PluginStarter for OrganizationStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "organization".to_string(),
            name: "组织中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "维护部门、团队归属与责任人。".to_string(),
            tags: vec!["system".to_string(), "org".to_string()],
            icon: Some("building".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "部门管理".to_string(),
                page_id: "departments".to_string(),
                order: 20,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "departments".to_string(),
                title: "部门管理".to_string(),
                subtitle: "组织树和责任域作为 RBAC 的基础维度。".to_string(),
                schema: PageSchema::Table(TableSchema {
                    columns: vec![
                        "部门".to_string(),
                        "上级".to_string(),
                        "负责人".to_string(),
                        "成员".to_string(),
                    ],
                    rows: vec![
                        TableRow {
                            cells: vec![
                                "平台工程".to_string(),
                                "技术中心".to_string(),
                                "zjarlin".to_string(),
                                "8".to_string(),
                            ],
                        },
                        TableRow {
                            cells: vec![
                                "资料运营".to_string(),
                                "运营中心".to_string(),
                                "mika".to_string(),
                                "6".to_string(),
                            ],
                        },
                    ],
                    empty_message: "暂无部门。".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        }
    }
}

#[az_starter]
pub fn register_organization() -> Box<dyn PluginStarter> {
    Box::new(OrganizationStarter)
}
