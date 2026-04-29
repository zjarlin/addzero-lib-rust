use addzero_plugin_contract::{
    MarkdownSchema, PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution, PluginPage,
};
use addzero_plugin_registry::{PluginRegistry, PluginStarter};

struct DemoStarter;

impl PluginStarter for DemoStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "demo".to_string(),
            name: "Demo".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "demo starter".to_string(),
            tags: vec!["system".to_string()],
            icon: None,
            compatibility: vec!["web".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "Demo Page".to_string(),
                page_id: "index".to_string(),
                order: 10,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "index".to_string(),
                title: "Demo Page".to_string(),
                subtitle: "subtitle".to_string(),
                schema: PageSchema::Markdown(MarkdownSchema {
                    body: "hello".to_string(),
                }),
            }],
        }
    }
}

#[test]
fn registry_resolves_system_pages_and_navigation() {
    let mut registry = PluginRegistry::new(vec![DemoStarter.descriptor()]);
    registry.replace_instances(vec![]);

    let sections = registry.plugin_navigation();
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].items[0].href, "/system/demo/index");

    let resolved = registry
        .resolve_system_page("demo", "index")
        .expect("page should resolve");
    assert_eq!(resolved.plugin_name, "Demo");
    assert_eq!(resolved.title, "Demo Page");
}
