//! `az-starter-menu` —— 菜单中心系统插件。
//!
//! 本 crate 实现了 addzero 系统级「菜单中心」插件，负责统一输出宿主菜单结构与插件挂载说明。
//! 通过 [`PluginStarter`] trait 向插件注册中心声明自身身份与页面内容，
//! 使管理员能够了解插件挂载机制：固定页 → 系统插件 → 业务实例，新增插件不再修改主路由表。
//!
//! ## 主要能力
//!
//! - 以 Markdown 页面展示菜单挂载机制说明
//! - 明确声明"插件只补描述与注册，不改主路由"的架构约束
//! - 注册到「系统插件」菜单分区，挂载「菜单挂载」入口页
//!
//! ## 关键类型
//!
//! - [`MenuStarter`] —— 实现 [`PluginStarter`] 的内部结构体，定义插件描述符与 Markdown 页面
//! - `register_menu()` —— 由 `#[az_starter]` 宏标记的注册入口函数

use az_plugin_contract::{
    MarkdownSchema, PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage,
};
use az_plugin_macros::az_starter;
use az_plugin_registry::PluginStarter;

pub fn ensure_linked() {}

struct MenuStarter;

impl PluginStarter for MenuStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "menu".to_string(),
            name: "菜单中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "统一输出宿主菜单与插件挂载说明。".to_string(),
            tags: vec!["system".to_string(), "menu".to_string()],
            icon: Some("menu".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "菜单挂载".to_string(),
                page_id: "mounting".to_string(),
                order: 40,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "mounting".to_string(),
                title: "菜单挂载".to_string(),
                subtitle: "system starter 与 business plugin 都不再修改 mainapp 路由表。".to_string(),
                schema: PageSchema::Markdown(MarkdownSchema {
                    body: "菜单由宿主壳子统一解析：固定页 -> 系统插件 -> 业务实例。新增插件只补描述与注册，不再改主路由。".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        }
    }
}

#[az_starter]
pub fn register_menu() -> Box<dyn PluginStarter> {
    Box::new(MenuStarter)
}
