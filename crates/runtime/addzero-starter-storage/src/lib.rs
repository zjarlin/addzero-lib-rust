use addzero_plugin_contract::{
    PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage, TableRow,
    TableSchema,
};
use addzero_plugin_macros::addzero_starter;
use addzero_plugin_registry::PluginStarter;

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
                subtitle: "业务插件包以 .azplugin 发布并进入 catalog。".to_string(),
                schema: PageSchema::Table(TableSchema {
                    columns: vec!["目录".to_string(), "内容".to_string(), "说明".to_string()],
                    rows: vec![
                        TableRow {
                            cells: vec![
                                "catalog/".to_string(),
                                "*.azplugin".to_string(),
                                "应用商店可安装包".to_string(),
                            ],
                        },
                        TableRow {
                            cells: vec![
                                "installed/".to_string(),
                                "解压后的包内容".to_string(),
                                "实例化运行前的宿主缓存".to_string(),
                            ],
                        },
                    ],
                    empty_message: "暂无包仓库信息。".to_string(),
                }),
            }],
        }
    }
}

#[addzero_starter]
pub fn register_storage() -> Box<dyn PluginStarter> {
    Box::new(StorageStarter)
}
