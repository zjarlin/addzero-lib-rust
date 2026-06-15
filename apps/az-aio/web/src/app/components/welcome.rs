#![allow(non_snake_case)]

use dioxus::prelude::*;

use super::model::PageChrome;

#[derive(PartialEq, Clone, Props)]
pub(super) struct WelcomeStartProps {
    pub(super) page: PageChrome,
}

pub(super) fn WelcomeStart(props: WelcomeStartProps) -> Element {
    rsx! {
        div { class: "codex-start",
            h1 { "我们该构建什么？" }
            ComposerPrompt { page_title: props.page.title }
            StarterList {}
            div { class: "empty-panel empty-panel--compact",
                div { class: "empty-panel__mark", "{props.page.mark}" }
                p { "{props.page.subtitle}" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct ComposerPromptProps {
    page_title: String,
}

fn ComposerPrompt(props: ComposerPromptProps) -> Element {
    rsx! {
        div { class: "codex-composer",
            div { class: "codex-composer__prompt",
                "用 Rudi 重构 {props.page_title} 全栈脚手架，保持插件前后端内聚"
            }
            div { class: "codex-composer__bar",
                span { class: "composer-icon", "+" }
                span { class: "composer-control", "⚙ 自定义⌄" }
                span { class: "composer-model", "5.5 超高⌄" }
                span { class: "composer-send", "↑" }
            }
            div { class: "codex-composer__meta",
                span { "▱ addzero-lib-rust" }
                span { "▱ 本地模式⌄" }
                span { "⑂ main⌄" }
            }
        }
    }
}

fn StarterList() -> Element {
    rsx! {
        div { class: "starter-list",
            a { href: "/?route=/lowcode", "继续 AZ AIO 低代码插件脚手架" }
            a { href: "/?route=/software", "整理插件目录与安装器能力" }
            a { href: "/?route=/config", "连接配置中心和本机环境" }
        }
    }
}
