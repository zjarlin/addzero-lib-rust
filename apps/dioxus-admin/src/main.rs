use dioxus::prelude::*;
use dioxus_components::{
    AdminWorkbench, Badge, ConfirmDialog, ContentHeader, Divider, Field, KeywordChips, ListItem,
    MainContent, MetricRow, ResponsiveGrid, Sidebar, SidebarSection, SidebarSide, Stack, StatTile,
    Surface, SurfaceHeader, TabStrip, Textarea, ThinTopbar, Tone, WorkbenchButton,
};

mod api;
mod scenes;
#[cfg(feature = "server")]
mod server;

use api::{
    SkillDto, SkillSourceDto, SkillUpsertDto, SyncReportDto, delete_skill, list_skills,
    server_status, sync_skills, upsert_skill,
};
use scenes::auth::{AuthSession, LoginPage};
use scenes::knowledge_base::{ConfigFilesScene, NotesScene, SoftwareScene};
use scenes::system_management::{DepartmentsScene, MenusScene, RolesScene, UsersScene};

const STYLE: Asset = asset!("/assets/admin.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let logged_in = use_signal(|| false);
    let username = use_signal(String::new);
    use_context_provider(|| AuthSession {
        logged_in,
        username,
    });

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }
        Router::<Route> {}
    }
}

#[component]
fn Login() -> Element {
    rsx! { LoginPage {} }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/login")]
    Login,
    #[layout(AppLayout)]
    #[route("/")]
    Dashboard,
    #[route("/objects")]
    Objects,
    #[route("/workflows")]
    Workflows,
    #[route("/agents")]
    Agents,
    #[route("/agents/:name")]
    AgentEditor { name: String },
    #[route("/knowledge/notes")]
    KnowledgeNotes,
    #[route("/knowledge/software")]
    KnowledgeSoftware,
    #[route("/knowledge/configs")]
    KnowledgeConfigs,
    #[route("/system/users")]
    SystemUsers,
    #[route("/system/menus")]
    SystemMenus,
    #[route("/system/roles")]
    SystemRoles,
    #[route("/system/departments")]
    SystemDepartments,
    #[route("/audit")]
    Audit,
}

#[component]
fn AppLayout() -> Element {
    let auth = use_context::<AuthSession>();
    let nav = use_navigator();
    if !*auth.logged_in.read() {
        nav.replace(Route::Login);
        return rsx! {};
    }

    rsx! {
        AdminWorkbench {
            topbar: rsx!(Topbar {}),
            left: rsx!(LeftRail {}),
            center: rsx!(MainContent { Outlet::<Route> {} }),
            right: rsx!(RightRail {}),
        }
    }
}

#[component]
fn Topbar() -> Element {
    let mut auth = use_context::<AuthSession>();
    let nav = use_navigator();
    let user = auth.username.read().clone();
    rsx! {
        ThinTopbar {
            eyebrow: "DioxusLabs/components 参考初始化".to_string(),
            title: format!("Admin Workbench · {}", if user.is_empty() { "未登录用户" } else { &user }),
            right_actions: rsx!(
                WorkbenchButton { class: "icon-button".to_string(), "搜索" }
                WorkbenchButton { class: "icon-button".to_string(), "通知" }
                WorkbenchButton {
                    class: "toolbar-button".to_string(),
                    onclick: move |_| {
                        auth.logged_in.set(false);
                        auth.username.set(String::new());
                        nav.replace(Route::Login);
                    },
                    "退出"
                }
                WorkbenchButton {
                    class: "action-button".to_string(),
                    tone: Tone::Accent,
                    "发布变更"
                }
            )
        }
    }
}

