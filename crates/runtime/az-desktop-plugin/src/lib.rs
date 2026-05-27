#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::{cell::RefCell, cmp::Reverse, collections::BTreeMap, rc::Rc, sync::Arc};

use az_assets::{
    AiModelProvider, AiModelProviderUpsert, AiProviderKind, Asset, AssetGraph, AssetKind,
    AssetUpsert,
};
use az_derive_aliases::{
    apply, impl_from_match, plain_clone, plain_code_enum, plain_default_eq, plain_eq,
};
use az_drive_agent::{
    HostedStatus, ListTrackedOptions, LocalRootState, PullRemoteItem, TrackedItem,
};
use az_drive_store::{DriveConflict, DriveSyncQueueItem, DriveSyncTaskStatus};
use az_software_catalog::{
    SoftwareCatalogDto, SoftwareEntryDto, SoftwareEntryInput, SoftwareMetadataDto,
    SoftwareMetadataFetchInput,
};
use gpui::AnyElement;
use serde_json::Value;
use uuid::Uuid;

pub trait Plugin<InitContext, Event, ExecContext, ViewContext, RenderLayer> {
    fn name(&self) -> &'static str;

    fn setup(&mut self, _ctx: &mut InitContext) {}

    fn on_event(&mut self, _event: &Event, _ctx: &mut ExecContext) -> EventPropagation {
        EventPropagation::Continue
    }

    fn render(&mut self, _ctx: &mut ViewContext) -> Option<AnyElement> {
        None
    }

    fn priority(&self) -> i32 {
        0
    }

    fn render_layer(&self) -> RenderLayer;
}

pub type DesktopPlugin = dyn Plugin<
        DesktopInitContext,
        DesktopEvent,
        DesktopExecContext,
        DesktopViewContext,
        DesktopRenderLayer,
    >;

#[apply(plain_code_enum)]
pub enum EventPropagation {
    Continue,
    Stop,
}

#[apply(plain_code_enum)]
pub enum DesktopRenderLayer {
    Main,
    Inspector,
    Overlay,
}

#[apply(plain_code_enum)]
pub enum DesktopPageRole {
    Owner,
    Contributor,
}

#[apply(plain_default_eq)]
pub struct DesktopContributions {
    pub domains: Vec<DesktopDomainRegistration>,
    pub branches: Vec<DesktopBranchRegistration>,
    pub pages: Vec<DesktopPageRegistration>,
    pub toolbar_actions: Vec<DesktopToolbarActionRegistration>,
    pub summary_cards: Vec<DesktopSummaryCardRegistration>,
    pub commands: Vec<DesktopCommandRegistration>,
}

#[apply(plain_default_eq)]
pub struct DesktopInitContext {
    current_plugin: Option<String>,
    contributions: DesktopContributions,
}

