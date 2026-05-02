use dioxus::prelude::*;
use dioxus_components::{ContentHeader, Surface, SurfaceHeader, Textarea, WorkbenchButton};

use crate::admin::domains::CHAT_DOMAIN_ID;
use crate::{
    services::{ChatMessageDto, ChatRequestDto},
    state::AppServices,
};

const CHAT_WORKBENCH_PAGE_ID: &str = "chat-workbench";

#[component]
pub fn ChatWorkbench() -> Element {
    let services = use_context::<AppServices>();
    let chat_api = services.openai_chat.clone();
    let messages = use_signal(Vec::<ChatMessageDto>::new);
    let input = use_signal(String::new);
    let feedback = use_signal(|| None::<String>);
    let pending = use_signal(|| false);
    let streaming = use_signal(|| false);

    rsx! {
        ContentHeader {
            title: "AI 聊天".to_string(),
            subtitle: "使用系统设置里的 OpenAI 兼容配置直接聊天。".to_string()
        }
        if let Some(message) = feedback.read().clone() {
            div {
                class: if message.contains("失败") { "callout" } else { "callout callout--info" },
                "{message}"
            }
        }
        Surface {
            SurfaceHeader {
                title: "聊天面板".to_string(),
                subtitle: "当前支持多轮文本对话。".to_string()
            }
            div { class: "stack content-stack",
                div { class: "stack content-stack",
                    for item in messages.read().iter() {
                        {
                            let speaker = if item.role == "user" { "你" } else { "AI" };
                            rsx! {
                                div { class: "callout",
                                    strong { "{speaker}: " }
                                    span { "{item.content}" }
                                }
                            }
                        }
                    }
                    if messages.read().is_empty() {
                        div { class: "empty-state", "还没有消息，先发一条试试。" }
                    }
                }
                Textarea {
                    label: "输入消息".to_string(),
                    value: input.read().clone(),
                    rows: Some(6),
                    placeholder: Some("直接开始聊天…".to_string()),
                    on_input: {
                        let mut input = input;
                        move |value| input.set(value)
                    }
                }
                div { class: "entry-actions",
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        disabled: *pending.read() || *streaming.read(),
                        onclick: {
                            let chat_api = chat_api.clone();
                            let mut messages = messages;
                            let mut input = input;
                            let mut feedback = feedback;
                            let mut pending = pending;
                            let mut streaming = streaming;
                            move |_| {
                                let content = input.read().trim().to_string();
                                if content.is_empty() || *pending.read() || *streaming.read() {
                                    return;
                                }
                                messages.with_mut(|items| {
                                    items.push(ChatMessageDto {
                                        role: "user".to_string(),
                                        content: content.clone(),
                                    });
                                });
                                input.set(String::new());
                                pending.set(true);
                                let chat_api = chat_api.clone();
                                let request = ChatRequestDto {
                                    messages: messages.read().clone(),
                                };
                                spawn(async move {
                                    match chat_api.chat(request).await {
                                        Ok(response) => {
                                            pending.set(false);
                                            streaming.set(true);
                                            messages.with_mut(|items| {
                                                items.push(ChatMessageDto {
                                                    role: response.message.role.clone(),
                                                    content: String::new(),
                                                });
                                            });
                                            for chunk in chunk_text(response.message.content.as_str(), 24) {
                                                messages.with_mut(|items| {
                                                    if let Some(last) = items.last_mut() {
                                                        last.content.push_str(chunk.as_str());
                                                    }
                                                });
                                                sleep_ms(24).await;
                                            }
                                            feedback.set(None);
                                        }
                                        Err(err) => feedback.set(Some(format!("聊天失败：{err}"))),
                                    }
                                    pending.set(false);
                                    streaming.set(false);
                                });
                            }
                        },
                        if *pending.read() || *streaming.read() { "输出中…" } else { "发送" }
                    }
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        disabled: messages.read().is_empty() || *pending.read() || *streaming.read(),
                        onclick: {
                            let mut messages = messages;
                            let mut feedback = feedback;
                            move |_| {
                                messages.set(Vec::new());
                                feedback.set(Some("当前会话已清空。".to_string()));
                            }
                        },
                        "清空会话"
                    }
                }
            }
        }
    }
}

fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;
    for ch in text.chars() {
        current.push(ch);
        count += 1;
        if count >= chunk_size {
            chunks.push(current);
            current = String::new();
            count = 0;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(u64::from(ms))).await;
}

addzero_admin_plugin_registry::register_admin_page! {
    id: CHAT_WORKBENCH_PAGE_ID,
    domain: CHAT_DOMAIN_ID,
    parent: None,
    label: "聊天工作台",
    order: 10,
    href: "/chat",
    active_patterns: &["/chat"],
    permissions_any_of: &[],
}
