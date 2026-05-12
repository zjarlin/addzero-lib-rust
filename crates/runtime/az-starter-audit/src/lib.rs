//! 审计中心系统插件启动器。
//!
//! 通过 `az_plugin_macros::az_starter` 宏自动注册为系统插件，
//! 提供审计日志页面，记录插件安装、实例创建与权限变更等事件。

use az_plugin_contract::{
    BoardSchema, MetricCard, PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution,
    PluginPage, RecordGroup, RecordItem,
};
use az_plugin_macros::az_starter;
use az_plugin_registry::PluginStarter;

pub fn ensure_linked() {}

struct AuditStarter;

impl PluginStarter for AuditStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "audit".to_string(),
            name: "审计中心".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::System,
            summary: "记录插件安装、实例创建与权限变更。".to_string(),
            tags: vec!["system".to_string(), "audit".to_string()],
            icon: Some("shield".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "系统插件".to_string(),
                label: "审计日志".to_string(),
                page_id: "events".to_string(),
                order: 50,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "events".to_string(),
                title: "审计日志".to_string(),
                subtitle: "宿主审计总线为系统插件和业务插件统一记账。".to_string(),
                schema: PageSchema::Board(BoardSchema {
                    metrics: vec![
                        MetricCard {
                            label: "今日事件".to_string(),
                            value: "12".to_string(),
                            detail: "含插件安装与实例创建".to_string(),
                        },
                        MetricCard {
                            label: "高风险".to_string(),
                            value: "0".to_string(),
                            detail: "当前没有未处理告警".to_string(),
                        },
                    ],
                    groups: vec![RecordGroup {
                        title: "最近日志".to_string(),
                        items: vec![
                            RecordItem {
                                title: "memory-manager 已安装".to_string(),
                                detail: "业务插件成功进入应用商店。".to_string(),
                                meta: "plugin-runtime".to_string(),
                            },
                            RecordItem {
                                title: "资料员管理系统实例已创建".to_string(),
                                detail: "新实例获得独立路由与命名空间。".to_string(),
                                meta: "instance".to_string(),
                            },
                        ],
                    }],
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        }
    }
}

#[az_starter]
pub fn register_audit() -> Box<dyn PluginStarter> {
    Box::new(AuditStarter)
}
