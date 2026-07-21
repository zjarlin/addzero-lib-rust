use az_aio_platform::plugin::contract::{
    BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, ToolbarActionContribution, UiContribution,
    UiContributionSlot,
};

pub const PLUGIN_ID: &str = "linux";
pub const ROUTE: &str = "/linux";
pub const RENDERER_ID: &str = "linux.page";

pub fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: PLUGIN_ID.to_string(),
        name: "Linux".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "客户端侧 Linux 服务器环境搭建、Ubuntu 适配、SSH 配置和 curl 配对入口。"
            .to_string(),
        activation: PluginActivation::Eager,
        priority: 870,
        dependencies: Vec::new(),
        capabilities: vec![
            "dioxus-ui-contract-page".to_string(),
            "axum-api".to_string(),
            "linux-onboarding-client".to_string(),
            "ubuntu-adapter".to_string(),
            "ssh-pairing-plan".to_string(),
        ],
        permissions: vec![
            "render-bootstrap-script".to_string(),
            "outbound-ssh-planned".to_string(),
        ],
        kind: PluginKind::Native,
    }
}

pub fn contributions() -> ContributionSet {
    ContributionSet {
        nav_items: vec![NavItemContribution {
            id: "linux.nav".to_string(),
            label: "Linux".to_string(),
            icon: "🐧".to_string(),
            route: ROUTE.to_string(),
            order: 55,
        }],
        pages: vec![PageContribution {
            route: ROUTE.to_string(),
            title: "Linux".to_string(),
            subtitle: "Ubuntu 环境搭建 · SSH 配对 · curl 引导".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            placeholder_mark: "🐧".to_string(),
            order: 55,
        }],
        client_pages: Vec::new(),
        ui_contributions: vec![UiContribution {
            id: "linux.ui.content".to_string(),
            slot: UiContributionSlot::Content,
            label: "Linux Content".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            route: Some(ROUTE.to_string()),
            order: 10,
        }],
        backend_apis: vec![
            backend_api(
                "linux.api.status",
                "GET",
                "/api/linux/status",
                "Linux Status",
                "返回客户端插件状态、契约版本和可用 API。",
                10,
            ),
            backend_api(
                "linux.api.profiles",
                "GET",
                "/api/linux/profiles",
                "Linux Profiles",
                "返回模块化 Linux 发行版适配器，当前先支持 Ubuntu。",
                20,
            ),
            backend_api(
                "linux.api.setup-catalog",
                "GET",
                "/api/linux/setup-catalog",
                "Setup Catalog",
                "读取 /Users/zjarlin/aio/note/环境搭建 中可复用的环境脚本命令。",
                30,
            ),
            backend_api(
                "linux.api.bootstrap-plan",
                "POST",
                "/api/linux/bootstrap-plan",
                "Bootstrap Plan",
                "生成手动 curl 引导命令、SSH config 和远端配对步骤。",
                50,
            ),
            backend_api(
                "linux.api.bootstrap-script",
                "GET",
                "/api/linux/bootstrap-script",
                "Bootstrap Script",
                "远端 Ubuntu 服务器初始不可连时复制执行的 curl 脚本入口。",
                40,
            ),
        ],
        toolbar_actions: vec![
            toolbar_action("linux.refresh", "Refresh", "RefreshCw", false, 10),
            toolbar_action("linux.generate-plan", "Generate Plan", "Terminal", true, 20),
        ],
        catalog_providers: Vec::new(),
        settings_sections: Vec::new(),
        shell_entries: Vec::new(),
        generated_files: Vec::new(),
    }
}

fn backend_api(
    id: &str,
    method: &str,
    path: &str,
    label: &str,
    description: &str,
    order: i32,
) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        order,
    }
}

fn toolbar_action(
    id: &str,
    label: &str,
    icon: &str,
    primary: bool,
    order: i32,
) -> ToolbarActionContribution {
    ToolbarActionContribution {
        id: id.to_string(),
        route: Some(ROUTE.to_string()),
        label: label.to_string(),
        icon: icon.to_string(),
        primary,
        order,
    }
}
