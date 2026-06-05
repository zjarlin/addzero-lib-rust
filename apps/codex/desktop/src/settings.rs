#![forbid(unsafe_code)]

use std::process::Command;

use crate::sidebar::{SidebarItemModel, SidebarSectionModel, SidebarSectionView};
use az_git::{
    AuthDiscovery, AuthDiscoveryOptions, AuthLoginFlow, AuthMethod, AuthSession, AuthState,
    DEFAULT_SYNC_WORKSPACE, GitAccountConfig, GitAccountConfigStore, GitHostingAccountStatus,
    GitHostingProvider, GitProjectBinding, GitRepositoryDiscovery,
};
use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn SettingsPage(on_return: EventHandler<()>) -> Element {
    let mut settings = use_signal(SettingsState::load);
    let mut active_route = use_signal(|| SettingsRoute::ProjectDefaults);

    let state = settings.read().clone();
    let route = *active_route.read();
    let route_id = route.id().to_string();

    rsx! {
        main { class: "settings-fullscreen",
            header { class: "settings-topbar",
                button {
                    class: "settings-return-button",
                    r#type: "button",
                    onclick: move |_| on_return.call(()),
                    span { "‹" }
                    "返回应用"
                }
                div { class: "settings-topbar__title",
                    span { "Codex" }
                    strong { "设置" }
                }
            }

            div { class: "settings-workbench",
                aside { class: "settings-tree",
                    div { class: "settings-tree__brand",
                        span { class: "settings-tree__brand-mark", "⚙" }
                        div {
                            strong { "设置中心" }
                            p { "插件化桌面端配置" }
                        }
                    }
                    for section in settings_tree_sections() {
                        SidebarSectionView {
                            section,
                            active_id: route_id.clone(),
                            on_select: move |selected_id: String| {
                                if let Some(next_route) = SettingsRoute::from_id(&selected_id) {
                                    active_route.set(next_route);
                                }
                            },
                        }
                    }
                }

                section { class: "settings-content",
                    SettingsContentHeader { route }
                    if route == SettingsRoute::GitAccounts {
                        GitAccountSettingsPanel {
                            statuses: state.statuses.clone(),
                            config_path: state.config_path.clone(),
                            message: state.message.clone(),
                            message_class: state.message_class().to_string(),
                            on_refresh: move |_| settings.write().refresh(),
                            on_save: move |_| settings.write().save(),
                            on_username_change: move |(provider, username)| {
                                settings.write().set_username(provider, username);
                            },
                            on_open_url: move |url| {
                                settings.write().open_url(url);
                            },
                        }
                    } else if route == SettingsRoute::ProjectDefaults {
                        ProjectDefaultsSettingsPanel {
                            sync_workspace: state.config.sync_workspace().to_string(),
                            project_bindings: state.config.project_bindings.clone(),
                            config_path: state.config_path.clone(),
                            message: state.message.clone(),
                            message_class: state.message_class().to_string(),
                            on_workspace_change: move |workspace| {
                                settings.write().set_sync_workspace(workspace);
                            },
                            on_save: move |_| settings.write().save(),
                            on_refresh_projects: move |_| settings.write().refresh_projects(),
                        }
                    } else {
                        SettingsPlaceholderPanel { route }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn SettingsContentHeader(route: SettingsRoute) -> Element {
    rsx! {
        section { class: "settings-header",
            div {
                p { class: "settings-header__eyebrow", "设置 / {route.group_label()}" }
                h1 { "{route.title()}" }
                p { "{route.subtitle()}" }
            }
            div { class: "settings-header__mark", "{route.mark()}" }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn GitAccountSettingsPanel(
    statuses: Vec<GitHostingAccountStatus>,
    config_path: String,
    message: Option<String>,
    message_class: String,
    on_refresh: EventHandler<()>,
    on_save: EventHandler<()>,
    on_username_change: EventHandler<(GitHostingProvider, String)>,
    on_open_url: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "settings-panel",
            div { class: "settings-panel__actions",
                button {
                    class: "toolbar-button",
                    r#type: "button",
                    onclick: move |_| on_refresh.call(()),
                    "刷新登录态"
                }
                button {
                    class: "toolbar-button toolbar-button--primary",
                    r#type: "button",
                    onclick: move |_| on_save.call(()),
                    "保存用户名"
                }
            }

            if let Some(message) = message.as_ref() {
                div { class: "{message_class}", "{message}" }
            }

            section { class: "settings-grid", aria_label: "Git 账号服务商",
                for status in statuses {
                    GitProviderCard {
                        status,
                        on_username_change,
                        on_open_url,
                    }
                }
            }

            section { class: "settings-note",
                h2 { "存储边界" }
                p { "当前只把用户名写入本地配置；令牌不写入 JSON 文件。后续接入系统凭据存储后，再开放令牌持久化。" }
                code { "{config_path}" }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ProjectDefaultsSettingsPanel(
    sync_workspace: String,
    project_bindings: Vec<GitProjectBinding>,
    config_path: String,
    message: Option<String>,
    message_class: String,
    on_workspace_change: EventHandler<String>,
    on_save: EventHandler<()>,
    on_refresh_projects: EventHandler<()>,
) -> Element {
    let project_count = project_bindings.len();

    rsx! {
        div { class: "settings-panel",
            div { class: "settings-panel__actions",
                button {
                    class: "toolbar-button",
                    r#type: "button",
                    onclick: move |_| on_save.call(()),
                    "保存同步空间"
                }
                button {
                    class: "toolbar-button toolbar-button--primary",
                    r#type: "button",
                    onclick: move |_| on_refresh_projects.call(()),
                    "获取并绑定仓库"
                }
            }

            if let Some(message) = message.as_ref() {
                div { class: "{message_class}", "{message}" }
            }

            label { class: "settings-form-row settings-form-row--wide",
                span { "同步空间" }
                input {
                    class: "settings-input",
                    value: "{sync_workspace}",
                    placeholder: DEFAULT_SYNC_WORKSPACE,
                    oninput: move |event| on_workspace_change.call(event.value()),
                }
            }

            div { class: "settings-summary-grid",
                div { class: "settings-summary-tile",
                    span { "本地根目录" }
                    strong { "{sync_workspace}" }
                }
                div { class: "settings-summary-tile",
                    span { "绑定仓库" }
                    strong { "{project_count}" }
                }
                div { class: "settings-summary-tile",
                    span { "仓库来源" }
                    strong { "gh 登录态" }
                }
            }

            section { class: "settings-project-list", aria_label: "已绑定 Git 仓库",
                if project_bindings.is_empty() {
                    div { class: "settings-note settings-note--wide",
                        h2 { "未绑定仓库" }
                        p { "点击获取并绑定仓库后，会读取当前 gh 登录态可见的 GitHub 仓库，并按同步空间自动生成本地目录路径。" }
                    }
                } else {
                    for project in project_bindings {
                        ProjectBindingRow { project }
                    }
                }
            }

            section { class: "settings-note settings-note--wide",
                h2 { "本地配置" }
                p { "左侧项目列表只读取本地绑定配置；刷新仓库只记录远端地址和本地目录路径，不会自动克隆或写入令牌。" }
                code { "{config_path}" }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ProjectBindingRow(project: GitProjectBinding) -> Element {
    let provider = project.provider.info().label;
    let name_with_owner = project.name_with_owner();

    rsx! {
        article { class: "settings-project-row",
            div {
                strong { "{project.name}" }
                p { "{provider} / {name_with_owner}" }
                code { "{project.remote_url}" }
                code { "{project.local_path}" }
            }
            span { class: "settings-row__badge settings-row__badge--ok", "已绑定" }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn SettingsPlaceholderPanel(route: SettingsRoute) -> Element {
    rsx! {
        div { class: "settings-panel settings-panel--placeholder",
            div { class: "settings-summary-grid",
                for item in route.preview_items() {
                    div { class: "settings-summary-tile",
                        span { "{item.label}" }
                        strong { "{item.value}" }
                    }
                }
            }
            section { class: "settings-note settings-note--wide",
                h2 { "{route.title()} 面板" }
                p { "{route.placeholder()}" }
            }
            div { class: "settings-row-list",
                for row in route.rows() {
                    div { class: "settings-row",
                        div {
                            strong { "{row.label}" }
                            p { "{row.description}" }
                        }
                        span { class: row.badge_class, "{row.badge}" }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn GitProviderCard(
    status: GitHostingAccountStatus,
    on_username_change: EventHandler<(GitHostingProvider, String)>,
    on_open_url: EventHandler<String>,
) -> Element {
    let provider = status.provider;
    let info = provider.info();
    let username = configured_or_session_username(&status);
    let badge_class = provider_badge_class(provider);
    let primary_state = primary_auth_state(&status);

    rsx! {
        article { class: "settings-provider-card",
            div { class: "settings-provider-card__header",
                div { class: badge_class, "{provider_mark(provider)}" }
                div {
                    h2 { "{info.label}" }
                    p { "{info.host}" }
                }
                span { class: auth_state_class(primary_state), "{auth_state_label(primary_state)}" }
            }

            label { class: "settings-form-row",
                span { "用户名" }
                input {
                    class: "settings-input",
                    value: "{username}",
                    placeholder: username_placeholder(provider),
                    oninput: move |event| on_username_change.call((provider, event.value())),
                }
            }

            div { class: "auth-session-list",
                if status.sessions.is_empty() {
                    div { class: "auth-session auth-session--muted",
                        span { "未检测命令行登录态" }
                        p { "可使用网页登录或令牌入口完成授权。" }
                    }
                } else {
                    for session in status.sessions {
                        AuthSessionRow { session }
                    }
                }
            }

            div { class: "settings-actions",
                for flow in status.login_flows {
                    LoginFlowButton {
                        flow,
                        on_open_url,
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn AuthSessionRow(session: AuthSession) -> Element {
    rsx! {
        div { class: auth_session_class(session.state),
            span { "{auth_method_label(session.method)}" }
            p { "{session.message}" }
            if let Some(username) = session.username.as_ref() {
                code { "{username}" }
            }
            if let Some(source) = session.source.as_ref() {
                em { "{source}" }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn LoginFlowButton(flow: AuthLoginFlow, on_open_url: EventHandler<String>) -> Element {
    let method = flow.method;
    let url = flow.url.clone();
    let command = flow.command.as_ref().map(|parts| parts.join(" "));

    rsx! {
        div { class: login_flow_class(method),
            div {
                h3 { "{flow.label}" }
                p { "{flow.description}" }
                if let Some(command) = command.as_ref() {
                    code { "{command}" }
                }
                if let Some(url) = url.as_ref() {
                    code { "{url}" }
                }
            }
            if let Some(url) = url {
                button {
                    class: "settings-link-button",
                    r#type: "button",
                    onclick: move |_| on_open_url.call(url.clone()),
                    "打开"
                }
            } else {
                span { class: "settings-flow-badge", "自动检测" }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct SettingsState {
    store: Option<GitAccountConfigStore>,
    config: GitAccountConfig,
    statuses: Vec<GitHostingAccountStatus>,
    config_path: String,
    message: Option<String>,
    message_kind: SettingsMessageKind,
}

impl SettingsState {
    fn load() -> Self {
        let store = GitAccountConfigStore::default_store();
        let mut message = None;
        let mut message_kind = SettingsMessageKind::Info;
        let (store, config, config_path) = match store {
            Ok(store) => {
                let path = store.path().display().to_string();
                let config = store.load().unwrap_or_else(|error| {
                    message = Some(format!("读取 Git 账号配置失败：{error}"));
                    message_kind = SettingsMessageKind::Error;
                    GitAccountConfig::default()
                });
                (Some(store), config, path)
            }
            Err(error) => {
                message = Some(format!("无法定位 Git 账号配置目录：{error}"));
                message_kind = SettingsMessageKind::Error;
                (None, GitAccountConfig::default(), "不可用".to_string())
            }
        };

        let statuses = configured_statuses(&config);
        Self {
            store,
            config,
            statuses,
            config_path,
            message,
            message_kind,
        }
    }

    fn refresh(&mut self) {
        self.statuses = discover_statuses(&self.config);
        self.message = Some("已刷新 Git 登录态。".to_string());
        self.message_kind = SettingsMessageKind::Info;
    }

    fn save(&mut self) {
        if self.persist_config("设置已保存。") {
            self.statuses = configured_statuses(&self.config);
        }
    }

    fn set_username(&mut self, provider: GitHostingProvider, username: String) {
        self.config.set_username(provider, username);
        self.statuses = configured_statuses(&self.config);
        self.message = None;
    }

    fn set_sync_workspace(&mut self, sync_workspace: String) {
        self.config.set_sync_workspace(sync_workspace);
        self.message = None;
    }

    fn refresh_projects(&mut self) {
        self.statuses = discover_statuses(&self.config);
        let Some(username) =
            sync_username_for_provider(&self.config, &self.statuses, GitHostingProvider::GitHub)
        else {
            self.message = Some("未检测到可用于同步仓库的 GitHub 登录态。".to_string());
            self.message_kind = SettingsMessageKind::Error;
            return;
        };

        match GitRepositoryDiscovery::system().discover_provider_repositories(
            GitHostingProvider::GitHub,
            &username,
            200,
        ) {
            Ok(repositories) => {
                let count = repositories.len();
                self.config.bind_remote_repositories(repositories);
                let message = format!(
                    "已根据 {username} 的 GitHub 登录态绑定 {count} 个仓库到 {}。",
                    self.config.sync_workspace()
                );
                self.persist_config(message);
            }
            Err(error) => {
                self.message = Some(format!("获取 GitHub 仓库失败：{error}"));
                self.message_kind = SettingsMessageKind::Error;
            }
        }
    }

    fn open_url(&mut self, url: String) {
        match open_external_url(&url) {
            Ok(()) => {
                self.message = Some(format!("已打开：{url}"));
                self.message_kind = SettingsMessageKind::Info;
            }
            Err(error) => {
                self.message = Some(format!("打开网页登录入口失败：{error}"));
                self.message_kind = SettingsMessageKind::Error;
            }
        }
    }

    fn persist_config(&mut self, success_message: impl Into<String>) -> bool {
        let Some(store) = self.store.as_ref() else {
            self.message = Some("配置目录不可用，无法保存。".to_string());
            self.message_kind = SettingsMessageKind::Error;
            return false;
        };

        match store.save(&self.config) {
            Ok(()) => {
                self.message = Some(success_message.into());
                self.message_kind = SettingsMessageKind::Success;
                true
            }
            Err(error) => {
                self.message = Some(format!("保存设置失败：{error}"));
                self.message_kind = SettingsMessageKind::Error;
                false
            }
        }
    }

    fn message_class(&self) -> &'static str {
        match self.message_kind {
            SettingsMessageKind::Info => "settings-message settings-message--info",
            SettingsMessageKind::Success => "settings-message settings-message--success",
            SettingsMessageKind::Error => "settings-message settings-message--error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingsMessageKind {
    Info,
    Success,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingsRoute {
    General,
    Appearance,
    Window,
    Plugins,
    Skills,
    Commands,
    Environment,
    Deployment,
    ProjectDefaults,
    GitAccounts,
    Network,
    Shortcuts,
    Privacy,
    Advanced,
}

impl SettingsRoute {
    const ALL: [Self; 14] = [
        Self::General,
        Self::Appearance,
        Self::Window,
        Self::Plugins,
        Self::Skills,
        Self::Commands,
        Self::Environment,
        Self::Deployment,
        Self::ProjectDefaults,
        Self::GitAccounts,
        Self::Network,
        Self::Shortcuts,
        Self::Privacy,
        Self::Advanced,
    ];

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|route| route.id() == id)
    }

    fn id(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Appearance => "appearance",
            Self::Window => "window",
            Self::Plugins => "plugins",
            Self::Skills => "skills",
            Self::Commands => "commands",
            Self::Environment => "environment",
            Self::Deployment => "deployment",
            Self::ProjectDefaults => "project-defaults",
            Self::GitAccounts => "git-accounts",
            Self::Network => "network",
            Self::Shortcuts => "shortcuts",
            Self::Privacy => "privacy",
            Self::Advanced => "advanced",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::General => "通用",
            Self::Appearance => "外观",
            Self::Window => "窗口",
            Self::Plugins => "插件",
            Self::Skills => "技能",
            Self::Commands => "命令行",
            Self::Environment => "环境变量",
            Self::Deployment => "部署路径",
            Self::ProjectDefaults => "项目默认值",
            Self::GitAccounts => "Git 账号",
            Self::Network => "网络",
            Self::Shortcuts => "快捷键",
            Self::Privacy => "隐私",
            Self::Advanced => "高级",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::GitAccounts => "配置项目常用 Git 托管账号，并优先复用本机已登录的命令行状态。",
            Self::Commands => "管理可视化命令行条目、来源识别和部署目标。",
            Self::Environment => "管理环境变量条目、逻辑删除状态和部署目标。",
            Self::Deployment => "维护识别路径与部署路径的对照关系。",
            Self::Skills => "按开发人员、设计等标签管理技能启停。",
            Self::Plugins => "查看插件贡献、启停状态和错误隔离信息。",
            Self::ProjectDefaults => "设置同步空间，并把登录态可见的 Git 仓库绑定到本地目录。",
            Self::Appearance => "调整字体渲染、密度、主题和窗口视觉。",
            Self::Window => "控制沉浸式标题栏、侧边栏和窗口行为。",
            Self::Network => "配置代理、请求超时和远端服务访问策略。",
            Self::Shortcuts => "管理常用操作的键盘入口。",
            Self::Privacy => "控制本地数据、日志和外部跳转边界。",
            Self::Advanced => "面向插件宿主和诊断的高级开关。",
            Self::General => "维护应用启动、语言和默认行为。",
        }
    }

    fn group_label(self) -> &'static str {
        match self {
            Self::General | Self::Appearance | Self::Window => "应用",
            Self::Plugins | Self::Skills => "插件与技能",
            Self::Commands | Self::Environment | Self::Deployment => "终端与环境",
            Self::ProjectDefaults | Self::GitAccounts => "项目与账号",
            Self::Network | Self::Shortcuts | Self::Privacy | Self::Advanced => "系统",
        }
    }

    fn mark(self) -> &'static str {
        match self {
            Self::General => "⌘",
            Self::Appearance => "◐",
            Self::Window => "▣",
            Self::Plugins => "▦",
            Self::Skills => "✦",
            Self::Commands => "›_",
            Self::Environment => "环",
            Self::Deployment => "↥",
            Self::ProjectDefaults => "▱",
            Self::GitAccounts => "账",
            Self::Network => "⌁",
            Self::Shortcuts => "⌨",
            Self::Privacy => "●",
            Self::Advanced => "⚙",
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            Self::GitAccounts => "",
            Self::Commands | Self::Environment | Self::Deployment => {
                "这里会承接命令行、环境变量和部署路径的可视化管理，不再散落到普通插件页。"
            }
            Self::Plugins | Self::Skills => {
                "这里会承接插件宿主与技能标签的全局配置，列表数据仍由插件系统提供。"
            }
            Self::ProjectDefaults => "",
            Self::Appearance | Self::Window => "这里会集中管理桌面壳的视觉、字体和窗口行为。",
            Self::Network | Self::Shortcuts | Self::Privacy | Self::Advanced | Self::General => {
                "这里先保留设置骨架，后续按真实配置源接入读写边界。"
            }
        }
    }

    fn preview_items(self) -> Vec<PreviewItem> {
        match self {
            Self::Commands => vec![
                PreviewItem::new("条目来源", "~/.add_fn"),
                PreviewItem::new("部署方式", "卡片 / 一键"),
                PreviewItem::new("删除策略", "逻辑删除"),
            ],
            Self::Environment => vec![
                PreviewItem::new("变量来源", "~/.add_fn"),
                PreviewItem::new("路径对照", "识别 / 部署"),
                PreviewItem::new("生效范围", "终端环境"),
            ],
            Self::Skills => vec![
                PreviewItem::new("系统技能", ".codex/skills/.system"),
                PreviewItem::new("用户技能", ".agents/skills"),
                PreviewItem::new("标签", "开发 / 设计"),
            ],
            Self::Plugins => vec![
                PreviewItem::new("宿主", "Dioxus"),
                PreviewItem::new("贡献", "描述符"),
                PreviewItem::new("外部插件", "组件协议"),
            ],
            Self::ProjectDefaults | Self::GitAccounts => Vec::new(),
            _ => vec![
                PreviewItem::new("状态", "骨架"),
                PreviewItem::new("来源", "本地"),
                PreviewItem::new("写入", "待接入"),
            ],
        }
    }

    fn rows(self) -> Vec<SettingsRow> {
        match self {
            Self::Commands => vec![
                SettingsRow::new(
                    "命令增删改查",
                    "可视化维护别名、函数和部署目标。",
                    "已规划",
                    "settings-row__badge",
                ),
                SettingsRow::new(
                    "逻辑删除",
                    "删除不立刻丢弃来源，可保留恢复边界。",
                    "启用",
                    "settings-row__badge settings-row__badge--ok",
                ),
                SettingsRow::new(
                    "一键部署",
                    "右上角执行所有命令卡片部署动作。",
                    "待接入",
                    "settings-row__badge",
                ),
            ],
            Self::Environment => vec![
                SettingsRow::new(
                    "变量增删改查",
                    "维护导出变量条目和来源对照。",
                    "已规划",
                    "settings-row__badge",
                ),
                SettingsRow::new(
                    "部署路径",
                    "支持单个默认路径和多个追加路径。",
                    "待接入",
                    "settings-row__badge",
                ),
                SettingsRow::new(
                    "只读源文件",
                    "~/.add_fn 由界面写入，不允许手工编辑。",
                    "受控",
                    "settings-row__badge settings-row__badge--ok",
                ),
            ],
            Self::ProjectDefaults | Self::GitAccounts => Vec::new(),
            _ => vec![
                SettingsRow::new(
                    "配置源",
                    "后续接入正式配置存储和读写边界。",
                    "骨架",
                    "settings-row__badge",
                ),
                SettingsRow::new(
                    "组件边界",
                    "设置树、内容头和面板独立封装。",
                    "已拆分",
                    "settings-row__badge settings-row__badge--ok",
                ),
                SettingsRow::new(
                    "插件贡献",
                    "保留给 app-local 插件系统继续扩展。",
                    "预留",
                    "settings-row__badge",
                ),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PreviewItem {
    label: &'static str,
    value: &'static str,
}

impl PreviewItem {
    const fn new(label: &'static str, value: &'static str) -> Self {
        Self { label, value }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SettingsRow {
    label: &'static str,
    description: &'static str,
    badge: &'static str,
    badge_class: &'static str,
}

impl SettingsRow {
    const fn new(
        label: &'static str,
        description: &'static str,
        badge: &'static str,
        badge_class: &'static str,
    ) -> Self {
        Self {
            label,
            description,
            badge,
            badge_class,
        }
    }
}

fn settings_tree_sections() -> Vec<SidebarSectionModel> {
    vec![
        SidebarSectionModel::settings_tree(
            "应用",
            vec![
                settings_tree_item(SettingsRoute::General, 0),
                settings_tree_item(SettingsRoute::Appearance, 1),
                settings_tree_item(SettingsRoute::Window, 1),
            ],
        ),
        SidebarSectionModel::settings_tree(
            "插件与技能",
            vec![
                settings_tree_item(SettingsRoute::Plugins, 0),
                settings_tree_item(SettingsRoute::Skills, 1),
            ],
        ),
        SidebarSectionModel::settings_tree(
            "终端与环境",
            vec![
                settings_tree_item(SettingsRoute::Commands, 0),
                settings_tree_item(SettingsRoute::Environment, 1),
                settings_tree_item(SettingsRoute::Deployment, 1),
            ],
        ),
        SidebarSectionModel::settings_tree(
            "项目与账号",
            vec![
                settings_tree_item(SettingsRoute::ProjectDefaults, 0)
                    .with_detail("同步空间 / 仓库绑定"),
                settings_tree_item(SettingsRoute::GitAccounts, 1)
                    .with_detail("GitHub / Gitee / GitLab"),
            ],
        ),
        SidebarSectionModel::settings_tree(
            "系统",
            vec![
                settings_tree_item(SettingsRoute::Network, 0),
                settings_tree_item(SettingsRoute::Shortcuts, 1),
                settings_tree_item(SettingsRoute::Privacy, 1),
                settings_tree_item(SettingsRoute::Advanced, 1),
            ],
        ),
    ]
}

fn settings_tree_item(route: SettingsRoute, depth: u8) -> SidebarItemModel {
    SidebarItemModel::tree(route.id(), route.title(), route.mark(), depth)
}

fn discover_statuses(config: &GitAccountConfig) -> Vec<GitHostingAccountStatus> {
    AuthDiscovery::system().discover_all(&AuthDiscoveryOptions {
        config: config.clone(),
    })
}

fn configured_statuses(config: &GitAccountConfig) -> Vec<GitHostingAccountStatus> {
    GitHostingProvider::ALL
        .iter()
        .map(|provider| GitHostingAccountStatus {
            provider: *provider,
            configured_username: config.configured_username(*provider),
            sessions: Vec::new(),
            login_flows: settings_login_flows(*provider),
        })
        .collect()
}

fn settings_login_flows(provider: GitHostingProvider) -> Vec<AuthLoginFlow> {
    let info = provider.info();
    let mut flows = Vec::new();

    if provider == GitHostingProvider::GitHub {
        flows.push(AuthLoginFlow {
            method: AuthMethod::GhCli,
            label: "复用 gh 登录态".to_string(),
            url: None,
            command: Some(vec![
                "gh".to_string(),
                "auth".to_string(),
                "login".to_string(),
                "--hostname".to_string(),
                info.host.to_string(),
            ]),
            stores_secret: false,
            description: "检测到 gh 后优先复用本机系统凭据中的 GitHub 登录态。".to_string(),
        });
    }

    flows.push(AuthLoginFlow {
        method: AuthMethod::Web,
        label: "网页登录".to_string(),
        url: Some(info.web_login_url.to_string()),
        command: None,
        stores_secret: false,
        description: format!("打开 {} 的网页登录入口。", info.label),
    });
    flows.push(AuthLoginFlow {
        method: AuthMethod::Token,
        label: "令牌登录".to_string(),
        url: Some(info.token_url.to_string()),
        command: None,
        stores_secret: true,
        description: "本版只提供令牌入口，不把令牌明文写入配置文件。".to_string(),
    });

    flows
}

fn open_external_url(url: &str) -> std::io::Result<()> {
    // Native shell open is deliberately kept at the app boundary. az-git only
    // describes login flows; the Dioxus host decides how to present them.
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).status().map(|_| ())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .map(|_| ())
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        Command::new("xdg-open").arg(url).status().map(|_| ())
    }
}

fn sync_username_for_provider(
    config: &GitAccountConfig,
    statuses: &[GitHostingAccountStatus],
    provider: GitHostingProvider,
) -> Option<String> {
    statuses
        .iter()
        .find(|status| status.provider == provider)
        .and_then(|status| {
            status
                .sessions
                .iter()
                .find(|session| {
                    matches!(session.state, AuthState::Connected | AuthState::Available)
                })
                .and_then(|session| session.username.as_ref())
                .map(|username| username.trim().to_string())
                .filter(|username| !username.is_empty())
        })
        .or_else(|| config.configured_username(provider))
}

fn configured_or_session_username(status: &GitHostingAccountStatus) -> String {
    status
        .configured_username
        .clone()
        .or_else(|| {
            status
                .sessions
                .iter()
                .find(|session| {
                    matches!(session.state, AuthState::Connected | AuthState::Available)
                })
                .and_then(|session| session.username.clone())
        })
        .unwrap_or_default()
}

fn primary_auth_state(status: &GitHostingAccountStatus) -> AuthState {
    if status
        .sessions
        .iter()
        .any(|session| session.state == AuthState::Connected)
    {
        AuthState::Connected
    } else if status
        .sessions
        .iter()
        .any(|session| session.state == AuthState::Available)
    {
        AuthState::Available
    } else if status.configured_username.is_some() {
        AuthState::Available
    } else if status
        .sessions
        .iter()
        .any(|session| session.state == AuthState::Error)
    {
        AuthState::Error
    } else {
        AuthState::NotDetected
    }
}

fn provider_mark(provider: GitHostingProvider) -> &'static str {
    match provider {
        GitHostingProvider::GitHub => "GH",
        GitHostingProvider::Gitee => "GE",
        GitHostingProvider::GitLab => "GL",
    }
}

fn provider_badge_class(provider: GitHostingProvider) -> &'static str {
    match provider {
        GitHostingProvider::GitHub => {
            "settings-provider-card__badge settings-provider-card__badge--github"
        }
        GitHostingProvider::Gitee => {
            "settings-provider-card__badge settings-provider-card__badge--gitee"
        }
        GitHostingProvider::GitLab => {
            "settings-provider-card__badge settings-provider-card__badge--gitlab"
        }
    }
}

fn username_placeholder(provider: GitHostingProvider) -> &'static str {
    match provider {
        GitHostingProvider::GitHub => "GitHub 用户名",
        GitHostingProvider::Gitee => "Gitee 用户名",
        GitHostingProvider::GitLab => "GitLab 用户名",
    }
}

fn auth_state_label(state: AuthState) -> &'static str {
    match state {
        AuthState::Connected => "已登录",
        AuthState::Available => "已配置",
        AuthState::NotDetected => "未检测",
        AuthState::Error => "异常",
    }
}

fn auth_state_class(state: AuthState) -> &'static str {
    match state {
        AuthState::Connected => {
            "settings-provider-card__status settings-provider-card__status--connected"
        }
        AuthState::Available => {
            "settings-provider-card__status settings-provider-card__status--available"
        }
        AuthState::NotDetected => "settings-provider-card__status",
        AuthState::Error => "settings-provider-card__status settings-provider-card__status--error",
    }
}

fn auth_method_label(method: AuthMethod) -> &'static str {
    match method {
        AuthMethod::GhCli => "gh 命令行",
        AuthMethod::Web => "网页",
        AuthMethod::Token => "令牌",
    }
}

fn auth_session_class(state: AuthState) -> &'static str {
    match state {
        AuthState::Connected => "auth-session auth-session--connected",
        AuthState::Available => "auth-session",
        AuthState::NotDetected => "auth-session auth-session--muted",
        AuthState::Error => "auth-session auth-session--error",
    }
}

fn login_flow_class(method: AuthMethod) -> &'static str {
    match method {
        AuthMethod::GhCli => "settings-login-flow settings-login-flow--cli",
        AuthMethod::Web => "settings-login-flow settings-login-flow--web",
        AuthMethod::Token => "settings-login-flow settings-login-flow--token",
    }
}
