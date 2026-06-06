#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::{
        CatalogItemContribution, CatalogItemKind, CatalogProviderContribution, CatalogSource,
        ContributionSet, NavItemContribution, PageContribution, PageRenderer, PluginActivation,
        PluginDescriptor, PluginKind, contributions_to_json, descriptor_to_json,
    };

    wit_bindgen::generate!({
        path: "../../wit",
        world: "az-aio-plugin",
    });

    struct HelloWasm;

    impl Guest for HelloWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            contributions_to_json(&ContributionSet {
                nav_items: vec![NavItemContribution {
                    id: "hello-wasm.nav".to_string(),
                    label: "外部组件示例".to_string(),
                    icon: "◇".to_string(),
                    route: "/hello-wasm".to_string(),
                    order: 900,
                }],
                pages: vec![PageContribution {
                    route: "/hello-wasm".to_string(),
                    title: "外部组件示例".to_string(),
                    subtitle: "外部 WIT 组件已通过描述符接入。".to_string(),
                    renderer: PageRenderer::Placeholder,
                    placeholder_mark: "◇".to_string(),
                    order: 900,
                }],
                ui_contributions: Vec::new(),
                backend_apis: Vec::new(),
                toolbar_actions: Vec::new(),
                catalog_providers: vec![CatalogProviderContribution {
                    id: "hello-wasm.catalog".to_string(),
                    label: "外部组件".to_string(),
                    order: 90,
                    items: vec![CatalogItemContribution {
                        id: "hello-wasm.catalog.item".to_string(),
                        name: "外部组件示例".to_string(),
                        description: "最小化组件模型插件描述符。".to_string(),
                        section: "外部组件".to_string(),
                        icon: "◇".to_string(),
                        accent_class: "plugin-icon--git".to_string(),
                        kind: CatalogItemKind::Plugin,
                        source: CatalogSource::Wasm,
                        installed: true,
                        tags: Vec::new(),
                        permissions: Vec::new(),
                        path: None,
                    }],
                }],
                settings_sections: Vec::new(),
                shell_entries: Vec::new(),
                generated_files: Vec::new(),
            })
            .map_err(|error| error.to_string())
        }

        fn on_load() -> Result<(), String> {
            Ok(())
        }

        fn on_enable() -> Result<(), String> {
            Ok(())
        }

        fn on_disable() -> Result<(), String> {
            Ok(())
        }

        fn on_unload() -> Result<(), String> {
            Ok(())
        }
    }

    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: "examples/hello-wasm".to_string(),
            name: "外部组件示例".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "用于验证描述符式插件加载的最小 WIT 组件。".to_string(),
            activation: PluginActivation::Eager,
            priority: 100,
            dependencies: Vec::new(),
            capabilities: vec!["nav-items".to_string(), "pages".to_string()],
            permissions: Vec::new(),
            kind: PluginKind::WasmComponent,
        }
    }

    export!(HelloWasm);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn build_hint() -> &'static str {
    "请把该 crate 构建为 wasm32 组件，再通过 az-aio-plugin-host 加载。"
}
