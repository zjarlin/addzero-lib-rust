use dioxus::prelude::*;
use dioxus_components::{ContentHeader, Field, Surface, SurfaceHeader, WorkbenchButton};

use crate::services::{
    BrandingLogoSource, LOGO_PREVIEW_BASE_URL, LogoUploadRequest, SharedBrandingSettingsApi,
};
use crate::state::{APP_ICON_ASSET_PATH, AppServices, BrandingState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    General,
    Branding,
    Security,
    Defaults,
    Storage,
}

impl SettingsTab {
    const ALL: [Self; 5] = [
        Self::General,
        Self::Branding,
        Self::Security,
        Self::Defaults,
        Self::Storage,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::General => "通用设置",
            Self::Branding => "品牌与站点",
            Self::Security => "安全与认证",
            Self::Defaults => "默认值",
            Self::Storage => "对象存储",
        }
    }
}

#[component]
pub fn SystemSettings() -> Element {
    let services = use_context::<AppServices>();
    let branding = services.branding;
    let active_tab = use_signal(|| SettingsTab::Branding);
    let feedback = use_signal::<Option<String>>(|| None);

    let site_name = use_signal(|| branding.state.read().site_name.clone());
    let site_tagline = use_signal(|| "面向对象、流程、Agent 和知识资产的统一工作台。".to_string());
    let compact_navigation = use_signal(|| false);
    let show_entry_dock = use_signal(|| true);

    let brand_copy = use_signal(|| branding.state.read().brand_copy.clone());
    let header_badge = use_signal(|| branding.state.read().header_badge.clone());

    let require_mfa = use_signal(|| true);
    let audit_notice = use_signal(|| true);
    let session_hours = use_signal(|| "12".to_string());

    let default_home = use_signal(|| "知识图谱概览".to_string());
    let default_lens = use_signal(|| "笔记".to_string());
    let default_theme = use_signal(|| "浅色".to_string());

    let storage_endpoint = use_signal(|| "http://127.0.0.1:9091".to_string());
    let storage_bucket = use_signal(|| "msc-aio".to_string());
    let public_base_url = use_signal(|| LOGO_PREVIEW_BASE_URL.to_string());
    let use_presigned_url = use_signal(|| true);

    let current_tab = *active_tab.read();

    rsx! {
        ContentHeader {
            title: "系统管理".to_string(),
            subtitle: "系统设置改为按专题分 tab 管理，布局参考控制台设置页，内容按当前后台场景收敛。".to_string()
        }

        div { class: "settings-page",
            div { class: "settings-tabs-shell",
                div { class: "settings-tabs",
                    for tab in SettingsTab::ALL {
                        SettingsTabButton {
                            tab,
                            active: tab == current_tab,
                            on_select: {
                                let mut active_tab = active_tab;
                                move || active_tab.set(tab)
                            }
                        }
                    }
                }
            }

            if let Some(message) = feedback.read().as_ref() {
                div {
                    class: if message.contains("失败") { "callout" } else { "callout callout--info" },
                    "{message}"
                }
            }

            div { class: "settings-stack",
                match current_tab {
                    SettingsTab::General => rsx! {
                        GeneralSettingsTab {
                            site_name,
                            site_tagline,
                            compact_navigation,
                            show_entry_dock,
                            feedback,
                        }
                    },
                    SettingsTab::Branding => rsx! {
                        BrandingSettingsTab {
                            site_name,
                            brand_copy,
                            header_badge,
                            feedback,
                        }
                    },
                    SettingsTab::Security => rsx! {
                        SecuritySettingsTab {
                            require_mfa,
                            audit_notice,
                            session_hours,
                            feedback,
                        }
                    },
                    SettingsTab::Defaults => rsx! {
                        DefaultValueSettingsTab {
                            default_home,
                            default_lens,
                            default_theme,
                            feedback,
                        }
                    },
                    SettingsTab::Storage => rsx! {
                        StorageSettingsTab {
                            storage_endpoint,
                            storage_bucket,
                            public_base_url,
                            use_presigned_url,
                            feedback,
                        }
                    },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SettingsTabButtonProps {
    tab: SettingsTab,
    active: bool,
    on_select: EventHandler<()>,
}

#[component]
fn SettingsTabButton(props: SettingsTabButtonProps) -> Element {
    let class = if props.active {
        "settings-tab settings-tab--active"
    } else {
        "settings-tab"
    };
    let on_select = props.on_select;

    rsx! {
        button {
            class: class,
            r#type: "button",
            onclick: move |_| on_select.call(()),
            "{props.tab.label()}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SettingsSelectFieldProps {
    label: String,
    value: String,
    options: Vec<(String, String)>,
    on_input: EventHandler<String>,
}

#[component]
fn SettingsSelectField(props: SettingsSelectFieldProps) -> Element {
    let on_input = props.on_input;

    rsx! {
        label { class: "field",
            span { class: "field__label", "{props.label}" }
            select {
                class: "field__input field__select",
                value: props.value,
                onchange: move |evt| {
                    on_input.call(evt.value());
                },
                for (option_value, option_label) in props.options {
                    option { value: option_value, "{option_label}" }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct GeneralSettingsTabProps {
    site_name: Signal<String>,
    site_tagline: Signal<String>,
    compact_navigation: Signal<bool>,
    show_entry_dock: Signal<bool>,
    feedback: Signal<Option<String>>,
}

#[component]
fn GeneralSettingsTab(props: GeneralSettingsTabProps) -> Element {
    let services = use_context::<AppServices>();
    let branding = services.branding;
    let branding_api = services.branding_settings.clone();

    rsx! {
        Surface {
            SurfaceHeader {
                title: "工作台基线".to_string(),
                subtitle: "定义系统设置页进入后台时的默认呈现方式。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: {
                            let branding_state = branding.state;
                            let site_name = props.site_name;
                            let feedback = props.feedback;
                            let branding_api = branding_api.clone();
                            move |_| {
                                let mut current = branding_state.read().clone();
                                current.site_name = site_name.read().trim().to_string();
                                persist_branding_state(
                                    branding_state,
                                    branding_api.clone(),
                                    feedback,
                                    current,
                                    "已保存工作台名称，顶部品牌区已同步更新。".to_string(),
                                );
                            }
                        },
                        "保存"
                    }
                )
            }
            div { class: "form-grid",
                Field {
                    label: "站点名称".to_string(),
                    value: props.site_name.read().clone(),
                    on_input: {
                        let mut site_name = props.site_name;
                        move |value| site_name.set(value)
                    }
                }
                Field {
                    label: "页头说明".to_string(),
                    value: props.site_tagline.read().clone(),
                    on_input: {
                        let mut site_tagline = props.site_tagline;
                        move |value| site_tagline.set(value)
                    }
                }
            }
        }

        Surface {
            SurfaceHeader {
                title: "界面行为".to_string(),
                subtitle: "这些项决定工作台打开后的默认密度与常用操作入口。".to_string()
            }
            div { class: "settings-list",
                SettingsToggleRow {
                    title: "紧凑导航".to_string(),
                    detail: "启用后，左侧导航会更强调信息密度，适合高频后台操作。".to_string(),
                    enabled: *props.compact_navigation.read(),
                    on_toggle: {
                        let mut compact_navigation = props.compact_navigation;
                        move || {
                            let next = !*compact_navigation.read();
                            compact_navigation.set(next);
                        }
                    }
                }
                SettingsToggleRow {
                    title: "保留条目录入入口".to_string(),
                    detail: "在知识图谱概览里保留贴图谱的条目录入面板，减少跳出当前上下文。".to_string(),
                    enabled: *props.show_entry_dock.read(),
                    on_toggle: {
                        let mut show_entry_dock = props.show_entry_dock;
                        move || {
                            let next = !*show_entry_dock.read();
                            show_entry_dock.set(next);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct BrandingSettingsTabProps {
    site_name: Signal<String>,
    brand_copy: Signal<String>,
    header_badge: Signal<String>,
    feedback: Signal<Option<String>>,
}

#[component]
fn BrandingSettingsTab(props: BrandingSettingsTabProps) -> Element {
    let services = use_context::<AppServices>();
    let branding = services.branding;
    let logo_storage = services.logo_storage.clone();
    let branding_api = services.branding_settings.clone();
    let mut uploading = use_signal(|| false);
    let current_state = branding.state.read().clone();
    let current_logo = current_state.logo.clone();
    let logo_source = current_state.logo_source;

    let logo_name = match logo_source {
        BrandingLogoSource::AppIcon => "app-icon.png".to_string(),
        BrandingLogoSource::CustomUpload => current_logo
            .as_ref()
            .map(|logo| logo.file_name.clone())
            .unwrap_or_default(),
        BrandingLogoSource::TextOnly => "不显示图形 Logo".to_string(),
    };
    let object_key = current_logo
        .as_ref()
        .map(|logo| logo.object_key.clone())
        .unwrap_or_default();
    let relative_path = current_logo
        .as_ref()
        .map(|logo| logo.relative_path.clone())
        .unwrap_or_default();
    let preview_url = current_state.active_logo_url().unwrap_or_default();
    let backend_label = match logo_source {
        BrandingLogoSource::AppIcon => "应用内置资产".to_string(),
        BrandingLogoSource::CustomUpload => current_logo
            .as_ref()
            .map(|logo| logo.backend_label.clone())
            .unwrap_or_else(storage_backend_hint),
        BrandingLogoSource::TextOnly => "顶部品牌位仅显示文字".to_string(),
    };
    let preview_empty_text = match logo_source {
        BrandingLogoSource::TextOnly => "当前设置为仅文字品牌位",
        BrandingLogoSource::CustomUpload => "还没有上传 logo",
        BrandingLogoSource::AppIcon => "App 图标未加载",
    };

    rsx! {
        Surface {
            SurfaceHeader {
                title: "品牌文案".to_string(),
                subtitle: "这一组字段控制 logo 旁边的站点识别文案。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: {
                            let branding_state = branding.state;
                            let site_name = props.site_name;
                            let feedback = props.feedback;
                            let brand_copy = props.brand_copy;
                            let header_badge = props.header_badge;
                            let branding_api = branding_api.clone();
                            move |_| {
                                let mut current = branding_state.read().clone();
                                current.site_name = site_name.read().trim().to_string();
                                current.brand_copy = brand_copy.read().trim().to_string();
                                current.header_badge = header_badge.read().trim().to_string();
                                persist_branding_state(
                                    branding_state,
                                    branding_api.clone(),
                                    feedback,
                                    current,
                                    "品牌配置已保存到数据库，顶部标题已同步更新。".to_string(),
                                );
                            }
                        },
                        "保存"
                    }
                )
            }
            div { class: "form-grid",
                Field {
                    label: "品牌名称".to_string(),
                    value: props.site_name.read().clone(),
                    on_input: {
                        let mut site_name = props.site_name;
                        move |value| site_name.set(value)
                    }
                }
                Field {
                    label: "顶部徽标文案".to_string(),
                    value: props.header_badge.read().clone(),
                    on_input: {
                        let mut header_badge = props.header_badge;
                        move |value| header_badge.set(value)
                    }
                }
                Field {
                    label: "品牌说明".to_string(),
                    value: props.brand_copy.read().clone(),
                    on_input: {
                        let mut brand_copy = props.brand_copy;
                        move |value| brand_copy.set(value)
                    }
                }
                Field {
                    label: "顶部渲染策略".to_string(),
                    value: "顶部仅展示品牌名称，Logo 不再占用左上角入口".to_string(),
                    readonly: true
                }
            }
        }

        Surface {
            SurfaceHeader {
                title: "品牌 Logo".to_string(),
                subtitle: "顶部品牌位可以使用 App 图标，也可以切换为数据库记录的自定义上传资产。".to_string()
            }
            div { class: "logo-source-options",
                for source in [
                    BrandingLogoSource::AppIcon,
                    BrandingLogoSource::CustomUpload,
                    BrandingLogoSource::TextOnly,
                ] {
                    LogoSourceButton {
                        source,
                        active: logo_source == source,
                        disabled: *uploading.read(),
                        on_select: {
                            let branding_state = branding.state;
                            let mut feedback = props.feedback;
                            let branding_api = branding_api.clone();
                            move |selected| {
                                let mut next = branding_state.read().clone();
                                if selected == BrandingLogoSource::CustomUpload && next.logo.is_none() {
                                    feedback.set(Some("先上传一个 logo，再切换为自定义上传。".to_string()));
                                    return;
                                }
                                next.logo_source = selected;
                                persist_branding_state(
                                    branding_state,
                                    branding_api.clone(),
                                    feedback,
                                    next,
                                    format!("Logo 来源已切换为{}。", selected.label()),
                                );
                            }
                        }
                    }
                }
            }
            div { class: "settings-grid",
                div { class: "settings-panel stack",
                    label { class: "upload-dropzone",
                        span { class: "upload-dropzone__eyebrow", "品牌入口" }
                        span { class: "upload-dropzone__title", "上传一张横向 logo" }
                        span { class: "upload-dropzone__detail", "上传后文件进入对象存储，数据库保存当前品牌配置和对象引用。" }
                        input {
                            class: "upload-dropzone__input",
                            r#type: "file",
                            accept: "image/*",
                            disabled: *uploading.read(),
                            onchange: move |evt| {
                                let Some(file) = evt.files().into_iter().next() else {
                                    return;
                                };

                                let mut feedback = props.feedback;
                                uploading.set(true);
                                feedback.set(Some("正在读取并上传 logo…".to_string()));

                                let logo_storage = logo_storage.clone();
                                let branding_api = branding_api.clone();
                                let mut feedback = props.feedback;
                                let mut uploading = uploading;
                                let mut branding_state = branding.state;

                                spawn(async move {
                                    let file_name = file.name();
                                    let content_type = file.content_type();
                                    match file.read_bytes().await {
                                        Ok(bytes) => {
                                            let upload = LogoUploadRequest {
                                                file_name,
                                                content_type,
                                                bytes: bytes.to_vec(),
                                            };
                                            match logo_storage.upload_logo(upload).await {
                                                Ok(stored) => {
                                                    let backend = stored.backend_label.clone();
                                                    let file_name = stored.file_name.clone();
                                                    let mut current = branding_state.read().clone();
                                                    current.logo = Some(stored.into());
                                                    current.logo_source = BrandingLogoSource::CustomUpload;
                                                    match branding_api
                                                        .save_settings(current.to_settings_update())
                                                        .await
                                                    {
                                                        Ok(saved) => {
                                                            branding_state.set(saved.into());
                                                            feedback.set(Some(format!(
                                                                "Logo 已更新并写入数据库：{file_name}，当前存储后端 {backend}"
                                                            )));
                                                        }
                                                        Err(err) => {
                                                            feedback.set(Some(format!(
                                                                "Logo 已上传但保存配置失败：{err}"
                                                            )));
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    feedback.set(Some(format!("上传失败：{err}")));
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            feedback.set(Some(format!("读取文件失败：{err}")));
                                        }
                                    }

                                    uploading.set(false);
                                });
                            }
                        }
                    }

                    div { class: "settings-note",
                        "内置 App 图标不需要上传；自定义 Logo 会走 RustFS / S3-compatible 存储，PG 只保存配置引用。"
                    }
                }

                div { class: "logo-preview-card stack",
                    div {
                        class: if preview_url.is_empty() { "logo-preview" } else { "logo-preview logo-preview--filled" },
                        if !preview_url.is_empty() {
                            img {
                                class: "logo-preview__image",
                                src: "{preview_url}",
                                alt: format!("{} preview", logo_name)
                            }
                        } else {
                            div { class: "logo-preview__empty", "{preview_empty_text}" }
                        }
                    }
                    Field {
                        label: "当前来源".to_string(),
                        value: logo_source.label().to_string(),
                        readonly: true
                    }
                    Field {
                        label: "当前文件".to_string(),
                        value: logo_name,
                        readonly: true
                    }
                    Field {
                        label: "对象键".to_string(),
                        value: object_key,
                        readonly: true,
                        placeholder: "上传后自动生成"
                    }
                    Field {
                        label: "MinIO 相对路径".to_string(),
                        value: relative_path,
                        readonly: true,
                        placeholder: "上传后返回 msc-aio 内相对路径"
                    }
                    Field {
                        label: "预览地址".to_string(),
                        value: preview_url.clone(),
                        readonly: true,
                        placeholder: APP_ICON_ASSET_PATH.to_string()
                    }
                    Field {
                        label: "存储后端".to_string(),
                        value: backend_label,
                        readonly: true
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct LogoSourceButtonProps {
    source: BrandingLogoSource,
    active: bool,
    disabled: bool,
    on_select: EventHandler<BrandingLogoSource>,
}

#[component]
fn LogoSourceButton(props: LogoSourceButtonProps) -> Element {
    let class = if props.active {
        "logo-source-button logo-source-button--active"
    } else {
        "logo-source-button"
    };
    let label = props.source.label();
    let source = props.source;
    let on_select = props.on_select;

    rsx! {
        button {
            class,
            r#type: "button",
            disabled: props.disabled,
            onclick: move |_| on_select.call(source),
            "{label}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SecuritySettingsTabProps {
    require_mfa: Signal<bool>,
    audit_notice: Signal<bool>,
    session_hours: Signal<String>,
    feedback: Signal<Option<String>>,
}

#[component]
fn SecuritySettingsTab(props: SecuritySettingsTabProps) -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "登录与会话".to_string(),
                subtitle: "按 tab 组织后，这里承接后台访问安全与会话有效期。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: {
                            let mut feedback = props.feedback;
                            move |_| feedback.set(Some("安全与认证设置已暂存，待接后端策略中心。".to_string()))
                        },
                        "保存"
                    }
                )
            }
            div { class: "settings-list",
                SettingsToggleRow {
                    title: "强制管理员二次认证".to_string(),
                    detail: "进入系统管理、菜单和角色等高风险页面前要求二次校验。".to_string(),
                    enabled: *props.require_mfa.read(),
                    on_toggle: {
                        let mut require_mfa = props.require_mfa;
                        move || {
                            let next = !*require_mfa.read();
                            require_mfa.set(next);
                        }
                    }
                }
                SettingsToggleRow {
                    title: "敏感操作提醒".to_string(),
                    detail: "对角色、菜单、对象存储这类配置变更给出更强的审计提示。".to_string(),
                    enabled: *props.audit_notice.read(),
                    on_toggle: {
                        let mut audit_notice = props.audit_notice;
                        move || {
                            let next = !*audit_notice.read();
                            audit_notice.set(next);
                        }
                    }
                }
            }
            div { class: "form-grid",
                Field {
                    label: "会话有效期（小时）".to_string(),
                    value: props.session_hours.read().clone(),
                    on_input: {
                        let mut session_hours = props.session_hours;
                        move |value| session_hours.set(value)
                    }
                }
                Field {
                    label: "高风险命令策略".to_string(),
                    value: "命令执行前需要 provider 明确给出导航或提交动作".to_string(),
                    readonly: true
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DefaultValueSettingsTabProps {
    default_home: Signal<String>,
    default_lens: Signal<String>,
    default_theme: Signal<String>,
    feedback: Signal<Option<String>>,
}

#[component]
fn DefaultValueSettingsTab(props: DefaultValueSettingsTabProps) -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "用户默认值".to_string(),
                subtitle: "给新用户或首次登录的后台账号一组稳定的默认工作上下文。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: {
                            let mut feedback = props.feedback;
                            move |_| feedback.set(Some("默认值草稿已更新，当前仅在前端态生效。".to_string()))
                        },
                        "保存"
                    }
                )
            }
            div { class: "form-grid",
                Field {
                    label: "默认首页".to_string(),
                    value: props.default_home.read().clone(),
                    on_input: {
                        let mut default_home = props.default_home;
                        move |value| default_home.set(value)
                    }
                }
                SettingsSelectField {
                    label: "默认知识视角".to_string(),
                    value: props.default_lens.read().clone(),
                    options: vec![
                        ("笔记".to_string(), "笔记".to_string()),
                        ("软件".to_string(), "软件".to_string()),
                        ("安装包".to_string(), "安装包".to_string()),
                    ],
                    on_input: {
                        let mut default_lens = props.default_lens;
                        move |value| default_lens.set(value)
                    }
                }
                SettingsSelectField {
                    label: "默认主题".to_string(),
                    value: props.default_theme.read().clone(),
                    options: vec![
                        ("浅色".to_string(), "浅色".to_string()),
                        ("深色".to_string(), "深色".to_string()),
                        ("跟随系统".to_string(), "跟随系统".to_string()),
                    ],
                    on_input: {
                        let mut default_theme = props.default_theme;
                        move |value| default_theme.set(value)
                    }
                }
                Field {
                    label: "首次进入提醒".to_string(),
                    value: "优先进入知识图谱概览，并展示最近知识更新".to_string(),
                    readonly: true
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StorageSettingsTabProps {
    storage_endpoint: Signal<String>,
    storage_bucket: Signal<String>,
    public_base_url: Signal<String>,
    use_presigned_url: Signal<bool>,
    feedback: Signal<Option<String>>,
}

#[component]
fn StorageSettingsTab(props: StorageSettingsTabProps) -> Element {
    let services = use_context::<AppServices>();
    let current_logo = services.branding.state.read().logo.clone();
    let backend_label = current_logo
        .as_ref()
        .map(|logo| logo.backend_label.clone())
        .unwrap_or_else(storage_backend_hint);

    rsx! {
        Surface {
            SurfaceHeader {
                title: "S3-Compatible 接入".to_string(),
                subtitle: "当前统一收敛到 MinIO；bucket 固定为 `msc-aio`，通过对象 key 的相对路径管理所有资源。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: {
                            let mut feedback = props.feedback;
                            move |_| feedback.set(Some("对象存储接入参数已暂存；当前上传链路统一走后台 API -> MinIO。".to_string()))
                        },
                        "保存"
                    }
                )
            }
            div { class: "form-grid",
                Field {
                    label: "Endpoint".to_string(),
                    value: props.storage_endpoint.read().clone(),
                    on_input: {
                        let mut storage_endpoint = props.storage_endpoint;
                        move |value| storage_endpoint.set(value)
                    }
                }
                Field {
                    label: "Bucket（固定）".to_string(),
                    value: props.storage_bucket.read().clone(),
                    readonly: true
                }
                Field {
                    label: "Public Base URL".to_string(),
                    value: props.public_base_url.read().clone(),
                    on_input: {
                        let mut public_base_url = props.public_base_url;
                        move |value| public_base_url.set(value)
                    }
                }
                Field {
                    label: "当前执行后端".to_string(),
                    value: backend_label,
                    readonly: true
                }
            }
            div { class: "settings-list",
                SettingsToggleRow {
                    title: "前端拼接公开预览域名".to_string(),
                    detail: "后台上传后只返回 relative_path，前端统一拼接 https://minio-api.addzero.site 做预览。".to_string(),
                    enabled: *props.use_presigned_url.read(),
                    on_toggle: {
                        let mut use_presigned_url = props.use_presigned_url;
                        move || {
                            let next = !*use_presigned_url.read();
                            use_presigned_url.set(next);
                        }
                    }
                }
            }
        }

        Surface {
            SurfaceHeader {
                title: "接入备注".to_string(),
                subtitle: "这组信息用于解释当前前后端边界，而不是替代正式的后端配置落点。".to_string()
            }
            div { class: "settings-list" ,
                div { class: "settings-row settings-row--static",
                    div { class: "settings-row__copy",
                        div { class: "settings-row__title", "MinIO 兼容性" }
                        div { class: "settings-row__detail", "MinIO 仍走 S3-compatible 协议，但后台现在只保留一个 `msc-aio` bucket，目录语义全部映射为对象 key 前缀。" }
                    }
                }
                div { class: "settings-row settings-row--static",
                    div { class: "settings-row__copy",
                        div { class: "settings-row__title", "前端边界" }
                        div { class: "settings-row__detail", "当前 msc-aio 通过后台上传口统一写入 MinIO；浏览器只消费后台返回的相对路径并自行拼接预览域名。" }
                    }
                }
                div { class: "settings-row settings-row--static",
                    div { class: "settings-row__copy",
                        div { class: "settings-row__title", "公开读要求" }
                        div { class: "settings-row__detail", "如果预览域名直接代理 MinIO 对象，目标 bucket 需要允许匿名下载，否则拼出的预览地址会返回 403。" }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SettingsToggleRowProps {
    title: String,
    detail: String,
    enabled: bool,
    on_toggle: EventHandler<()>,
}

#[component]
fn SettingsToggleRow(props: SettingsToggleRowProps) -> Element {
    let on_toggle = props.on_toggle;
    let switch_class = if props.enabled {
        "settings-switch settings-switch--on"
    } else {
        "settings-switch"
    };

    rsx! {
        div { class: "settings-row",
            div { class: "settings-row__copy",
                div { class: "settings-row__title", "{props.title}" }
                div { class: "settings-row__detail", "{props.detail}" }
            }
            button {
                class: switch_class,
                r#type: "button",
                "aria-pressed": if props.enabled { "true" } else { "false" },
                onclick: move |_| on_toggle.call(()),
                span { class: "settings-switch__knob" }
            }
        }
    }
}

fn storage_backend_hint() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        "Web -> Admin API -> MinIO · 浏览器只拿相对路径".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        "MinIO / S3-compatible · 使用 MSC_AIO_MINIO_* 环境变量，bucket 固定为 msc-aio".to_string()
    }
}

fn persist_branding_state(
    mut branding_state: Signal<BrandingState>,
    branding_api: SharedBrandingSettingsApi,
    mut feedback: Signal<Option<String>>,
    next: BrandingState,
    success_message: String,
) {
    spawn(async move {
        match branding_api.save_settings(next.to_settings_update()).await {
            Ok(saved) => {
                branding_state.set(saved.into());
                feedback.set(Some(success_message));
            }
            Err(err) => {
                feedback.set(Some(format!("保存失败：{err}")));
            }
        }
    });
}
