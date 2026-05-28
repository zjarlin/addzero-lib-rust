#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::{cell::RefCell, cmp::Reverse, collections::BTreeMap, rc::Rc, sync::Arc};

use az_assets::{
    AiModelProvider, AiModelProviderUpsert, AiProviderKind, Asset, AssetGraph, AssetKind,
    AssetUpsert,
};
use az_derive_aliases::{
    apply, impl_from_match, plain_clone, plain_code_enum, plain_copy_eq, plain_default_eq, plain_eq,
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

/// 桌面插件生命周期和渲染契约。
///
/// 宿主通过泛型上下文把初始化、事件执行、视图渲染和渲染层解耦，插件只声明自己关心的贡献和行为。
pub trait Plugin<InitContext, Event, ExecContext, ViewContext, RenderLayer> {
    /// 插件稳定名称。
    fn name(&self) -> &'static str;

    /// 插件初始化入口，用于向宿主注册 domain、branch、page 等贡献。
    fn setup(&mut self, _ctx: &mut InitContext) {}

    /// 处理宿主事件。
    fn on_event(&mut self, _event: &Event, _ctx: &mut ExecContext) -> EventPropagation {
        EventPropagation::Continue
    }

    /// 渲染插件视图元素。
    fn render(&mut self, _ctx: &mut ViewContext) -> Option<AnyElement> {
        None
    }

    /// 同一渲染层内的优先级，值越大越靠前。
    fn priority(&self) -> i32 {
        0
    }

    /// 插件渲染层。
    fn render_layer(&self) -> RenderLayer;
}

/// AddZero 桌面宿主使用的插件对象类型。
pub type DesktopPlugin = dyn Plugin<
        DesktopInitContext,
        DesktopEvent,
        DesktopExecContext,
        DesktopViewContext,
        DesktopRenderLayer,
    >;

/// 插件事件处理后的传播策略。
#[apply(plain_code_enum)]
pub enum EventPropagation {
    /// 允许后续插件继续处理事件。
    Continue,
    /// 阻止事件继续传播。
    Stop,
}

/// 桌面插件渲染层。
#[apply(plain_code_enum)]
pub enum DesktopRenderLayer {
    /// 主内容区域。
    Main,
    /// 右侧或详情检查器区域。
    Inspector,
    /// 覆盖层、弹窗或浮层区域。
    Overlay,
}

/// 插件对页面路由的所有权角色。
#[apply(plain_code_enum)]
pub enum DesktopPageRole {
    /// 路由的主拥有者。
    Owner,
    /// 对已有路由追加内容或动作的贡献者。
    Contributor,
}

/// 插件在 setup 阶段声明的全部 shell 贡献。
#[apply(plain_default_eq)]
pub struct DesktopContributions {
    /// 顶层业务域。
    pub domains: Vec<DesktopDomainRegistration>,
    /// 侧轴导航分支。
    pub branches: Vec<DesktopBranchRegistration>,
    /// 页面路由。
    pub pages: Vec<DesktopPageRegistration>,
    /// 工具栏动作。
    pub toolbar_actions: Vec<DesktopToolbarActionRegistration>,
    /// 首页或概览区摘要卡片。
    pub summary_cards: Vec<DesktopSummaryCardRegistration>,
    /// 命令面板动作。
    pub commands: Vec<DesktopCommandRegistration>,
}

/// 插件初始化上下文。
///
/// 宿主在调用每个插件 setup 前设置当前插件名，后续注册项会自动带上来源插件。
#[apply(plain_default_eq)]
pub struct DesktopInitContext {
    current_plugin: Option<String>,
    contributions: DesktopContributions,
}

impl DesktopInitContext {
    /// 创建空初始化上下文。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置当前正在执行 setup 的插件名。
    pub fn set_current_plugin(&mut self, plugin_name: impl Into<String>) {
        self.current_plugin = Some(plugin_name.into());
    }

    /// 注册顶层业务域。
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

    /// 注册侧轴导航分支。
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

    /// 注册由当前插件拥有的页面。
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

    /// 注册页面并显式指定当前插件对该路由的角色。
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

    /// 注册路由相关或全局工具栏动作。
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

    /// 为同一路由批量注册工具栏动作。
    ///
    /// 插件仍显式列出每个 action 的 ID、文案、顺序和主次关系；这里只收敛重复的
    /// `Some(route)` 样板，避免 setup 阶段被机械注册代码淹没。
    pub fn register_route_toolbar_actions(
        &mut self,
        route: impl Into<String>,
        actions: impl IntoIterator<Item = DesktopToolbarActionSpec>,
    ) {
        let route = route.into();
        for action in actions {
            self.register_toolbar_action(
                Some(route.as_str()),
                action.action_id,
                action.label,
                action.tooltip,
                action.order,
                action.primary,
            );
        }
    }

    /// 注册首页或概览区摘要卡片。
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

    /// 注册可由命令面板触发的命令。
    pub fn register_command(&mut self, command_id: impl Into<String>, title: impl Into<String>) {
        self.contributions
            .commands
            .push(DesktopCommandRegistration {
                plugin_name: self.plugin_name(),
                command_id: command_id.into(),
                title: title.into(),
            });
    }

    /// 只读访问当前已收集的贡献。
    #[must_use]
    pub fn contributions(&self) -> &DesktopContributions {
        &self.contributions
    }

    /// 消耗上下文并取出贡献集合。
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

/// 顶层业务域注册项。
#[apply(plain_eq)]
pub struct DesktopDomainRegistration {
    /// 贡献该业务域的插件名。
    pub plugin_name: String,
    /// 业务域稳定 ID。
    pub id: String,
    /// 展示名称。
    pub label: String,
    /// 展示顺序，值越小越靠前。
    pub order: i32,
    /// 进入该业务域时的默认路由。
    pub default_route: String,
}

/// 侧轴导航分支注册项。
#[apply(plain_eq)]
pub struct DesktopBranchRegistration {
    /// 贡献该分支的插件名。
    pub plugin_name: String,
    /// 分支稳定 ID。
    pub id: String,
    /// 所属顶层业务域 ID。
    pub domain_id: String,
    /// 父分支 ID；为空表示业务域根分支。
    pub parent_id: Option<String>,
    /// 展示名称。
    pub label: String,
    /// 展示顺序，值越小越靠前。
    pub order: i32,
}

/// 页面路由注册项。
#[apply(plain_eq)]
pub struct DesktopPageRegistration {
    /// 贡献该页面的插件名。
    pub plugin_name: String,
    /// 页面稳定 ID。
    pub id: String,
    /// 所属顶层业务域 ID。
    pub domain_id: String,
    /// 所属侧轴分支 ID；为空表示业务域根页面。
    pub parent_branch_id: Option<String>,
    /// 页面标题。
    pub title: String,
    /// 页面副标题或简短说明。
    pub subtitle: String,
    /// 宿主 shell 使用的路由。
    pub route: String,
    /// 展示顺序，值越小越靠前。
    pub order: i32,
    /// 当前插件对该页面路由的所有权角色。
    pub role: DesktopPageRole,
}

/// 工具栏动作注册项。
#[apply(plain_eq)]
pub struct DesktopToolbarActionRegistration {
    /// 贡献该动作的插件名。
    pub plugin_name: String,
    /// 绑定路由；为空表示全局动作。
    pub route: Option<String>,
    /// 动作稳定 ID。
    pub action_id: String,
    /// 按钮或菜单展示文本。
    pub label: String,
    /// 悬停提示或辅助说明。
    pub tooltip: String,
    /// 展示顺序，值越小越靠前。
    pub order: i32,
    /// 是否作为主动作突出展示。
    pub primary: bool,
}

/// 插件 setup 中声明工具栏动作的轻量规格。
///
/// 该类型只描述 action 自身，不携带 route 和 plugin_name；route 由批量注册方法提供，
/// plugin_name 仍由 [`DesktopInitContext`] 根据当前 setup 插件自动注入。
#[apply(plain_copy_eq)]
pub struct DesktopToolbarActionSpec {
    /// 动作稳定 ID。
    pub action_id: &'static str,
    /// 按钮或菜单展示文本。
    pub label: &'static str,
    /// 悬停提示或辅助说明。
    pub tooltip: &'static str,
    /// 展示顺序，值越小越靠前。
    pub order: i32,
    /// 是否作为主动作突出展示。
    pub primary: bool,
}

impl DesktopToolbarActionSpec {
    /// 创建工具栏动作规格。
    #[must_use]
    pub const fn new(
        action_id: &'static str,
        label: &'static str,
        tooltip: &'static str,
        order: i32,
        primary: bool,
    ) -> Self {
        Self {
            action_id,
            label,
            tooltip,
            order,
            primary,
        }
    }

    /// 创建主工具栏动作规格。
    #[must_use]
    pub const fn primary(
        action_id: &'static str,
        label: &'static str,
        tooltip: &'static str,
        order: i32,
    ) -> Self {
        Self::new(action_id, label, tooltip, order, true)
    }

    /// 创建次级工具栏动作规格。
    #[must_use]
    pub const fn secondary(
        action_id: &'static str,
        label: &'static str,
        tooltip: &'static str,
        order: i32,
    ) -> Self {
        Self::new(action_id, label, tooltip, order, false)
    }
}

/// 摘要卡片注册项。
#[apply(plain_eq)]
pub struct DesktopSummaryCardRegistration {
    /// 贡献该卡片的插件名。
    pub plugin_name: String,
    /// 卡片稳定 ID。
    pub card_id: String,
    /// 卡片标题。
    pub title: String,
    /// 卡片摘要内容。
    pub summary: String,
    /// 点击卡片时进入的路由。
    pub route: String,
    /// 展示顺序，值越小越靠前。
    pub order: i32,
}

/// 命令面板动作注册项。
#[apply(plain_eq)]
pub struct DesktopCommandRegistration {
    /// 贡献该命令的插件名。
    pub plugin_name: String,
    /// 命令稳定 ID。
    pub command_id: String,
    /// 命令展示标题。
    pub title: String,
}

/// 宿主 shell 导航和插件渲染所有权查询模型。
///
/// 该注册表由插件 setup 贡献构建，desktop shell 可据此渲染 domain 与 context tree，
/// 不需要在壳子层硬编码应用路由。
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
    /// 返回按展示顺序排序后的业务域。
    #[must_use]
    pub fn domains(&self) -> &[DesktopDomainRegistration] {
        &self.domains
    }

    /// 返回按展示顺序排序后的摘要卡片。
    #[must_use]
    pub fn summary_cards(&self) -> &[DesktopSummaryCardRegistration] {
        &self.summary_cards
    }

    /// 根据路由查找页面注册项。
    #[must_use]
    pub fn page_for_route(&self, route: &str) -> Option<&DesktopPageRegistration> {
        self.pages.iter().find(|page| page.route == route)
    }

    /// 根据路由查找所属业务域。
    #[must_use]
    pub fn domain_for_route(&self, route: &str) -> Option<&DesktopDomainRegistration> {
        let page = self.page_for_route(route)?;
        self.domains
            .iter()
            .find(|domain| domain.id == page.domain_id)
    }

    /// 返回指定路由可见的工具栏动作。
    #[must_use]
    pub fn toolbar_actions_for_route(&self, route: &str) -> Vec<&DesktopToolbarActionRegistration> {
        self.toolbar_actions
            .iter()
            .filter(|action| action.route.as_deref().is_none_or(|item| item == route))
            .collect()
    }

    /// 返回某业务域下的根分支。
    #[must_use]
    pub fn root_branches_for_domain(&self, domain_id: &str) -> Vec<&DesktopBranchRegistration> {
        self.branches
            .iter()
            .filter(|branch| branch.domain_id == domain_id && branch.parent_id.is_none())
            .collect()
    }

    /// 返回某分支下的子分支。
    #[must_use]
    pub fn child_branches(&self, branch_id: &str) -> Vec<&DesktopBranchRegistration> {
        self.branches
            .iter()
            .filter(|branch| branch.parent_id.as_deref() == Some(branch_id))
            .collect()
    }

    /// 返回某分支下的页面。
    #[must_use]
    pub fn pages_for_branch(&self, branch_id: &str) -> Vec<&DesktopPageRegistration> {
        self.pages
            .iter()
            .filter(|page| page.parent_branch_id.as_deref() == Some(branch_id))
            .collect()
    }

    /// 返回某业务域下无父分支的页面。
    #[must_use]
    pub fn root_pages_for_domain(&self, domain_id: &str) -> Vec<&DesktopPageRegistration> {
        self.pages
            .iter()
            .filter(|page| page.domain_id == domain_id && page.parent_branch_id.is_none())
            .collect()
    }

    /// 返回某路由上具备指定页面角色的插件名。
    #[must_use]
    pub fn plugins_for_route(&self, route: &str, role: DesktopPageRole) -> Vec<String> {
        self.pages
            .iter()
            .filter(|page| page.route == route && page.role == role)
            .map(|page| page.plugin_name.clone())
            .collect()
    }

    /// 返回某路由和渲染层上应参与渲染的插件索引。
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

/// 宿主 shell 在事件执行或视图渲染时暴露的当前状态快照。
#[apply(plain_default_eq)]
pub struct DesktopShellSnapshot {
    /// 当前路由。
    pub current_route: String,
    /// 当前顶层业务域 ID。
    pub current_domain_id: Option<String>,
    /// 当前页面 ID。
    pub current_page_id: Option<String>,
    /// 当前选中的实体 ID。
    pub selected_entity: Option<String>,
    /// 当前 shell 级提示信息。
    pub notice: Option<String>,
}

/// 桌面宿主分发给插件的事件协议。
#[apply(plain_eq)]
pub enum DesktopEvent {
    /// 宿主启动事件。
    Startup,
    /// 当前路由发生变化。
    RouteChanged {
        /// 切换后的路由。
        route: String,
    },
    /// 用户触发了工具栏或命令动作。
    ActionInvoked {
        /// 动作触发时所在路由。
        route: String,
        /// 被触发的动作 ID。
        action_id: String,
    },
    /// 当前选中实体发生变化。
    SelectionChanged {
        /// 选择发生时所在路由。
        route: String,
        /// 新的实体 ID；为空表示清空选择。
        entity_id: Option<String>,
    },
    /// 请求刷新当前或指定路由的数据。
    RefreshRequested {
        /// 需要刷新的路由；为空表示由宿主决定刷新范围。
        route: Option<String>,
    },
    /// 宿主周期性心跳事件。
    Tick,
    /// 插件自定义动作事件。
    PluginAction {
        /// 动作稳定 ID。
        action_id: String,
        /// 动作携带的 JSON 负载。
        payload: Value,
    },
}

/// 插件事件处理后返还给宿主的副作用反馈。
#[apply(plain_default_eq)]
pub struct DesktopExecFeedback {
    /// 需要展示给用户的提示信息。
    pub notice: Option<String>,
    /// 选择状态更新；外层为空表示不修改选择，内层为空表示清空选择。
    pub selected_entity: Option<Option<String>>,
    /// 是否请求宿主刷新数据。
    pub refresh_requested: bool,
    /// 是否请求宿主跳转到指定路由。
    pub route_override: Option<String>,
}

/// 插件处理事件时可使用的执行上下文。
#[apply(plain_clone)]
pub struct DesktopExecContext {
    /// 宿主提供的领域服务集合。
    pub services: Arc<dyn DesktopHostServices>,
    /// 当前 shell 状态快照。
    pub shell: DesktopShellSnapshot,
    feedback: Rc<RefCell<DesktopExecFeedback>>,
}

impl DesktopExecContext {
    /// 创建事件执行上下文，并返回宿主可读取的反馈句柄。
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

    /// 请求宿主展示提示信息。
    pub fn notify(&self, message: impl Into<String>) {
        self.feedback.borrow_mut().notice = Some(message.into());
    }

    /// 请求宿主刷新数据。
    pub fn request_refresh(&self) {
        self.feedback.borrow_mut().refresh_requested = true;
    }

    /// 请求宿主更新当前选中实体。
    pub fn set_selected_entity(&self, entity: Option<String>) {
        self.feedback.borrow_mut().selected_entity = Some(entity);
    }

    /// 请求宿主跳转到指定路由。
    pub fn navigate_to(&self, route: impl Into<String>) {
        self.feedback.borrow_mut().route_override = Some(route.into());
    }
}

/// 插件渲染视图时可读取的上下文。
#[apply(plain_default_eq)]
pub struct DesktopViewContext {
    /// 当前 shell 状态快照。
    pub shell: DesktopShellSnapshot,
}

/// 网盘插件页面一次性读取的同步状态快照。
#[apply(plain_default_eq)]
pub struct DesktopDriveSnapshot {
    /// 本地根目录状态。
    pub roots: Vec<LocalRootState>,
    /// 已托管路径状态。
    pub hosted: Vec<HostedStatus>,
    /// 已追踪文件或目录。
    pub tracked: Vec<TrackedItem>,
    /// 当前同步冲突。
    pub conflicts: Vec<DriveConflict>,
    /// 同步队列条目。
    pub queue: Vec<DriveSyncQueueItem>,
}

/// AI 服务商连通性测试结果。
#[apply(plain_default_eq)]
pub struct DesktopProviderTestResult {
    /// 被测试的 provider 名称。
    pub provider: String,
    /// 测试是否通过。
    pub ok: bool,
    /// 面向用户或日志的测试结果说明。
    pub message: String,
}

/// 桌面插件访问宿主后端能力的服务边界。
///
/// 插件只依赖该 trait，不直接绑定网盘、资产、AI 服务商或软件目录的具体实现。
pub trait DesktopHostServices: Send + Sync {
    /// 加载网盘页面所需的聚合快照。
    fn load_drive_snapshot(&self) -> Result<DesktopDriveSnapshot, String>;

    /// 将本地路径纳入网盘托管。
    fn drive_host_path(&self, path: &str) -> Result<String, String>;

    /// 取消本地路径的网盘托管。
    fn drive_unhost_path(&self, path: &str) -> Result<String, String>;

    /// 触发一次网盘同步。
    fn drive_sync_once(&self) -> Result<String, String>;

    /// 重试网盘同步队列中的失败任务。
    fn drive_retry_queue(&self) -> Result<String, String>;

    /// 从远端拉取指定路径或全部路径。
    fn drive_pull_remote(&self, path: Option<&str>) -> Result<Vec<PullRemoteItem>, String>;

    /// 列出网盘追踪项。
    fn list_tracked(
        &self,
        path: Option<&str>,
        options: ListTrackedOptions,
    ) -> Result<Vec<TrackedItem>, String>;

    /// 列出当前网盘冲突。
    fn drive_conflicts(&self) -> Result<Vec<DriveConflict>, String>;

    /// 按状态过滤网盘同步队列。
    fn drive_sync_queue(
        &self,
        status: Option<DriveSyncTaskStatus>,
    ) -> Result<Vec<DriveSyncQueueItem>, String>;

    /// 按资产类型列出资产。
    fn list_assets(&self, kind: Option<AssetKind>) -> Result<Vec<Asset>, String>;

    /// 读取资产关系图。
    fn asset_graph(&self) -> Result<AssetGraph, String>;

    /// 新增或更新资产。
    fn upsert_asset(&self, input: AssetUpsert) -> Result<Asset, String>;

    /// 删除资产。
    fn delete_asset(&self, id: Uuid) -> Result<(), String>;

    /// 列出 AI 服务商配置。
    fn list_provider_configs(&self) -> Result<Vec<AiModelProvider>, String>;

    /// 新增或更新 AI 服务商配置。
    fn upsert_provider(&self, input: AiModelProviderUpsert) -> Result<AiModelProvider, String>;

    /// 测试指定 AI 服务商是否可用。
    fn test_provider(&self, provider: AiProviderKind) -> Result<DesktopProviderTestResult, String>;

    /// 读取软件目录聚合数据。
    fn software_catalog(&self) -> Result<SoftwareCatalogDto, String>;

    /// 保存软件目录条目。
    fn software_save_entry(&self, input: SoftwareEntryInput) -> Result<SoftwareEntryDto, String>;

    /// 拉取软件条目的外部元数据。
    fn software_fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> Result<SoftwareMetadataDto, String>;

    /// 请求宿主用系统默认方式打开路径。
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
        DesktopToolbarActionSpec, EventPropagation, ListTrackedOptions,
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
        ctx.register_route_toolbar_actions(
            "/drive",
            [
                DesktopToolbarActionSpec::primary(
                    "drive.refresh",
                    "Refresh",
                    "Reload snapshot",
                    10,
                ),
                DesktopToolbarActionSpec::secondary("drive.sync", "Sync", "Run sync", 20),
            ],
        );
        ctx.register_summary_card("drive-card", "Drive Center", "Sync roots", "/drive", 10);
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
}
