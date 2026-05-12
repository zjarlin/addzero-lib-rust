//! `az-starter-identity` —— 用户中心系统插件。
//!
//! 本 crate 实现了 addzero 系统级「用户中心」插件，负责用户、角色与开发环境默认登录管理。
//! 通过 [`PluginStarter`] trait 向插件注册中心声明自身身份、菜单入口与页面结构，
//! 使宿主壳子无需硬编码即可自动发现并挂载用户管理页面。
//!
//! ## 主要能力
//!
//! - 以表格形式暴露用户列表（用户名、角色、状态、登录源）
//! - 默认内置 `admin`（管理员）与 `luna`（审核员）两个示例用户
//! - 开发环境默认使用 `admin/admin` 本地登录；生产环境通过环境变量配置真实登录
//! - 注册到「系统插件」菜单分区，挂载「用户管理」入口页
//!
//! ## 关键类型
//!
//! - [`IdentityStarter`] —— 实现 [`PluginStarter`] 的内部结构体，定义插件描述符与页面 schema
//! - `register_identity()` —— 由 `#[az_starter]` 宏标记的注册入口函数

use az_plugin_contract::{
    PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage, TableRow,
    TableSchema,
};
use az_plugin_macros::az_starter;
use az_plugin_registry::PluginStarter;

pub fn ensure_linked() {}

struct IdentityStarter;

impl PluginStarter for IdentityStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "identity".to_string(),
            name: "用户中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "负责用户、角色与开发环境默认登录。".to_string(),
            tags: vec!["system".to_string(), "auth".to_string()],
            icon: Some("users".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "用户管理".to_string(),
                page_id: "users".to_string(),
                order: 10,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "users".to_string(),
                title: "用户管理".to_string(),
                subtitle: "开发环境默认 admin / admin；生产环境请改用环境变量配置的真实登录。"
                    .to_string(),
                schema: PageSchema::Table(TableSchema {
                    columns: vec![
                        "用户名".to_string(),
                        "角色".to_string(),
                        "状态".to_string(),
                        "登录源".to_string(),
                    ],
                    rows: vec![
                        TableRow {
                            cells: vec![
                                "admin".to_string(),
                                "管理员".to_string(),
                                "启用".to_string(),
                                "dev-local".to_string(),
                            ],
                        },
                        TableRow {
                            cells: vec![
                                "luna".to_string(),
                                "审核员".to_string(),
                                "启用".to_string(),
                                "github".to_string(),
                            ],
                        },
                    ],
                    empty_message: "暂无用户。".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        }
    }
}

#[az_starter]
pub fn register_identity() -> Box<dyn PluginStarter> {
    Box::new(IdentityStarter)
}
