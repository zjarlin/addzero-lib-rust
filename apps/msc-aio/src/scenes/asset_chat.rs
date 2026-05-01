use dioxus::prelude::*;
use dioxus_components::{Stack, Textarea, WorkbenchButton};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetChatKind {
    Note,
    Skill,
    Package,
}

impl AssetChatKind {
    fn label(self) -> &'static str {
        match self {
            Self::Note => "笔记",
            Self::Skill => "技能",
            Self::Package => "安装包",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetChatFact {
    pub label: String,
    pub value: String,
}

impl AssetChatFact {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AssetChatPanelProps {
    pub kind: AssetChatKind,
    pub object_title: String,
    pub facts: Vec<AssetChatFact>,
    pub draft: String,
    pub placeholder: String,
    #[props(optional)]
    pub readonly_excerpt: Option<String>,
    #[props(optional)]
    pub on_draft: Option<EventHandler<String>>,
    #[props(optional)]
    pub on_submit: Option<EventHandler<String>>,
}

#[component]
pub fn AssetChatPanel(props: AssetChatPanelProps) -> Element {
    let kind = props.kind.label();
    let title = if props.object_title.trim().is_empty() {
        format!("{kind}对象")
    } else {
        props.object_title.clone()
    };
    let draft = props.draft.clone();
    let can_submit = props
        .on_submit
        .as_ref()
        .map(|_| !draft.trim().is_empty())
        .unwrap_or(false);
    let on_draft = props.on_draft;
    let on_submit = props.on_submit;

    rsx! {
        div { class: "asset-chat",
            div { class: "asset-chat__thread",
                div { class: "asset-chat__message asset-chat__message--system",
                    div { class: "asset-chat__role", "{kind}" }
                    div { class: "asset-chat__body",
                        div { class: "asset-chat__title", "{title}" }
                        if !props.facts.is_empty() {
                            dl { class: "asset-chat__facts",
                                for fact in props.facts.iter() {
                                    dt { "{fact.label}" }
                                    dd { "{fact.value}" }
                                }
                            }
                        }
                    }
                }
                if let Some(excerpt) = props.readonly_excerpt.as_ref() {
                    div { class: "asset-chat__message asset-chat__message--assistant",
                        div { class: "asset-chat__role", "摘录" }
                        div { class: "asset-chat__body asset-chat__body--pre", "{excerpt}" }
                    }
                }
                if !draft.trim().is_empty() {
                    div { class: "asset-chat__message asset-chat__message--user",
                        div { class: "asset-chat__role", "输入" }
                        div { class: "asset-chat__body asset-chat__body--pre", "{draft}" }
                    }
                }
            }
            Stack {
                Textarea {
                    label: "聊天框".to_string(),
                    value: draft.clone(),
                    rows: 5,
                    placeholder: props.placeholder.clone(),
                    on_input: move |value| {
                        if let Some(handler) = on_draft.as_ref() {
                            handler.call(value);
                        }
                    }
                }
                div { class: "asset-chat__footer",
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        disabled: !can_submit,
                        onclick: move |_| {
                            if can_submit {
                                if let Some(handler) = on_submit.as_ref() {
                                    handler.call(draft.clone());
                                }
                            }
                        },
                        "发送"
                    }
                }
            }
        }
    }
}
