use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, Field, MetricRow, MetricStrip, SidebarSection, Stack, Surface, SurfaceHeader,
    Tone, WorkbenchButton,
};

use crate::{
    admin::domains::AGENTS_DOMAIN_ID,
    app::Route,
    domains::asset_chat::{AssetChatFact, AssetChatKind, AssetChatPanel},
    services::{SkillDto, SkillSourceDto, SyncReportDto},
    state::AppServices,
};

const AGENTS_PAGE_ID: &str = "agent-assets";

#[component]
pub fn Agents() -> Element {
    let mut search = use_signal(String::new);
    let mut source_filter = use_signal(|| SkillSourceFilter::All);
    let mut feedback = use_signal::<Option<String>>(|| None);
    let skills_api = use_context::<AppServices>().skills.clone();

    let mut skills_resource = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            async move { skills_api.list_skills().await }
        })
    };
    let mut status_resource = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            async move { skills_api.server_status().await }
        })
    };

    let do_sync = move || {
        let skills_api = skills_api.clone();
        spawn(async move {
            match skills_api.sync_skills().await {
                Ok(report) => {
                    feedback.set(Some(format_sync_report(&report)));
                    skills_resource.restart();
                    status_resource.restart();
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
                    title: "Skills".to_string(),
                    subtitle: "这里承载单个 Skill 定义与同步状态，不是市场分发页。".to_string()
                }
                Surface {
                    div { class: "callout",
                        "无法加载 Skill 列表：{err}"
                    }
                }
            };
        }
        None => {
            return rsx! {
                ContentHeader {
                    title: "Skills".to_string(),
                    subtitle: "这里承载单个 Skill 定义与同步状态，不是市场分发页。".to_string()
                }
                Surface { div { class: "empty-state", "正在加载…" } }
            };
        }
    };

    let query = search.read().trim().to_lowercase();
    let active_filter = *source_filter.read();
    let mut filtered: Vec<SkillDto> = raw_skills
        .iter()
        .filter(|skill| {
            active_filter.matches(&skill.source)
                && (query.is_empty()
                    || skill.name.to_lowercase().contains(&query)
                    || skill
                        .keywords
                        .iter()
                        .any(|keyword| keyword.to_lowercase().contains(&query))
                    || skill.description.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();
    filtered.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.name.cmp(&right.name))
    });

    let hero_preview = if filtered.is_empty() {
        raw_skills.iter().take(8).cloned().collect::<Vec<_>>()
    } else {
        filtered.iter().take(8).cloned().collect::<Vec<_>>()
    };
    let namespace_stats = build_namespace_stats(&filtered);
    let namespace_total = raw_skills
        .iter()
        .map(|skill| skill_group_label(&skill.name))
        .collect::<BTreeSet<_>>()
        .len();
    let keyword_total = raw_skills
        .iter()
        .flat_map(|skill| skill.keywords.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let hybrid_total = raw_skills
        .iter()
        .filter(|skill| matches!(&skill.source, SkillSourceDto::Both))
        .count();
    let fresh_total = raw_skills
        .iter()
        .filter(|skill| skill.updated_at >= Utc::now() - Duration::days(7))
        .count();
    let status_view = status_resource.read();
    let status_report = match status_view.as_ref() {
        Some(Ok(report)) => Some(report.clone()),
        _ => None,
    };
    let status_note = match status_view.as_ref() {
        Some(Ok(report)) => format_status_note(report),
        Some(Err(err)) => format!("状态读取失败：{err}"),
        None => "正在探测 PG / FS 同步状态…".to_string(),
    };
    let status_fs_root = status_report
        .as_ref()
        .map(|report| report.fs_root.clone())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| "—".to_string());
    let pg_online = status_report
        .as_ref()
        .is_some_and(|report| report.pg_online);

    rsx! {
        ContentHeader {
            title: "Skills".to_string(),
            subtitle: "复刻 skills.sh 的目录体验，但这里仍然是 Skill 资产定义与同步入口。".to_string()
        }
        div { class: "skills-page",
            section { class: "skills-hero",
                div { class: "skills-hero__intro",
                    div { class: "skills-eyebrow", "THE OPEN AGENT SKILLS MATRIX" }
                    h1 { class: "skills-hero__title", "SKILLS" }
                    p { class: "skills-hero__lede",
                        "将单个 agent 的程序性知识沉淀为可检索、可同步、可审计的 Skill 资产。页面结构参考 skills.sh，但展示主体改为稳定生成的矩阵图标。"
                    }
                    div { class: "skills-command",
                        div { class: "skills-command__label", "SYNC ROOT" }
                        code { class: "skills-command__value", "{status_fs_root}" }
                        div { class: "skills-command__meta",
                            span {
                                class: if pg_online { "skills-status skills-status--online" } else { "skills-status skills-status--offline" },
                                if pg_online { "PG + FS" } else { "FS ONLY" }
                            }
                            span { class: "skills-command__hint", "{status_note}" }
                        }
                        button {
                            class: "skills-cta",
                            onclick: move |_| do_sync(),
                            "立即同步"
                        }
                    }
                    div { class: "skills-hero__stats",
                        SkillHeroStat {
                            label: "Catalog".to_string(),
                            value: raw_skills.len().to_string(),
                            note: "已索引 Skill".to_string(),
                        }
                        SkillHeroStat {
                            label: "Namespaces".to_string(),
                            value: namespace_total.to_string(),
                            note: "命名空间".to_string(),
                        }
                        SkillHeroStat {
                            label: "Keywords".to_string(),
                            value: keyword_total.to_string(),
                            note: "唯一关键词".to_string(),
                        }
                        SkillHeroStat {
                            label: "Fresh".to_string(),
                            value: fresh_total.to_string(),
                            note: "7 天内更新".to_string(),
                        }
                    }
                }
                div { class: "skills-hero__showcase",
                    div { class: "skills-eyebrow skills-eyebrow--muted", "VISIBLE IN THIS WORKSPACE" }
                    div { class: "skills-showcase-grid",
                        if hero_preview.is_empty() {
                            div { class: "skills-showcase-empty", "没有可展示的 Skill。" }
                        } else {
                            for skill in hero_preview.iter() {
                                HeroSkillPreviewCard { skill: skill.clone() }
                            }
                        }
                    }
                    div { class: "skills-showcase__footer",
                        div { class: "skills-showcase__summary",
                            span { "Visible {filtered.len()}" }
                            span { "Hybrid {hybrid_total}" }
                        }
                        div { class: "skills-chip-row",
                            for stat in namespace_stats.iter().take(6) {
                                span { class: "skills-chip",
                                    span { class: "skills-chip__label", "{stat.label}" }
                                    span { class: "skills-chip__count", "{stat.count}" }
                                }
                            }
                        }
                    }
                }
            }
            section { class: "skills-catalog",
                div { class: "skills-catalog__header",
                    div {
                        div { class: "skills-eyebrow skills-eyebrow--muted", "SKILLS CATALOG" }
                        h2 { class: "skills-catalog__title", "Skill Matrix" }
                        p { class: "skills-catalog__subtitle",
                            "按名称、关键词和来源筛选；点击卡片查看单个 Skill 的只读详情与正文。"
                        }
                    }
                    div { class: "skills-catalog__summary",
                        span { "{filtered.len()} visible" }
                        span { "{raw_skills.len()} total" }
                        if let Some(report) = status_report.as_ref() {
                            span { "{report.conflicts.len()} conflicts" }
                        }
                    }
                }
                div { class: "skills-catalog__toolbar",
                    input {
                        class: "skills-search__input",
                        placeholder: "Search skills, keywords, namespaces…",
                        value: search.read().clone(),
                        oninput: move |evt| search.set(evt.value())
                    }
                    div { class: "skills-filter-row",
                        for filter in SkillSourceFilter::ALL.iter().copied() {
                            button {
                                class: if filter == active_filter { "skills-filter is-active" } else { "skills-filter" },
                                onclick: move |_| source_filter.set(filter),
                                "{filter.label()}"
                            }
                        }
                    }
                }
                if let Some(msg) = feedback.read().as_ref() {
                    div { class: "callout callout--info", "{msg}" }
                }
                if filtered.is_empty() {
                    div { class: "empty-state skills-empty-state",
                        "没有匹配的 Skill。"
                    }
                } else {
                    div { class: "skills-grid",
                        for skill in filtered.iter() {
                            SkillCard { skill: skill.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct NamespaceStat {
    label: String,
    count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SkillSourceFilter {
    All,
    FileSystem,
    Postgres,
    Both,
}

impl SkillSourceFilter {
    const ALL: [Self; 4] = [Self::All, Self::FileSystem, Self::Postgres, Self::Both];

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::FileSystem => "FS",
            Self::Postgres => "PG",
            Self::Both => "Hybrid",
        }
    }

    fn matches(self, source: &SkillSourceDto) -> bool {
        match self {
            Self::All => true,
            Self::FileSystem => matches!(source, SkillSourceDto::FileSystem),
            Self::Postgres => matches!(source, SkillSourceDto::Postgres),
            Self::Both => matches!(source, SkillSourceDto::Both),
        }
    }
}

#[component]
fn SkillHeroStat(label: String, value: String, note: String) -> Element {
    rsx! {
        div { class: "skills-stat",
            div { class: "skills-stat__label", "{label}" }
            div { class: "skills-stat__value", "{value}" }
            div { class: "skills-stat__note", "{note}" }
        }
    }
}

#[component]
fn HeroSkillPreviewCard(skill: SkillDto) -> Element {
    let nav = use_navigator();
    let route = Route::AgentEditor {
        name: skill.name.clone(),
    };
    let namespace = skill_group_label(&skill.name);

    rsx! {
        button {
            class: "skills-preview-card",
            onclick: move |_| {
                nav.push(route.clone());
            },
            SkillGlyph { name: skill.name.clone(), compact: true }
            div { class: "skills-preview-card__meta",
                div { class: "skills-preview-card__title", "{skill_leaf_label(&skill.name)}" }
                div { class: "skills-preview-card__subtitle", "{namespace}" }
            }
        }
    }
}

#[component]
fn SkillCard(skill: SkillDto) -> Element {
    let nav = use_navigator();
    let route = Route::AgentEditor {
        name: skill.name.clone(),
    };
    let namespace = skill_group_label(&skill.name);
    let updated = skill.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let preview = skill_description_preview(&skill);

    rsx! {
        button {
            class: "skill-card",
            onclick: move |_| {
                nav.push(route.clone());
            },
            div { class: "skill-card__header",
                SkillGlyph { name: skill.name.clone() }
                span {
                    class: format!("skill-source-pill {}", source_pill_class(&skill.source)),
                    "{source_short_label(&skill.source)}"
                }
            }
            div { class: "skill-card__body",
                div { class: "skill-card__namespace", "{namespace}" }
                h3 { class: "skill-card__title", "{skill_leaf_label(&skill.name)}" }
                div { class: "skill-card__slug", "{skill.name}" }
                p { class: "skill-card__description", "{preview}" }
            }
            if !skill.keywords.is_empty() {
                div { class: "skill-card__tags",
                    for keyword in skill.keywords.iter().take(4) {
                        span { class: "skill-tag", "{keyword}" }
                    }
                }
            }
            div { class: "skill-card__meta",
                span { "{updated}" }
                span { "{skill.body.lines().count()} lines" }
            }
        }
    }
}

#[component]
fn SkillGlyph(name: String, #[props(default = false)] compact: bool) -> Element {
    let class = if compact {
        "skill-glyph skill-glyph--compact"
    } else {
        "skill-glyph"
    };
    let hue = skill_hue(&name);
    let accent = (hue + 46) % 360;
    let cells = skill_matrix_cells(&name);

    rsx! {
        div {
            class: class,
            style: format!("--skill-hue: {hue}; --skill-accent: {accent};"),
            for (idx, active) in cells.into_iter().enumerate() {
                span {
                    key: "{idx}",
                    class: if active { "skill-glyph__cell is-on" } else { "skill-glyph__cell" }
                }
            }
        }
    }
}

fn build_namespace_stats(skills: &[SkillDto]) -> Vec<NamespaceStat> {
    let mut stats = BTreeMap::<String, usize>::new();

    for skill in skills {
        let label = skill_group_label(&skill.name);
        *stats.entry(label).or_default() += 1;
    }

    let mut stats = stats
        .into_iter()
        .map(|(label, count)| NamespaceStat { label, count })
        .collect::<Vec<_>>();
    stats.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    stats
}

fn skill_group_label(name: &str) -> String {
    if let Some((head, _)) = name.split_once('/') {
        return head.to_string();
    }
    if let Some((head, _)) = name.split_once(':') {
        return head.to_string();
    }
    name.split('-')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("ungrouped")
        .to_string()
}

fn skill_leaf_label(name: &str) -> String {
    name.rsplit(|ch| ch == '/' || ch == ':')
        .next()
        .unwrap_or(name)
        .to_string()
}

fn skill_description_preview(skill: &SkillDto) -> String {
    if !skill.description.trim().is_empty() {
        return skill.description.clone();
    }

    skill
        .body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or("No description.")
        .to_string()
}

fn source_short_label(source: &SkillSourceDto) -> &'static str {
    match source {
        SkillSourceDto::Postgres => "PG",
        SkillSourceDto::FileSystem => "FS",
        SkillSourceDto::Both => "HY",
    }
}

fn source_pill_class(source: &SkillSourceDto) -> &'static str {
    match source {
        SkillSourceDto::Postgres => "skill-source-pill--pg",
        SkillSourceDto::FileSystem => "skill-source-pill--fs",
        SkillSourceDto::Both => "skill-source-pill--both",
    }
}

fn format_status_note(report: &SyncReportDto) -> String {
    let last_sync = report
        .finished_at
        .map(|time| time.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "尚未同步".to_string());
    if report.pg_online {
        format!("{last_sync} · {} conflicts", report.conflicts.len())
    } else {
        format!("{last_sync} · PG offline")
    }
}

fn skill_hue(name: &str) -> u16 {
    (stable_skill_hash(name) % 360) as u16
}

fn skill_matrix_cells(name: &str) -> [bool; 9] {
    let hash = stable_skill_hash(name);
    let mut cells = [false; 9];

    for (idx, cell) in cells.iter_mut().enumerate() {
        let bit = ((hash >> (idx * 3 % 48)) & 1) == 1;
        *cell = bit;
    }

    cells[4] = true;
    cells
}

fn stable_skill_hash(input: &str) -> u64 {
    let mut acc: u64 = 1469598103934665603;

    for byte in input.bytes() {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1099511628211);
    }

    acc
}

#[component]
pub fn AgentEditor(name: String) -> Element {
    let skills_api = use_context::<AppServices>().skills.clone();
    let is_new = name == "_new";
    let load_name = name.clone();
    let mut name_state = use_signal(|| if is_new { String::new() } else { name.clone() });
    let mut keywords_state = use_signal(Vec::new);
    let mut description_state = use_signal(String::new);
    let mut body_state = use_signal(String::new);
    let mut source_state = use_signal(|| SkillSourceDto::FileSystem);
    let mut updated_at_state = use_signal::<Option<DateTime<Utc>>>(|| None);
    let mut hash_state = use_signal(String::new);
    let mut feedback = use_signal::<Option<String>>(|| None);
    let mut loading = use_signal(|| !is_new);
    let mut chat_draft = use_signal(String::new);

    let _loader = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            let load_name = load_name.clone();
            async move {
                if is_new {
                    loading.set(false);
                    return;
                }
                match skills_api.get_skill(load_name).await {
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
                        feedback.set(Some("未找到该 Skill。".into()));
                        loading.set(false);
                    }
                    Err(err) => {
                        feedback.set(Some(format!("加载失败：{err}")));
                        loading.set(false);
                    }
                }
            }
        })
    };

    let header_title = if is_new {
        "Skill 资产".to_string()
    } else {
        format!("Skill：{name}")
    };
    let updated_display = match *updated_at_state.read() {
        Some(timestamp) => timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "—".into(),
    };
    let hash_display = {
        let hash = hash_state.read().clone();
        if hash.is_empty() {
            "—".to_string()
        } else {
            hash.chars().take(12).collect::<String>() + "…"
        }
    };
    let source_display = source_label(&source_state.read());

    rsx! {
        ContentHeader {
            title: header_title,
            subtitle: "这里是单个 Skill 定义查看页，不是 Skills 市场；只读展示 SKILL.md 元信息、关键词与正文。".to_string(),
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
                    title: "Skill 元信息".to_string(),
                    subtitle: "name 是单个 Skill 在文件夹与 PG 里的主键，关键词决定触发范围。".to_string()
                }
                MetricStrip { columns: 2,
                    Field {
                        label: "名称".to_string(),
                        value: name_state.read().clone(),
                        readonly: true,
                        placeholder: "kebab-case，例如 my-skill".to_string(),
                    }
                    Field {
                        label: "来源".to_string(),
                        value: source_display.clone(),
                        readonly: true,
                    }
                    Field {
                        label: "最后更新".to_string(),
                        value: updated_display.clone(),
                        readonly: true,
                    }
                    Field {
                        label: "Content hash".to_string(),
                        value: hash_display.clone(),
                        readonly: true,
                    }
                }
                if !keywords_state.read().is_empty() {
                    div { class: "knowledge-meta",
                        for keyword in keywords_state.read().iter() {
                            span { class: "badge", "{keyword}" }
                        }
                    }
                }
            }
            Surface {
                SurfaceHeader {
                    title: "说明与正文".to_string(),
                    subtitle: "description 是当前 Skill 的 SKILL.md frontmatter；正文来自来源目录，只读展示。".to_string()
                }
                AssetChatPanel {
                    kind: AssetChatKind::Skill,
                    object_title: name_state.read().clone(),
                    facts: vec![
                        AssetChatFact::new("来源", source_display.clone()),
                        AssetChatFact::new("最后更新", updated_display.clone()),
                        AssetChatFact::new("Content hash", hash_display.clone()),
                        AssetChatFact::new("关键词", keywords_state.read().join("，")),
                    ],
                    draft: chat_draft.read().clone(),
                    placeholder: "输入 Skill 触发、正文调整、安装或同步记录".to_string(),
                    readonly_excerpt: Some(description_state.read().clone()),
                    on_draft: move |value| chat_draft.set(value),
                    on_submit: move |_| chat_draft.set(String::new()),
                }
                Stack {
                    div { class: "readonly-block",
                        div { class: "knowledge-detail__label", "Description" }
                        div { class: "knowledge-excerpt", "{description_state.read()}" }
                    }
                    div { class: "readonly-block",
                        div { class: "knowledge-detail__label", "Body (Markdown)" }
                        pre { class: "knowledge-excerpt knowledge-excerpt--mono", "{body_state.read()}" }
                    }
                }
            }
            if let Some(msg) = feedback.read().as_ref() {
                div { class: "callout callout--info", "{msg}" }
            }
            div { class: "editor-footer",
                Link { to: Route::Agents,
                    WorkbenchButton { class: "toolbar-button".to_string(), "返回" }
                }
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
pub fn AgentContext() -> Element {
    let skills_api = use_context::<AppServices>().skills.clone();
    let status = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            async move { skills_api.server_status().await }
        })
    };
    let view = status.read();

    let (pg_online, fs_root, last_report) = match view.as_ref() {
        Some(Ok(report)) => (
            report.pg_online,
            report.fs_root.clone(),
            Some(report.clone()),
        ),
        Some(Err(_)) | None => (false, String::new(), None),
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

addzero_admin_plugin_registry::register_admin_page! {
    id: AGENTS_PAGE_ID,
    domain: AGENTS_DOMAIN_ID,
    parent: None,
    label: "Skill 资产",
    order: 10,
    href: "/agents",
    active_patterns: &["/agents", "/agents/:name"],
    permissions_any_of: &["knowledge:skill"],
}
