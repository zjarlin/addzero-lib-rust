use addzero_plugin_contract::{
    PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage, TableRow,
    TableSchema,
};
use addzero_plugin_macros::addzero_starter;
use addzero_plugin_registry::PluginStarter;

pub fn ensure_linked() {}

struct DictionaryStarter;

impl PluginStarter for DictionaryStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "dictionary".to_string(),
            name: "字典中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "统一维护系统枚举和值域，闪念只是 note_type 的一个值。".to_string(),
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
                subtitle: "笔记体系通过 note_type 管理闪念、笔记、知识库与 Skill。".to_string(),
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
                                "闪念".to_string(),
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
        }
    }
}

#[addzero_starter]
pub fn register_dictionary() -> Box<dyn PluginStarter> {
    Box::new(DictionaryStarter)
}
