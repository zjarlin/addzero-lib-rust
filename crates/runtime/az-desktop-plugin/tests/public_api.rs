use az_derive_aliases::{apply, plain_default};
use std::sync::Arc;

use az_desktop_plugin::{
    DesktopActionOutcome, DesktopBranchRegistration, DesktopContributions,
    DesktopDomainRegistration, DesktopEvent, DesktopExecContext, DesktopHostRegistry,
    DesktopHostServices, DesktopInitContext, DesktopPageContributionSpec, DesktopPageRegistration,
    DesktopPageRole, DesktopProviderTestResult, DesktopRenderLayer, DesktopShellSnapshot,
    DesktopSummaryCardRegistration, DesktopToolbarActionRegistration, DesktopToolbarActionSpec,
    EventPropagation,
};
use az_drive_agent::agent::ListTrackedOptions;
use uuid::Uuid;

#[apply(plain_default)]
struct FakeServices;

impl DesktopHostServices for FakeServices {
    fn load_drive_snapshot(&self) -> Result<az_desktop_plugin::DesktopDriveSnapshot, String> {
        Ok(az_desktop_plugin::DesktopDriveSnapshot::default())
    }

    fn drive_host_path(&self, _path: &str) -> Result<String, String> {
        Ok(String::new())
    }

    fn drive_unhost_path(&self, _path: &str) -> Result<String, String> {
        Ok(String::new())
    }

    fn drive_sync_once(&self) -> Result<String, String> {
        Ok(String::new())
    }

    fn drive_retry_queue(&self) -> Result<String, String> {
        Ok(String::new())
    }

    fn drive_pull_remote(
        &self,
        _path: Option<&str>,
    ) -> Result<Vec<az_drive_agent::agent::PullRemoteItem>, String> {
        Ok(Vec::new())
    }

    fn list_tracked(
        &self,
        _path: Option<&str>,
        _options: ListTrackedOptions,
    ) -> Result<Vec<az_drive_agent::agent::TrackedItem>, String> {
        Ok(Vec::new())
    }

    fn drive_conflicts(&self) -> Result<Vec<az_drive_store::api::DriveConflict>, String> {
        Ok(Vec::new())
    }

    fn drive_sync_queue(
        &self,
        _status: Option<az_drive_store::api::DriveSyncTaskStatus>,
    ) -> Result<Vec<az_drive_store::api::DriveSyncQueueItem>, String> {
        Ok(Vec::new())
    }

    fn list_assets(
        &self,
        _kind: Option<az_assets::types::AssetKind>,
    ) -> Result<Vec<az_assets::types::Asset>, String> {
        Ok(Vec::new())
    }

    fn asset_graph(&self) -> Result<az_assets::types::AssetGraph, String> {
        Ok(az_assets::types::AssetGraph::default())
    }

    fn upsert_asset(
        &self,
        _input: az_assets::types::AssetUpsert,
    ) -> Result<az_assets::types::Asset, String> {
        Err("not implemented".to_string())
    }

    fn delete_asset(&self, _id: Uuid) -> Result<(), String> {
        Ok(())
    }

    fn list_provider_configs(&self) -> Result<Vec<az_assets::types::AiModelProvider>, String> {
        Ok(Vec::new())
    }

    fn upsert_provider(
        &self,
        _input: az_assets::types::AiModelProviderUpsert,
    ) -> Result<az_assets::types::AiModelProvider, String> {
        Err("not implemented".to_string())
    }

    fn test_provider(
        &self,
        _provider: az_assets::types::AiProviderKind,
    ) -> Result<DesktopProviderTestResult, String> {
        Ok(DesktopProviderTestResult::default())
    }

    fn software_catalog(&self) -> Result<az_software_catalog::model::SoftwareCatalogDto, String> {
        Err("unavailable".to_string())
    }

    fn software_save_entry(
        &self,
        _input: az_software_catalog::model::SoftwareEntryInput,
    ) -> Result<az_software_catalog::model::SoftwareEntryDto, String> {
        Err("unavailable".to_string())
    }