impl DesktopInitContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_current_plugin(&mut self, plugin_name: impl Into<String>) {
        self.current_plugin = Some(plugin_name.into());
    }

    pub fn register_domain(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        order: i32,
        default_route: impl Into<String>,
    ) {
        self.contributions.domains.push(DesktopDomainRegistration {
            plugin_name: self.plugin_name(),
            id: id.into(),
            label: label.into(),
            order,
            default_route: default_route.into(),
        });
    }

    pub fn register_branch(
        &mut self,
        id: impl Into<String>,
        domain_id: impl Into<String>,
        parent_id: Option<impl Into<String>>,
        label: impl Into<String>,
        order: i32,
    ) {
        self.contributions.branches.push(DesktopBranchRegistration {
            plugin_name: self.plugin_name(),
            id: id.into(),
            domain_id: domain_id.into(),
            parent_id: parent_id.map(Into::into),
            label: label.into(),
            order,
        });
    }

    pub fn register_page(
        &mut self,
        id: impl Into<String>,
        domain_id: impl Into<String>,
        parent_branch_id: Option<impl Into<String>>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        route: impl Into<String>,
        order: i32,
    ) {
        self.register_page_with_role(
            DesktopPageRole::Owner,
            id,
            domain_id,
            parent_branch_id,
            title,
            subtitle,
            route,
            order,
        );
    }

    pub fn register_page_with_role(
        &mut self,
        role: DesktopPageRole,
        id: impl Into<String>,
        domain_id: impl Into<String>,
        parent_branch_id: Option<impl Into<String>>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        route: impl Into<String>,
        order: i32,
    ) {
        self.contributions.pages.push(DesktopPageRegistration {
            plugin_name: self.plugin_name(),
            id: id.into(),
            domain_id: domain_id.into(),
            parent_branch_id: parent_branch_id.map(Into::into),
            title: title.into(),
            subtitle: subtitle.into(),
            route: route.into(),
            order,
            role,
        });
    }

    pub fn register_toolbar_action(
        &mut self,
        route: Option<impl Into<String>>,
        action_id: impl Into<String>,
        label: impl Into<String>,
        tooltip: impl Into<String>,
        order: i32,
        primary: bool,
    ) {
        self.contributions
            .toolbar_actions
            .push(DesktopToolbarActionRegistration {
                plugin_name: self.plugin_name(),
                route: route.map(Into::into),
                action_id: action_id.into(),
                label: label.into(),
                tooltip: tooltip.into(),
                order,
                primary,
            });
    }

    pub fn register_summary_card(
        &mut self,
        card_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        route: impl Into<String>,
        order: i32,
    ) {
        self.contributions
            .summary_cards
            .push(DesktopSummaryCardRegistration {
                plugin_name: self.plugin_name(),
                card_id: card_id.into(),
                title: title.into(),
                summary: summary.into(),
                route: route.into(),
                order,
            });
    }

    pub fn register_command(&mut self, command_id: impl Into<String>, title: impl Into<String>) {
        self.contributions
            .commands
            .push(DesktopCommandRegistration {
                plugin_name: self.plugin_name(),
                command_id: command_id.into(),
                title: title.into(),
            });
    }

    #[must_use]
    pub fn contributions(&self) -> &DesktopContributions {
        &self.contributions
    }

    #[must_use]
    pub fn into_contributions(self) -> DesktopContributions {
        self.contributions
    }

    fn plugin_name(&self) -> String {
        self.current_plugin
            .clone()
            .unwrap_or_else(|| "unknown-plugin".to_string())
    }
}

#[apply(plain_eq)]
pub struct DesktopDomainRegistration {
    pub plugin_name: String,
    pub id: String,
    pub label: String,
    pub order: i32,
    pub default_route: String,
}

#[apply(plain_eq)]
pub struct DesktopBranchRegistration {
    pub plugin_name: String,
    pub id: String,
    pub domain_id: String,
    pub parent_id: Option<String>,
    pub label: String,
    pub order: i32,
}

#[apply(plain_eq)]
pub struct DesktopPageRegistration {
    pub plugin_name: String,
    pub id: String,
    pub domain_id: String,
    pub parent_branch_id: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub route: String,
    pub order: i32,
    pub role: DesktopPageRole,
}

#[apply(plain_eq)]
pub struct DesktopToolbarActionRegistration {
    pub plugin_name: String,
    pub route: Option<String>,
    pub action_id: String,
    pub label: String,
    pub tooltip: String,
    pub order: i32,
    pub primary: bool,
}

#[apply(plain_eq)]
pub struct DesktopSummaryCardRegistration {
    pub plugin_name: String,
    pub card_id: String,
    pub title: String,
    pub summary: String,
    pub route: String,
    pub order: i32,
}

#[apply(plain_eq)]
pub struct DesktopCommandRegistration {
    pub plugin_name: String,
    pub command_id: String,
    pub title: String,
}