#[component]
fn LeftRail() -> Element {
    rsx! {
        Sidebar { side: SidebarSide::Left,
            SidebarSection { label: "工作区".to_string(),
                WorkbenchButton { class: "sidebar__workspace".to_string(), "运营控制台" }
            }
            SidebarSection { label: "导航".to_string(),
                NavLink { to: Route::Dashboard, label: "总览" }
                NavLink { to: Route::Objects, label: "对象管理" }
                NavLink { to: Route::Workflows, label: "流程编排" }
                NavLink { to: Route::Agents, label: "Agent 技能" }
                NavLink { to: Route::KnowledgeNotes, label: "知识库 / 笔记" }
                NavLink { to: Route::KnowledgeSoftware, label: "知识库 / 软件" }
                NavLink { to: Route::KnowledgeConfigs, label: "知识库 / 配置文件" }
                NavLink { to: Route::SystemUsers, label: "系统管理 / 用户" }
                NavLink { to: Route::SystemMenus, label: "系统管理 / 菜单" }
                NavLink { to: Route::SystemRoles, label: "系统管理 / 角色" }
                NavLink { to: Route::SystemDepartments, label: "系统管理 / 部门" }
                NavLink { to: Route::Audit, label: "审计日志" }
            }
        }
    }
}

#[component]
fn NavLink(to: Route, label: &'static str) -> Element {
    let current = use_route::<Route>();
    let is_agents_subroute =
        matches!(&current, Route::AgentEditor { .. }) && matches!(&to, Route::Agents);
    let knowledge_group_active = matches!(
        (&current, &to),
        (
            Route::KnowledgeNotes | Route::KnowledgeSoftware | Route::KnowledgeConfigs,
            Route::KnowledgeNotes
        )
    );
    let system_group_active = matches!(
        (&current, &to),
        (
            Route::SystemUsers | Route::SystemMenus | Route::SystemRoles | Route::SystemDepartments,
            Route::SystemUsers
        )
    );
    let class =
        if current == to || is_agents_subroute || knowledge_group_active || system_group_active {
            "nav-item nav-item--active"
        } else {
            "nav-item"
        };

    rsx! {
        Link { to, class: class, "{label}" }
    }
}

#[component]
fn Dashboard() -> Element {
    rsx! {
        ContentHeader {
            title: "运行概览".to_string(),
            subtitle: "上层是工作台布局，下层开始分化成后台场景。".to_string(),
            actions: rsx!(SceneTabs { active: "总览" })
        }
        SummarySection {}
        ObjectsSurface {}
        ConfigSurface {}
    }
}

#[component]
fn Objects() -> Element {
    rsx! {
        ContentHeader {
            title: "对象管理".to_string(),
            subtitle: "从列表、筛选和同页编辑开始长。".to_string(),
            actions: rsx!(SceneTabs { active: "对象管理" })
        }
        ObjectsSurface {}
        ConfigSurface {}
    }
}

#[component]
fn Workflows() -> Element {
    rsx! {
        ContentHeader {
            title: "流程编排".to_string(),
            subtitle: "先给流程面板和阶段分组，后面再接动作节点。".to_string(),
            actions: rsx!(SceneTabs { active: "流程编排" })
        }
        Surface {
            SurfaceHeader {
                title: "流程阶段".to_string(),
                subtitle: "保持单页上下文，不把流程拆成厚重向导。".to_string()
            }
            ResponsiveGrid { columns: 3,
                StatTile { label: "采集".to_string(), value: "12".to_string(), detail: "等待输入映射".to_string() }
                StatTile { label: "校验".to_string(), value: "04".to_string(), detail: "规则链已挂载".to_string() }
                StatTile { label: "分发".to_string(), value: "09".to_string(), detail: "2 个节点待审批".to_string() }
            }
        }
        Surface {
            SurfaceHeader {
                title: "流程参数".to_string(),
                subtitle: "继续沿用双栏表单。".to_string()
            }
            ResponsiveGrid { columns: 2,
                Field { label: "流程名称".to_string(), value: "履约流程 B-02".to_string() }
                Field { label: "当前版本".to_string(), value: "2026.04".to_string() }
                Field { label: "触发条件".to_string(), value: "订单已支付".to_string() }
                Field { label: "重试策略".to_string(), value: "3 次指数退避".to_string() }
            }
        }
    }
}

