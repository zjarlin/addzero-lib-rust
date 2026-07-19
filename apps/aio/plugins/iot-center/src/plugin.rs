//! 物联网中心 AIO 子插件注册。

use std::sync::Arc;

use anyhow::anyhow;
use az_aio_platform::plugin::api::{
    AdminCliContribution, AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree,
    BackendApiContribution, ContributionSet, DynAdminPluginProvider, NativePluginContext,
    NativePluginProvider, NativePluginRuntime, NativeUiRenderer, NavItemContribution,
    PageContribution, PluginActivation, PluginDependency, PluginDescriptor, PluginKind,
    UiContribution, UiContributionSlot,
};
use az_engine::EngineStore;
use rudi::Singleton;

use crate::{
    contract::{
        APPLY_TEMPLATE_PATH, DEVICES_PATH, OP_DEVICES_CREATE, OP_TEMPLATE_APPLY, ROUTE, STATUS_PATH,
    },
    routes::{IotApiState, iot_router},
    service::IotService,
    state::install_service,
    ui::IotCenterPage,
};

const PLUGIN_ID: &str = "iot-center";
const RENDERER_ID: &str = "iot-center.page";

/// 低代码驱动的物联网中心插件。
#[derive(Default)]
pub struct IotCenterPlugin;

impl NativePluginProvider for IotCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PLUGIN_ID.to_string(),
            name: "物联网中心".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "产品、网关、设备、遥测和告警统一由低代码 EngineStore 驱动。".to_string(),
            activation: PluginActivation::Eager,
            priority: 500,
            dependencies: vec![PluginDependency {
                id: "lowcode".to_string(),
                optional: false,
            }],
            capabilities: vec![
                "dioxus-ui-contract-page".to_string(),
                "engine-meta-model".to_string(),
                "iot-device-health".to_string(),
                "postgres-persistence".to_string(),
            ],
            permissions: vec!["PostgreSQL engine_* 表读写".to_string()],
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "iot.nav".to_string(),
                label: "物联网".to_string(),
                icon: "⌁".to_string(),
                route: ROUTE.to_string(),
                order: 40,
            }],
            pages: vec![PageContribution {
                route: ROUTE.to_string(),
                title: "物联网中心".to_string(),
                subtitle: "产品、网关、设备、遥测、告警".to_string(),
                renderer_id: RENDERER_ID.to_string(),
                placeholder_mark: "⌁".to_string(),
                order: 40,
            }],
            ui_contributions: vec![UiContribution {
                id: "iot.ui.content".to_string(),
                slot: UiContributionSlot::Content,
                label: "物联网中心内容区".to_string(),
                renderer_id: RENDERER_ID.to_string(),
                route: Some(ROUTE.to_string()),
                order: 40,
            }],
            backend_apis: vec![
                api("iot.status", "GET", STATUS_PATH, "物联网状态", 10),
                api(
                    OP_TEMPLATE_APPLY,
                    "POST",
                    APPLY_TEMPLATE_PATH,
                    "初始化物联网模板",
                    20,
                ),
                api(OP_DEVICES_CREATE, "POST", DEVICES_PATH, "新建设备", 30),
            ],
            ..ContributionSet::default()
        })
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: "iot".to_string(),
                label: "物联网".to_string(),
                default_href: "/iot?view=devices".to_string(),
                order: 250,
                menus: vec![AdminMenuNode {
                    id: "iot.device-management".to_string(),
                    kind: AdminMenuNodeKind::Branch,
                    label: "设备管理".to_string(),
                    href: ROUTE.to_string(),
                    icon: "⌁".to_string(),
                    order: 10,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["iot:read".to_string()],
                    children: vec![
                        menu("iot.products", "产品模板", "products", "◫", 10),
                        menu("iot.devices", "设备管理", "devices", "◉", 20),
                        menu("iot.gateways", "网关管理", "gateways", "⌂", 30),
                        menu("iot.telemetry", "数据采集", "telemetry", "≋", 40),
                        menu("iot.alarms", "告警中心", "alarms", "△", 50),
                    ],
                }],
            }],
        }
    }

    fn admin_cli(&self) -> Vec<AdminCliContribution> {
        vec![
            AdminCliContribution {
                id: "iot.cli.template-apply".to_string(),
                label: "初始化物联网模板".to_string(),
                command: "az iot template apply --seed-demo".to_string(),
                resource_id: None,
                operation_id: Some(OP_TEMPLATE_APPLY.to_string()),
            },
            AdminCliContribution {
                id: "iot.cli.device-create".to_string(),
                label: "新建设备".to_string(),
                command: "az iot device create --request <device.json>".to_string(),
                resource_id: None,
                operation_id: Some(OP_DEVICES_CREATE.to_string()),
            },
        ]
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let shared_db = context
            .shared_db
            .ok_or_else(|| anyhow!("物联网中心启动需要共享 PostgreSQL Db"))?;
        let store = EngineStore::from_shared_db(shared_db.shared_handle());
        let service = IotService::new(store);
        install_service(service.clone());
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: IotCenterPage,
            }],
            router: iot_router(IotApiState::new(service)),
            startup: None,
        })
    }
}

#[Singleton(name = "iot-center")]
pub fn iot_center_plugin() -> DynAdminPluginProvider {
    Arc::new(IotCenterPlugin)
}

fn api(id: &str, method: &str, path: &str, label: &str, order: i32) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: format!("{label}: {path}"),
        order,
    }
}

fn menu(id: &str, label: &str, view: &str, icon: &str, order: i32) -> AdminMenuNode {
    let href = format!("/iot?view={view}");
    AdminMenuNode {
        id: id.to_string(),
        kind: AdminMenuNodeKind::Page,
        label: label.to_string(),
        href: href.clone(),
        icon: icon.to_string(),
        order,
        active_patterns: vec![href],
        permissions_any_of: vec!["iot:read".to_string()],
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_contributes_iot_domain_and_engine_contracts() -> anyhow::Result<()> {
        let plugin = IotCenterPlugin;
        let contributions = plugin.contributions()?;
        let menu = plugin.admin_menu(&contributions);

        // 关键断言：物联网必须作为 AIO 主轴子插件出现，并声明低代码引擎依赖。
        assert_eq!(menu.sections[0].domain_id, "iot");
        assert!(
            plugin
                .descriptor()
                .dependencies
                .iter()
                .any(|dependency| dependency.id == "lowcode" && !dependency.optional)
        );
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.id == OP_TEMPLATE_APPLY)
        );
        Ok(())
    }
}