    fn software_fetch_metadata(
        &self,
        _input: az_software_catalog::model::SoftwareMetadataFetchInput,
    ) -> Result<az_software_catalog::model::SoftwareMetadataDto, String> {
        Err("unavailable".to_string())
    }

    fn open_path(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn init_context_records_plugin_contributions() {
    let mut ctx = DesktopInitContext::new();
    ctx.set_current_plugin("demo");
    const ACTIONS: &[DesktopToolbarActionSpec] = &[
        DesktopToolbarActionSpec::primary("drive.refresh", "Refresh", "Reload snapshot", 10),
        DesktopToolbarActionSpec::secondary("drive.sync", "Sync", "Run sync", 20),
    ];
    ctx.register_page_contribution(DesktopPageContributionSpec {
        domain_id: "ops",
        domain_label: "Operations",
        domain_order: 10,
        branch_id: "drive-branch",
        parent_branch_id: None,
        branch_label: "Drive",
        branch_order: 10,
        page_id: "drive-home",
        page_title: "Drive Center",
        page_subtitle: "Sync roots",
        route: "/drive",
        page_order: 10,
        summary_card_id: "drive-card",
        summary_title: "Drive Center",
        summary: "Sync roots",
        summary_order: 10,
        toolbar_actions: ACTIONS,
    });
    ctx.register_command("drive.refresh", "Refresh");

    let contributions = ctx.into_contributions();
    // 确认 setup 阶段的注册项会被完整记录，并保留来源插件。
    assert_eq!(contributions.domains.len(), 1);
    assert_eq!(contributions.pages[0].plugin_name, "demo");
    assert_eq!(contributions.toolbar_actions.len(), 2);
    assert_eq!(contributions.toolbar_actions[0].action_id, "drive.refresh");
    assert!(contributions.toolbar_actions[0].primary);
    assert_eq!(
        contributions.toolbar_actions[1].route.as_deref(),
        Some("/drive")
    );
}

#[test]
fn host_registry_queries_plugin_setup_contributions() {
    let registry = DesktopHostRegistry::from(DesktopContributions {
        domains: vec![DesktopDomainRegistration {
            plugin_name: "demo".to_string(),
            id: "ops".to_string(),
            label: "Operations".to_string(),
            order: 10,
            default_route: "/drive".to_string(),
        }],
        branches: vec![DesktopBranchRegistration {
            plugin_name: "demo".to_string(),
            id: "storage".to_string(),
            domain_id: "ops".to_string(),
            parent_id: None,
            label: "Storage".to_string(),
            order: 10,
        }],
        pages: vec![DesktopPageRegistration {
            plugin_name: "demo".to_string(),
            id: "drive".to_string(),
            domain_id: "ops".to_string(),
            parent_branch_id: Some("storage".to_string()),
            title: "Drive".to_string(),
            subtitle: "drive page".to_string(),
            route: "/drive".to_string(),
            order: 10,
            role: DesktopPageRole::Owner,
        }],
        toolbar_actions: vec![DesktopToolbarActionRegistration {
            plugin_name: "demo".to_string(),
            route: Some("/drive".to_string()),
            action_id: "drive.refresh".to_string(),
            label: "Refresh".to_string(),
            tooltip: "refresh".to_string(),
            order: 10,
            primary: false,
        }],
        summary_cards: vec![DesktopSummaryCardRegistration {
            plugin_name: "demo".to_string(),
            card_id: "drive-card".to_string(),
            title: "Drive".to_string(),
            summary: "Drive summary".to_string(),
            route: "/drive".to_string(),
            order: 10,
        }],
        commands: Vec::new(),
    });

    // 注册表是 shell 查询唯一入口，必须能从贡献项还原导航、页面和动作关系。
    assert_eq!(registry.domains().len(), 1);
    assert_eq!(registry.root_branches_for_domain("ops").len(), 1);
    assert_eq!(registry.pages_for_branch("storage").len(), 1);
    assert_eq!(registry.toolbar_actions_for_route("/drive").len(), 1);
    assert_eq!(registry.summary_cards().len(), 1);
}

#[test]
fn desktop_event_helpers_keep_route_guards_explicit() {
    let route_change = DesktopEvent::RouteChanged {
        route: "/drive".to_string(),
    };
    let route_refresh = DesktopEvent::RefreshRequested {
        route: Some("/drive".to_string()),
    };
    let global_refresh = DesktopEvent::RefreshRequested { route: None };
    let action = DesktopEvent::ActionInvoked {
        route: "/drive".to_string(),
        action_id: "drive.sync".to_string(),
    };

    // 页面插件依赖这些 helper 统一宿主事件守卫，业务 action 仍在插件内显式匹配。
    assert!(route_change.refreshes_route("/drive"));
    assert!(route_refresh.refreshes_route("/drive"));
    assert!(!route_refresh.refreshes_route("/config"));
    assert!(global_refresh.is_global_refresh());
    assert_eq!(action.action_id_for_route("/drive"), Some("drive.sync"));
    assert_eq!(action.action_id_for_route("/config"), None);
}

#[test]
fn action_outcome_updates_feedback_and_propagation() {
    let services: Arc<dyn DesktopHostServices> = Arc::new(FakeServices);
    let (ctx, feedback) = DesktopExecContext::new(
        services,
        DesktopShellSnapshot {
            current_route: "/drive".to_string(),
            ..DesktopShellSnapshot::default()
        },
    );

    // 未处理动作继续传播；已处理和错误都停止传播，提示通过反馈通道交给宿主。
    assert_eq!(
        ctx.apply_action_outcome(Ok(DesktopActionOutcome::Ignored)),
        EventPropagation::Continue
    );
    assert_eq!(
        ctx.apply_action_outcome(Ok(DesktopActionOutcome::Handled)),
        EventPropagation::Stop
    );
    assert_eq!(
        ctx.apply_action_outcome(Ok(DesktopActionOutcome::notified("done"))),
        EventPropagation::Stop
    );
    assert_eq!(feedback.borrow().notice.as_deref(), Some("done"));
    assert_eq!(
        ctx.apply_action_outcome(Err("failed".to_string())),
        EventPropagation::Stop
    );
    assert_eq!(feedback.borrow().notice.as_deref(), Some("failed"));
}

#[test]
fn exec_context_feedback_updates_are_captured() {
    let services: Arc<dyn DesktopHostServices> = Arc::new(FakeServices);
    let (ctx, feedback) = DesktopExecContext::new(
        services,
        DesktopShellSnapshot {
            current_route: "/drive".to_string(),
            ..DesktopShellSnapshot::default()
        },
    );
    let event = DesktopEvent::RefreshRequested {
        route: Some("/drive".to_string()),
    };
    assert!(matches!(
        match event {
            DesktopEvent::RefreshRequested { .. } => EventPropagation::Continue,
            _ => EventPropagation::Stop,
        },
        EventPropagation::Continue
    ));

    ctx.notify("ok");
    ctx.request_refresh();
    ctx.set_selected_entity(Some("item-1".to_string()));
    ctx.navigate_to("/config");

    let feedback = feedback.borrow().clone();
    // 插件只能通过反馈通道影响 shell 状态，不能直接修改宿主。
    assert_eq!(feedback.notice.as_deref(), Some("ok"));
    assert!(feedback.refresh_requested);
    assert_eq!(feedback.selected_entity, Some(Some("item-1".to_string())));
    assert_eq!(feedback.route_override.as_deref(), Some("/config"));
}

#[test]
fn desktop_protocol_enums_expose_stable_codes() {
    // 这些 code 会被插件协议和 UI 绑定使用，变更必须显式暴露。
    assert_eq!(
        DesktopRenderLayer::ALL,
        &[
            DesktopRenderLayer::Main,
            DesktopRenderLayer::Inspector,
            DesktopRenderLayer::Overlay
        ]
    );
    assert_eq!(EventPropagation::Stop.code(), "stop");
    assert_eq!(
        DesktopRenderLayer::from_code("inspector"),
        Some(DesktopRenderLayer::Inspector)
    );
    assert_eq!(DesktopPageRole::Contributor.as_str(), "contributor");
}