#[component]
fn Audit() -> Element {
    rsx! {
        ContentHeader {
            title: "审计日志".to_string(),
            subtitle: "右侧上下文栏继续承担辅助理解，不抢主内容权重。".to_string(),
            actions: rsx!(SceneTabs { active: "审计日志" })
        }
        Surface {
            SurfaceHeader {
                title: "最近日志".to_string(),
                subtitle: "先有时间线，再接过滤器和详情抽屉。".to_string()
            }
            Stack {
                ListItem { title: "09:12 审计任务结束".to_string(), detail: "策略审计通过，没有新增风险项".to_string(), meta: "system".to_string() }
                ListItem { title: "08:41 权限模板变更".to_string(), detail: "P-09 删除了一条遗留白名单".to_string(), meta: "Chen".to_string() }
                ListItem { title: "昨天 18:20 发布完成".to_string(), detail: "release-0426 已部署到 production".to_string(), meta: "Luna".to_string() }
            }
        }
    }
}

#[component]
fn KnowledgeNotes() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库".to_string(),
            subtitle: "知识域和系统域分离，避免混写在同一个业务模块。".to_string(),
            actions: rsx!(SceneTabs { active: "知识库" })
        }
        NotesScene {}
    }
}

#[component]
fn KnowledgeSoftware() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库".to_string(),
            subtitle: "软件资产和系统配置文件分成独立子场景。".to_string(),
            actions: rsx!(SceneTabs { active: "知识库" })
        }
        SoftwareScene {}
    }
}

#[component]
fn KnowledgeConfigs() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库".to_string(),
            subtitle: "配置文件知识沉淀，支持后续审计和回滚策略。".to_string(),
            actions: rsx!(SceneTabs { active: "知识库" })
        }
        ConfigFilesScene {}
    }
}

#[component]
fn SystemUsers() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "系统域：用户、菜单、角色、部门分模块维护。".to_string(),
            actions: rsx!(SceneTabs { active: "系统管理" })
        }
        UsersScene {}
    }
}

#[component]
fn SystemMenus() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "菜单管理单独成模块，不与用户角色混杂。".to_string(),
            actions: rsx!(SceneTabs { active: "系统管理" })
        }
        MenusScene {}
    }
}

#[component]
fn SystemRoles() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "角色管理聚焦权限域，与部门管理解耦。".to_string(),
            actions: rsx!(SceneTabs { active: "系统管理" })
        }
        RolesScene {}
    }
}

#[component]
fn SystemDepartments() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "部门组织结构独立建模，支撑用户归属。".to_string(),
            actions: rsx!(SceneTabs { active: "系统管理" })
        }
        DepartmentsScene {}
    }
}

