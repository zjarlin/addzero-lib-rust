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
