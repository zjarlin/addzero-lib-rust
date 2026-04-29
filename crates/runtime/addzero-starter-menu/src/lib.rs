use addzero_plugin_contract::{
    MarkdownSchema, PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage,
};
use addzero_plugin_macros::addzero_starter;
use addzero_plugin_registry::PluginStarter;

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
        }
    }
}

#[addzero_starter]
pub fn register_menu() -> Box<dyn PluginStarter> {
    Box::new(MenuStarter)
}