#[component]
fn Agents() -> Element {
    let mut search = use_signal(String::new);
    let mut feedback = use_signal::<Option<String>>(|| None);

    let mut skills_resource = use_resource(|| async move { list_skills().await });

    let do_sync = move || {
        spawn(async move {
            match sync_skills().await {
                Ok(report) => {
                    feedback.set(Some(format_sync_report(&report)));
                    skills_resource.restart();
                }
                Err(err) => feedback.set(Some(format!("同步失败：{err}"))),
            }
        });
    };

    let skills_view = skills_resource.read();
    let raw_skills = match skills_view.as_ref() {
        Some(Ok(list)) => list.clone(),
        Some(Err(err)) => {
            return rsx! {
                ContentHeader {
                    title: "Agent 技能".to_string(),
                    subtitle: "管理 SKILL.md 与触发关键词".to_string(),
                    actions: rsx!(SceneTabs { active: "Agent 技能" })
                }
                Surface {
                    div { class: "callout",
                        "无法加载技能列表：{err}"
                    }
                }
            };
        }
        None => {
            return rsx! {
                ContentHeader {
                    title: "Agent 技能".to_string(),
                    subtitle: "管理 SKILL.md 与触发关键词".to_string(),
                    actions: rsx!(SceneTabs { active: "Agent 技能" })
                }
                Surface { div { class: "empty-state", "正在加载…" } }
            };
        }
    };

    let query = search.read().to_lowercase();
    let filtered: Vec<SkillDto> = raw_skills
        .iter()
        .filter(|s| {
            if query.is_empty() {
                true
            } else {
                s.name.to_lowercase().contains(&query)
                    || s.keywords.iter().any(|k| k.to_lowercase().contains(&query))
                    || s.description.to_lowercase().contains(&query)
            }
        })
        .cloned()
        .collect();

    rsx! {
        ContentHeader {
            title: "Agent 技能".to_string(),
            subtitle: "管理 SKILL.md 与触发关键词".to_string(),
            actions: rsx!(
                SceneTabs { active: "Agent 技能" }
                Link { to: Route::AgentEditor { name: "_new".to_string() },
                    WorkbenchButton { class: "action-button".to_string(), tone: Tone::Accent, "新增技能" }
                }
            )
        }
        Surface {
            SurfaceHeader {
                title: "技能列表".to_string(),
                subtitle: "按名称或关键词搜索，行点击进入编辑。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "toolbar-button".to_string(),
                        onclick: move |_| do_sync(),
                        "手动同步"
                    }
                )
            }
            div { class: "toolbar",
                input {
                    class: "toolbar__search",
                    placeholder: "按名称 / 关键词搜索",
                    value: "{search}",
                    oninput: move |evt| search.set(evt.value())
                }
                span { class: "toolbar__spacer" }
                span { class: "cell-overflow", "共 {raw_skills.len()} 条" }
            }
            if let Some(msg) = feedback.read().as_ref() {
                div { class: "callout callout--info", "{msg}" }
            }
            if filtered.is_empty() {
                div { class: "empty-state", "没有匹配的技能。" }
            } else {
                table { class: "data-table",
                    thead {
                        tr {
                            th { "名称" }
                            th { "关键词" }
                            th { "来源" }
                            th { "更新时间" }
                        }
                    }
                    tbody {
                        for skill in filtered.iter() {
                            SkillRow { skill: skill.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SkillRow(skill: SkillDto) -> Element {
    let nav = use_navigator();
    let route = Route::AgentEditor {
        name: skill.name.clone(),
    };
    let preview: Vec<&String> = skill.keywords.iter().take(3).collect();
    let extra = skill.keywords.len().saturating_sub(preview.len());
    let badge = source_badge_props(&skill.source);
    let updated = skill.updated_at.format("%Y-%m-%d %H:%M").to_string();

    rsx! {
        tr {
            class: "row-link",
            onclick: move |_| { nav.push(route.clone()); },
            td { "{skill.name}" }
            td {
                div { class: "cell-keywords",
                    for keyword in preview.iter() {
                        span { class: "chip",
                            span { class: "chip__label", "{keyword}" }
                        }
                    }
                    if extra > 0 {
                        span { class: "cell-overflow", "+{extra}" }
                    }
                }
            }
            td {
                Badge { label: badge.0, variant: badge.1 }
            }
            td { "{updated}" }
        }
    }
}

fn source_badge_props(source: &SkillSourceDto) -> (String, String) {
    match source {
        SkillSourceDto::Postgres => ("PG".into(), "pg".into()),
        SkillSourceDto::FileSystem => ("FS".into(), "fs".into()),
        SkillSourceDto::Both => ("Both".into(), "both".into()),
    }
}

#[component]
fn AgentEditor(name: String) -> Element {
    let nav = use_navigator();
    let is_new = name == "_new";
    let load_name = name.clone();
    let mut name_state = use_signal(|| if is_new { String::new() } else { name.clone() });
    let mut keywords_state = use_signal::<Vec<String>>(Vec::new);
    let mut description_state = use_signal(String::new);
    let mut body_state = use_signal(String::new);
    let mut source_state = use_signal(|| SkillSourceDto::FileSystem);
    let mut updated_at_state = use_signal::<Option<chrono::DateTime<chrono::Utc>>>(|| None);
    let mut hash_state = use_signal(String::new);
    let mut feedback = use_signal::<Option<String>>(|| None);
    let mut confirm_open = use_signal(|| false);
    let mut loading = use_signal(|| !is_new);

    let _loader = use_resource(move || {
        let load_name = load_name.clone();
        async move {
            if is_new {
                loading.set(false);
                return;
            }
            match api::get_skill(load_name).await {
                Ok(Some(skill)) => {
                    name_state.set(skill.name.clone());
                    keywords_state.set(skill.keywords.clone());
                    description_state.set(skill.description.clone());
                    body_state.set(skill.body.clone());
                    source_state.set(skill.source.clone());
                    updated_at_state.set(Some(skill.updated_at));
                    hash_state.set(skill.content_hash.clone());
                    loading.set(false);
                }
                Ok(None) => {
                    feedback.set(Some("未找到该技能。".into()));
                    loading.set(false);
                }
                Err(err) => {
                    feedback.set(Some(format!("加载失败：{err}")));
                    loading.set(false);
                }
            }
        }
    });

    let save = move |_| {
        let payload = SkillUpsertDto {
            name: name_state.read().trim().to_string(),
            keywords: keywords_state.read().clone(),
            description: description_state.read().clone(),
            body: body_state.read().clone(),
        };
        if payload.name.is_empty() {
            feedback.set(Some("名称不能为空。".into()));
            return;
        }
        spawn(async move {
            feedback.set(Some("正在保存…".into()));
            match upsert_skill(payload).await {
                Ok(skill) => {
                    feedback.set(Some(format!("已保存（{}）。", source_label(&skill.source))));
                    name_state.set(skill.name.clone());
                    source_state.set(skill.source.clone());
                    updated_at_state.set(Some(skill.updated_at));
                    hash_state.set(skill.content_hash.clone());
                }
                Err(err) => feedback.set(Some(format!("保存失败：{err}"))),
            }
        });
    };

    let request_delete = move |_| confirm_open.set(true);
    let cancel_delete = move |_: ()| confirm_open.set(false);
    let confirm_delete = move |_: ()| {
        confirm_open.set(false);
        let name = name_state.read().clone();
        spawn(async move {
            match delete_skill(name).await {
                Ok(()) => {
                    nav.replace(Route::Agents);
                }
                Err(err) => feedback.set(Some(format!("删除失败：{err}"))),
            }
        });
    };

    let header_title = if is_new {
        "新增 Agent 技能".to_string()
    } else {
        format!("编辑：{}", name)
    };

    let updated_display = match *updated_at_state.read() {
        Some(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "—".into(),
    };
    let hash_display = {
        let h = hash_state.read().clone();
        if h.is_empty() {
            "—".to_string()
        } else {
            h.chars().take(12).collect::<String>() + "…"
        }
    };
    let source_display = source_label(&source_state.read());

    rsx! {
        ContentHeader {
            title: header_title,
            subtitle: "管理 SKILL.md 元信息、关键词触发与正文。".to_string(),
            actions: rsx!(
                Link { to: Route::Agents,
                    WorkbenchButton { class: "toolbar-button".to_string(), "返回列表" }
                }
            )
        }
        if *loading.read() {
            Surface { div { class: "empty-state", "正在加载…" } }
        } else {
            Surface {
                SurfaceHeader {
                    title: "元信息".to_string(),
                    subtitle: "name 是文件夹与 PG 的主键，关键词决定触发范围。".to_string()
                }
                ResponsiveGrid { columns: 2,
                    Field {
                        label: "名称".to_string(),
                        value: name_state.read().clone(),
                        readonly: !is_new,
                        on_input: move |v: String| name_state.set(v),
                        placeholder: "kebab-case，例如 my-skill".to_string(),
                    }
                    Field {
                        label: "来源".to_string(),
                        value: source_display.clone(),
                        readonly: true,
                    }
                    Field {
                        label: "最后更新".to_string(),
                        value: updated_display,
                        readonly: true,
                    }
                    Field {
                        label: "Content hash".to_string(),
                        value: hash_display,
                        readonly: true,
                    }
                }
                Divider {}
                div {
                    div { class: "field__label", style: "margin-bottom: 8px;", "触发关键词" }
                    KeywordChips {
                        value: keywords_state.read().clone(),
                        on_change: move |next: Vec<String>| keywords_state.set(next),
                    }
                    div { class: "context-line",
                        span { "保存时会写回 description 头部 \"当用户提到 …\" 模板段。" }
                    }
                }
            }
            Surface {
                SurfaceHeader {
                    title: "说明与正文".to_string(),
                    subtitle: "description 是 SKILL.md frontmatter；正文写技能的具体指令。".to_string()
                }
                Stack {
                    Textarea {
                        label: "Description".to_string(),
                        value: description_state.read().clone(),
                        rows: 4,
                        placeholder: "保存时关键词会自动渲染到开头".to_string(),
                        on_input: move |v: String| description_state.set(v),
                    }
                    Textarea {
                        label: "Body (Markdown)".to_string(),
                        value: body_state.read().clone(),
                        rows: 14,
                        monospace: true,
                        placeholder: "# 技能名\n\n操作指南……".to_string(),
                        on_input: move |v: String| body_state.set(v),
                    }
                }
            }
            if let Some(msg) = feedback.read().as_ref() {
                div { class: "callout callout--info", "{msg}" }
            }
            div { class: "editor-footer",
                if !is_new {
                    WorkbenchButton {
                        class: "toolbar-button".to_string(),
                        onclick: request_delete,
                        "删除"
                    }
                }
                span { class: "editor-footer__spacer" }
                Link { to: Route::Agents,
                    WorkbenchButton { class: "toolbar-button".to_string(), "取消" }
                }
                WorkbenchButton {
                    class: "action-button".to_string(),
                    tone: Tone::Accent,
                    onclick: save,
                    "保存"
                }
            }
            ConfirmDialog {
                open: *confirm_open.read(),
                title: "确认删除".to_string(),
                message: format!("将删除 SKILL.md 与（如果在线）PG 中的 {} 记录。该操作不可撤销。", name_state.read()),
                confirm_label: "删除".to_string(),
                cancel_label: "取消".to_string(),
                on_confirm: confirm_delete,
                on_cancel: cancel_delete,
            }
        }
    }
}

fn format_sync_report(report: &SyncReportDto) -> String {
    if !report.pg_online {
        return "PG 未连接：仅 fs 模式，无需同步。".into();
    }
    let total = report.added_to_fs.len()
        + report.added_to_pg.len()
        + report.updated_in_fs.len()
        + report.updated_in_pg.len();
    if total == 0 && report.conflicts.is_empty() {
        "已同步：两侧一致。".into()
    } else {
        format!(
            "同步完成：fs+{}, pg+{}, fs↑{}, pg↑{}, 冲突 {}",
            report.added_to_fs.len(),
            report.added_to_pg.len(),
            report.updated_in_fs.len(),
            report.updated_in_pg.len(),
            report.conflicts.len()
        )
    }
}

fn source_label(source: &SkillSourceDto) -> String {
    match source {
        SkillSourceDto::Postgres => "Postgres".into(),
        SkillSourceDto::FileSystem => "FileSystem".into(),
        SkillSourceDto::Both => "PG + FS".into(),
    }
}

#[component]
fn SummarySection() -> Element {
    rsx! {
        ResponsiveGrid { columns: 3,
            StatTile {
                label: "规则健康度".to_string(),
                value: "98.4%".to_string(),
                detail: "过去 24 小时没有阻断性变更".to_string()
            }
            StatTile {
                label: "待处理工单".to_string(),
                value: "14".to_string(),
                detail: "其中 3 个已超过 SLA 预警线".to_string()
            }
            StatTile {
                label: "执行队列".to_string(),
                value: "128".to_string(),
                detail: "平均处理耗时 1.8s".to_string()
            }
        }
    }
}

#[component]
fn ObjectsSurface() -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "对象列表".to_string(),
                subtitle: "表格和工具条作为内容层拼在布局件之上。".to_string(),
                actions: rsx!(
                    WorkbenchButton { class: "toolbar-button".to_string(), "筛选" }
                    WorkbenchButton { class: "toolbar-button".to_string(), "导出" }
                    WorkbenchButton { class: "toolbar-button".to_string(), "列设置" }
                )
            }
            table { class: "data-table",
                thead {
                    tr {
                        th { "对象" }
                        th { "负责人" }
                        th { "状态" }
                        th { "更新时间" }
                    }
                }
                tbody {
                    DataRow { name: "风控规则 A-17", owner: "Luna", status: "稳定", updated_at: "10:32" }
                    DataRow { name: "履约流程 B-02", owner: "Mika", status: "待校验", updated_at: "09:48" }
                    DataRow { name: "权限模板 P-09", owner: "Chen", status: "变更中", updated_at: "09:16" }
                    DataRow { name: "告警编排 S-11", owner: "Wen", status: "稳定", updated_at: "昨天" }
                }
            }
        }
    }
}

#[component]
fn ConfigSurface() -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "对象配置".to_string(),
                subtitle: "表单按器官分层，后续继续填入领域字段。".to_string()
            }
            TabStrip {
                WorkbenchButton { class: "form-tab".to_string(), tone: Tone::Accent, "基本信息" }
                WorkbenchButton { class: "form-tab".to_string(), "执行策略" }
                WorkbenchButton { class: "form-tab".to_string(), "通知" }
            }
            ResponsiveGrid { columns: 2,
                Field { label: "对象名称".to_string(), value: "风控规则 A-17".to_string() }
                Field { label: "业务域".to_string(), value: "支付".to_string() }
                Field { label: "负责人".to_string(), value: "Luna".to_string() }
                Field { label: "变更批次".to_string(), value: "release-0426".to_string() }
                Field { label: "优先级".to_string(), value: "P1".to_string() }
                Field { label: "执行环境".to_string(), value: "production".to_string() }
            }
            Divider {}
            Stack {
                ListItem {
                    title: "发布前检查".to_string(),
                    detail: "2 个字段需要补充说明".to_string()
                }
                ListItem {
                    title: "策略审计".to_string(),
                    detail: "最近一次审计通过".to_string()
                }
            }
        }
    }
}

