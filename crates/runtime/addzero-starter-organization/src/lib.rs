use addzero_plugin_contract::{
    PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage, TableRow,
    TableSchema,
};
use addzero_plugin_macros::addzero_starter;
use addzero_plugin_registry::PluginStarter;

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
        }
    }
}

#[addzero_starter]
pub fn register_organization() -> Box<dyn PluginStarter> {
    Box::new(OrganizationStarter)
}
