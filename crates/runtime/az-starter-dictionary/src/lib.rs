//! `az-starter-dictionary` —— 字典中心系统插件。
//!
//! 本 crate 实现了 addzero 系统级「字典中心」插件，负责统一维护系统枚举与值域。
//! 通过 [`PluginStarter`] trait 向插件注册中心声明自身身份、菜单入口与页面结构，
//! 使宿主壳子无需硬编码即可自动发现并挂载字典管理页面。
//!
//! ## 主要能力
//!
//! - 以表格形式暴露字典项列表（字典编码、值、显示名、用途）
//! - 默认内置 `note_type` 字典，区分智能体工作台（`flash`）与 Skill（`skill`）两种类型
//! - 注册到「系统插件」菜单分区，挂载「字典管理」入口页
//!
//! ## 关键类型
//!
//! - [`DictionaryStarter`] —— 实现 [`PluginStarter`] 的内部结构体，定义插件描述符与页面 schema
//! - `register_dictionary()` —— 由 `#[az_starter]` 宏标记的注册入口函数

use az_plugin_contract::{
    PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage, TableRow,
    TableSchema,
};
use az_plugin_macros::az_starter;
use az_plugin_registry::PluginStarter;

pub fn ensure_linked() {}

struct DictionaryStarter;

impl PluginStarter for DictionaryStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "dictionary".to_string(),
            name: "字典中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "统一维护系统枚举和值域，智能体工作台只是 note_type 的一个值。".to_string(),
            tags: vec!["system".to_string(), "dictionary".to_string()],
            icon: Some("book-key".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "字典管理".to_string(),
                page_id: "note-types".to_string(),
                order: 30,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "note-types".to_string(),
                title: "字典管理".to_string(),
                subtitle: "笔记体系通过 note_type 管理智能体工作台、笔记、知识库与 Skill。"
                    .to_string(),
                schema: PageSchema::Table(TableSchema {
                    columns: vec![
                        "字典编码".to_string(),
                        "值".to_string(),
                        "显示名".to_string(),
                        "用途".to_string(),
                    ],
                    rows: vec![
                        TableRow {
                            cells: vec![
                                "note_type".to_string(),
                                "flash".to_string(),
                                "智能体工作台".to_string(),
                                "统一新增入口默认值".to_string(),
                            ],
                        },
                        TableRow {
                            cells: vec![
                                "note_type".to_string(),
                                "skill".to_string(),
                                "Skill".to_string(),
                                "沉淀为可执行能力".to_string(),
                            ],
                        },
                    ],
                    empty_message: "暂无字典项。".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        }
    }
}

#[az_starter]
pub fn register_dictionary() -> Box<dyn PluginStarter> {
    Box::new(DictionaryStarter)
}