#[component]
fn SceneTabs(active: &'static str) -> Element {
    let tab = |label: &'static str| -> (String, Option<Tone>) {
        if active == label {
            ("segment-button".to_string(), Some(Tone::Accent))
        } else {
            ("segment-button".to_string(), None)
        }
    };
    let total = tab("总览");
    let object = tab("对象管理");
    let workflow = tab("流程编排");
    let agent = tab("Agent 技能");
    let knowledge = tab("知识库");
    let system = tab("系统管理");
    let audit = tab("审计日志");

    rsx! {
        Link { to: Route::Dashboard,
            WorkbenchButton { class: total.0, tone: total.1, "总览" }
        }
        Link { to: Route::Objects,
            WorkbenchButton { class: object.0, tone: object.1, "对象" }
        }
        Link { to: Route::Workflows,
            WorkbenchButton { class: workflow.0, tone: workflow.1, "流程" }
        }
        Link { to: Route::Agents,
            WorkbenchButton { class: agent.0, tone: agent.1, "Agent" }
        }
        Link { to: Route::KnowledgeNotes,
            WorkbenchButton { class: knowledge.0, tone: knowledge.1, "知识库" }
        }
        Link { to: Route::SystemUsers,
            WorkbenchButton { class: system.0, tone: system.1, "系统管理" }
        }
        Link { to: Route::Audit,
            WorkbenchButton { class: audit.0, tone: audit.1, "审计" }
        }
    }
}