/// Query model for host shell navigation and plugin render ownership.
///
/// The registry is built from plugin setup contributions so the desktop shell
/// can render domain and context trees without hardcoding app-specific routes.
#[apply(plain_eq)]
pub struct DesktopHostRegistry {
    domains: Vec<DesktopDomainRegistration>,
    branches: Vec<DesktopBranchRegistration>,
    pages: Vec<DesktopPageRegistration>,
    toolbar_actions: Vec<DesktopToolbarActionRegistration>,
    summary_cards: Vec<DesktopSummaryCardRegistration>,
}

impl_from_match!(DesktopContributions => DesktopHostRegistry {
    value => DesktopHostRegistry {
        domains: dedupe_and_sort_domains(value.domains),
        branches: dedupe_and_sort_branches(value.branches),
        pages: dedupe_and_sort_pages(value.pages),
        toolbar_actions: sort_toolbar_actions(value.toolbar_actions),
        summary_cards: sort_summary_cards(value.summary_cards),
    }
});

impl DesktopHostRegistry {
    #[must_use]
    pub fn domains(&self) -> &[DesktopDomainRegistration] {
        &self.domains
    }

    #[must_use]
    pub fn summary_cards(&self) -> &[DesktopSummaryCardRegistration] {
        &self.summary_cards
    }

    #[must_use]
    pub fn page_for_route(&self, route: &str) -> Option<&DesktopPageRegistration> {
        self.pages.iter().find(|page| page.route == route)
    }

    #[must_use]
    pub fn domain_for_route(&self, route: &str) -> Option<&DesktopDomainRegistration> {
        let page = self.page_for_route(route)?;
        self.domains
            .iter()
            .find(|domain| domain.id == page.domain_id)
    }

    #[must_use]
    pub fn toolbar_actions_for_route(&self, route: &str) -> Vec<&DesktopToolbarActionRegistration> {
        self.toolbar_actions
            .iter()
            .filter(|action| action.route.as_deref().is_none_or(|item| item == route))
            .collect()
    }

    #[must_use]
    pub fn root_branches_for_domain(&self, domain_id: &str) -> Vec<&DesktopBranchRegistration> {
        self.branches
            .iter()
            .filter(|branch| branch.domain_id == domain_id && branch.parent_id.is_none())
            .collect()
    }

    #[must_use]
    pub fn child_branches(&self, branch_id: &str) -> Vec<&DesktopBranchRegistration> {
        self.branches
            .iter()
            .filter(|branch| branch.parent_id.as_deref() == Some(branch_id))
            .collect()
    }

    #[must_use]
    pub fn pages_for_branch(&self, branch_id: &str) -> Vec<&DesktopPageRegistration> {
        self.pages
            .iter()
            .filter(|page| page.parent_branch_id.as_deref() == Some(branch_id))
            .collect()
    }

    #[must_use]
    pub fn root_pages_for_domain(&self, domain_id: &str) -> Vec<&DesktopPageRegistration> {
        self.pages
            .iter()
            .filter(|page| page.domain_id == domain_id && page.parent_branch_id.is_none())
            .collect()
    }

    #[must_use]
    pub fn plugins_for_route(&self, route: &str, role: DesktopPageRole) -> Vec<String> {
        self.pages
            .iter()
            .filter(|page| page.route == route && page.role == role)
            .map(|page| page.plugin_name.clone())
            .collect()
    }

    #[must_use]
    pub fn plugins_for_render_layer(
        &self,
        route: &str,
        layer: DesktopRenderLayer,
        plugins: &[Box<DesktopPlugin>],
        plugin_indices: &BTreeMap<String, usize>,
    ) -> Vec<usize> {
        let mut indices = self
            .pages
            .iter()
            .filter(|page| page.route == route)
            .filter_map(|page| plugin_indices.get(&page.plugin_name).copied())
            .filter(|index| plugins[*index].render_layer() == layer)
            .collect::<Vec<_>>();

        if layer == DesktopRenderLayer::Overlay {
            for index in plugin_indices.values() {
                if plugins[*index].render_layer() == DesktopRenderLayer::Overlay
                    && !indices.contains(index)
                {
                    indices.push(*index);
                }
            }
        }

        indices.sort_by_key(|index| Reverse(plugins[*index].priority()));
        indices
    }
}

fn dedupe_and_sort_domains(
    domains: Vec<DesktopDomainRegistration>,
) -> Vec<DesktopDomainRegistration> {
    let mut by_id = BTreeMap::new();
    for domain in domains {
        by_id.entry(domain.id.clone()).or_insert(domain);
    }
    let mut values = by_id.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(&right.label))
            .then(left.id.cmp(&right.id))
    });
    values
}

fn dedupe_and_sort_branches(
    branches: Vec<DesktopBranchRegistration>,
) -> Vec<DesktopBranchRegistration> {
    let mut by_id = BTreeMap::new();
    for branch in branches {
        by_id.entry(branch.id.clone()).or_insert(branch);
    }
    let mut values = by_id.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.domain_id
            .cmp(&right.domain_id)
            .then(left.parent_id.cmp(&right.parent_id))
            .then(left.order.cmp(&right.order))
            .then(left.label.cmp(&right.label))
            .then(left.id.cmp(&right.id))
    });
    values
}

fn dedupe_and_sort_pages(pages: Vec<DesktopPageRegistration>) -> Vec<DesktopPageRegistration> {
    let mut by_id = BTreeMap::new();
    for page in pages {
        by_id.entry(page.id.clone()).or_insert(page);
    }
    let mut values = by_id.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.domain_id
            .cmp(&right.domain_id)
            .then(left.parent_branch_id.cmp(&right.parent_branch_id))
            .then(left.order.cmp(&right.order))
            .then(left.title.cmp(&right.title))
            .then(left.id.cmp(&right.id))
    });
    values
}

fn sort_toolbar_actions(
    mut actions: Vec<DesktopToolbarActionRegistration>,
) -> Vec<DesktopToolbarActionRegistration> {
    actions.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then(left.order.cmp(&right.order))
            .then(left.label.cmp(&right.label))
            .then(left.action_id.cmp(&right.action_id))
    });
    actions
}

fn sort_summary_cards(
    mut cards: Vec<DesktopSummaryCardRegistration>,
) -> Vec<DesktopSummaryCardRegistration> {
    cards.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.title.cmp(&right.title))
            .then(left.card_id.cmp(&right.card_id))
    });
    cards
}

#[apply(plain_default_eq)]
pub struct DesktopShellSnapshot {
    pub current_route: String,
    pub current_domain_id: Option<String>,
    pub current_page_id: Option<String>,
    pub selected_entity: Option<String>,
    pub notice: Option<String>,
}

#[apply(plain_eq)]
pub enum DesktopEvent {
    Startup,
    RouteChanged {
        route: String,
    },
    ActionInvoked {
        route: String,
        action_id: String,
    },
    SelectionChanged {
        route: String,
        entity_id: Option<String>,
    },
    RefreshRequested {
        route: Option<String>,
    },
    Tick,
    PluginAction {
        action_id: String,
        payload: Value,
    },
}

#[apply(plain_default_eq)]
pub struct DesktopExecFeedback {
    pub notice: Option<String>,
    pub selected_entity: Option<Option<String>>,
    pub refresh_requested: bool,
    pub route_override: Option<String>,
}

#[apply(plain_clone)]
pub struct DesktopExecContext {
    pub services: Arc<dyn DesktopHostServices>,
    pub shell: DesktopShellSnapshot,
    feedback: Rc<RefCell<DesktopExecFeedback>>,
}

impl DesktopExecContext {
    #[must_use]
    pub fn new(
        services: Arc<dyn DesktopHostServices>,
        shell: DesktopShellSnapshot,
    ) -> (Self, Rc<RefCell<DesktopExecFeedback>>) {
        let feedback = Rc::new(RefCell::new(DesktopExecFeedback::default()));
        (
            Self {
                services,
                shell,
                feedback: feedback.clone(),
            },
            feedback,
        )
    }

    pub fn notify(&self, message: impl Into<String>) {
        self.feedback.borrow_mut().notice = Some(message.into());
    }