#[component]
fn RightRail() -> Element {
    let route = use_route::<Route>();
    let is_agent_scene = matches!(&route, Route::Agents | Route::AgentEditor { .. });

    rsx! {
        Sidebar { side: SidebarSide::Right,
            if is_agent_scene {
                AgentContext {}
            } else {
                DefaultContext {}
            }
        }
    }
}

#[component]
fn AgentContext() -> Element {
    let status = use_resource(|| async move { server_status().await });
    let view = status.read();

    let (pg_online, fs_root, last_report) = match view.as_ref() {
        Some(Ok(report)) => (
            report.pg_online,
            report.fs_root.clone(),
            Some(report.clone()),
        ),
        Some(Err(_)) => (false, String::new(), None),
        None => (false, String::new(), None),
    };

    rsx! {
        SidebarSection { label: "Agent 上下文".to_string(),
            Stack {
                MetricRow {
                    label: "Postgres".to_string(),
                    value: if pg_online { "Online".to_string() } else { "Offline".to_string() },
                    tone: if pg_online { Tone::Positive } else { Tone::Warning },
                }
                MetricRow {
                    label: "fs 根目录".to_string(),
                    value: if fs_root.is_empty() { "—".to_string() } else { fs_root.clone() },
                }
                if let Some(report) = &last_report {
                    MetricRow {
                        label: "上次同步冲突".to_string(),
                        value: report.conflicts.len().to_string(),
                        tone: if report.conflicts.is_empty() { Tone::Default } else { Tone::Warning },
                    }
                }
            }
        }
        if let Some(report) = last_report {
            SidebarSection { label: "最近同步".to_string(),
                if report.finished_at.is_none() {
                    div { class: "context-line", span { "尚未触发过同步。" } }
                } else {
                    Stack {
                        ContextLine { label: "fs 新增".to_string(), value: report.added_to_fs.len().to_string() }
                        ContextLine { label: "PG 新增".to_string(), value: report.added_to_pg.len().to_string() }
                        ContextLine { label: "fs 更新".to_string(), value: report.updated_in_fs.len().to_string() }
                        ContextLine { label: "PG 更新".to_string(), value: report.updated_in_pg.len().to_string() }
                        if !report.conflicts.is_empty() {
                            div { class: "callout",
                                "冲突项："
                                "{report.conflicts.join(\"，\")}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContextLine(label: String, value: String) -> Element {
    rsx! {
        div { class: "context-line",
            span { "{label}" }
            span { class: "context-line__value", "{value}" }
        }
    }
}

#[component]
fn DefaultContext() -> Element {
    rsx! {
        SidebarSection { label: "概览".to_string(),
            Stack {
                MetricRow { label: "成功率".to_string(), value: "99.2%".to_string(), tone: Tone::Positive }
                MetricRow { label: "告警数".to_string(), value: "03".to_string(), tone: Tone::Warning }
                MetricRow { label: "挂起变更".to_string(), value: "07".to_string() }
            }
        }
        SidebarSection { label: "最近动作".to_string(),
            Stack {
                ListItem {
                    title: "规则 A-17 已提交".to_string(),
                    detail: "Luna 更新了命中条件".to_string(),
                    meta: "2 分钟前".to_string()
                }
                ListItem {
                    title: "执行器扩容完成".to_string(),
                    detail: "队列延迟恢复到基线".to_string(),
                    meta: "18 分钟前".to_string()
                }
                ListItem {
                    title: "审计任务结束".to_string(),
                    detail: "未发现新增风险项".to_string(),
                    meta: "今天 09:12".to_string()
                }
            }
        }
    }
}

#[component]
fn DataRow(
    name: &'static str,
    owner: &'static str,
    status: &'static str,
    updated_at: &'static str,
) -> Element {
    let status_class = match status {
        "稳定" => "status-pill status-pill--ok",
        "待校验" => "status-pill status-pill--warn",
        _ => "status-pill",
    };

    rsx! {
        tr {
            td { "{name}" }
            td { "{owner}" }
            td { span { class: status_class, "{status}" } }
            td { "{updated_at}" }
        }
    }
}