    pub fn request_refresh(&self) {
        self.feedback.borrow_mut().refresh_requested = true;
    }

    pub fn set_selected_entity(&self, entity: Option<String>) {
        self.feedback.borrow_mut().selected_entity = Some(entity);
    }

    pub fn navigate_to(&self, route: impl Into<String>) {
        self.feedback.borrow_mut().route_override = Some(route.into());
    }
}

#[apply(plain_default_eq)]
pub struct DesktopViewContext {
    pub shell: DesktopShellSnapshot,
}

#[apply(plain_default_eq)]
pub struct DesktopDriveSnapshot {
    pub roots: Vec<LocalRootState>,
    pub hosted: Vec<HostedStatus>,
    pub tracked: Vec<TrackedItem>,
    pub conflicts: Vec<DriveConflict>,
    pub queue: Vec<DriveSyncQueueItem>,
}

#[apply(plain_default_eq)]
pub struct DesktopProviderTestResult {
    pub provider: String,
    pub ok: bool,
    pub message: String,
}

pub trait DesktopHostServices: Send + Sync {
    fn load_drive_snapshot(&self) -> Result<DesktopDriveSnapshot, String>;

    fn drive_host_path(&self, path: &str) -> Result<String, String>;

    fn drive_unhost_path(&self, path: &str) -> Result<String, String>;

    fn drive_sync_once(&self) -> Result<String, String>;

    fn drive_retry_queue(&self) -> Result<String, String>;

    fn drive_pull_remote(&self, path: Option<&str>) -> Result<Vec<PullRemoteItem>, String>;

    fn list_tracked(
        &self,
        path: Option<&str>,
        options: ListTrackedOptions,
    ) -> Result<Vec<TrackedItem>, String>;

    fn drive_conflicts(&self) -> Result<Vec<DriveConflict>, String>;

    fn drive_sync_queue(
        &self,
        status: Option<DriveSyncTaskStatus>,
    ) -> Result<Vec<DriveSyncQueueItem>, String>;

    fn list_assets(&self, kind: Option<AssetKind>) -> Result<Vec<Asset>, String>;

    fn asset_graph(&self) -> Result<AssetGraph, String>;

    fn upsert_asset(&self, input: AssetUpsert) -> Result<Asset, String>;

    fn delete_asset(&self, id: Uuid) -> Result<(), String>;

    fn list_provider_configs(&self) -> Result<Vec<AiModelProvider>, String>;

    fn upsert_provider(&self, input: AiModelProviderUpsert) -> Result<AiModelProvider, String>;

    fn test_provider(&self, provider: AiProviderKind) -> Result<DesktopProviderTestResult, String>;

    fn software_catalog(&self) -> Result<SoftwareCatalogDto, String>;

    fn software_save_entry(&self, input: SoftwareEntryInput) -> Result<SoftwareEntryDto, String>;

    fn software_fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> Result<SoftwareMetadataDto, String>;

    fn open_path(&self, path: &str) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use az_derive_aliases::{apply, plain_default};
    use std::sync::Arc;

    use super::{
        DesktopBranchRegistration, DesktopContributions, DesktopDomainRegistration, DesktopEvent,
        DesktopExecContext, DesktopHostRegistry, DesktopHostServices, DesktopInitContext,
        DesktopPageRegistration, DesktopPageRole, DesktopProviderTestResult, DesktopRenderLayer,
        DesktopShellSnapshot, DesktopSummaryCardRegistration, DesktopToolbarActionRegistration,
        EventPropagation, ListTrackedOptions,
    };
    use uuid::Uuid;

    #[apply(plain_default)]
    struct FakeServices;

    impl DesktopHostServices for FakeServices {
        fn load_drive_snapshot(&self) -> Result<super::DesktopDriveSnapshot, String> {
            Ok(super::DesktopDriveSnapshot::default())
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
        ) -> Result<Vec<az_drive_agent::PullRemoteItem>, String> {
            Ok(Vec::new())
        }

        fn list_tracked(
            &self,
            _path: Option<&str>,
            _options: ListTrackedOptions,
        ) -> Result<Vec<az_drive_agent::TrackedItem>, String> {
            Ok(Vec::new())
        }

        fn drive_conflicts(&self) -> Result<Vec<az_drive_store::DriveConflict>, String> {
            Ok(Vec::new())
        }

        fn drive_sync_queue(
            &self,
            _status: Option<az_drive_store::DriveSyncTaskStatus>,
        ) -> Result<Vec<az_drive_store::DriveSyncQueueItem>, String> {
            Ok(Vec::new())
        }

        fn list_assets(
            &self,
            _kind: Option<az_assets::AssetKind>,
        ) -> Result<Vec<az_assets::Asset>, String> {
            Ok(Vec::new())
        }

        fn asset_graph(&self) -> Result<az_assets::AssetGraph, String> {
            Ok(az_assets::AssetGraph::default())
        }

        fn upsert_asset(&self, _input: az_assets::AssetUpsert) -> Result<az_assets::Asset, String> {
            Err("not implemented".to_string())
        }

        fn delete_asset(&self, _id: Uuid) -> Result<(), String> {
            Ok(())
        }

        fn list_provider_configs(&self) -> Result<Vec<az_assets::AiModelProvider>, String> {
            Ok(Vec::new())
        }

        fn upsert_provider(
            &self,
            _input: az_assets::AiModelProviderUpsert,
        ) -> Result<az_assets::AiModelProvider, String> {
            Err("not implemented".to_string())
        }

        fn test_provider(
            &self,
            _provider: az_assets::AiProviderKind,
        ) -> Result<DesktopProviderTestResult, String> {
            Ok(DesktopProviderTestResult::default())
        }

        fn software_catalog(&self) -> Result<az_software_catalog::SoftwareCatalogDto, String> {
            Err("unavailable".to_string())
        }

        fn software_save_entry(
            &self,
            _input: az_software_catalog::SoftwareEntryInput,
        ) -> Result<az_software_catalog::SoftwareEntryDto, String> {
            Err("unavailable".to_string())
        }

        fn software_fetch_metadata(
            &self,
            _input: az_software_catalog::SoftwareMetadataFetchInput,
        ) -> Result<az_software_catalog::SoftwareMetadataDto, String> {
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
        ctx.register_domain("ops", "Operations", 10, "/drive");
        ctx.register_branch("drive-branch", "ops", None::<String>, "Drive", 10);
        ctx.register_page(
            "drive-home",
            "ops",
            Some("drive-branch"),
            "Drive Center",
            "Sync roots",
            "/drive",
            10,
        );
        ctx.register_toolbar_action(
            Some("/drive"),
            "drive.refresh",
            "Refresh",
            "Reload snapshot",
            10,
            true,
        );
        ctx.register_summary_card("drive-card", "Drive Center", "Sync roots", "/drive", 10);
        ctx.register_command("drive.refresh", "Refresh");

        let contributions = ctx.into_contributions();
        assert_eq!(contributions.domains.len(), 1);
        assert_eq!(contributions.pages[0].plugin_name, "demo");
        assert_eq!(contributions.toolbar_actions[0].action_id, "drive.refresh");
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

        assert_eq!(registry.domains().len(), 1);
        assert_eq!(registry.root_branches_for_domain("ops").len(), 1);
        assert_eq!(registry.pages_for_branch("storage").len(), 1);
        assert_eq!(registry.toolbar_actions_for_route("/drive").len(), 1);
        assert_eq!(registry.summary_cards().len(), 1);
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
        assert_eq!(feedback.notice.as_deref(), Some("ok"));
        assert!(feedback.refresh_requested);
        assert_eq!(feedback.selected_entity, Some(Some("item-1".to_string())));
        assert_eq!(feedback.route_override.as_deref(), Some("/config"));
    }

    #[test]
    fn desktop_protocol_enums_expose_stable_codes() {
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
}
