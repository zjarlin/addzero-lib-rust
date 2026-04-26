use dioxus::prelude::*;
use dioxus_components::{ContentHeader, Field, Surface, SurfaceHeader, WorkbenchButton};

use crate::Route;

#[derive(Clone, Copy)]
pub struct AuthSession {
    pub logged_in: Signal<bool>,
    pub username: Signal<String>,
}

#[component]
pub fn LoginPage() -> Element {
    let mut auth = use_context::<AuthSession>();
    let nav = use_navigator();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut err = use_signal::<Option<String>>(|| None);

    if *auth.logged_in.read() {
        nav.replace(Route::Dashboard);
    }

    let submit = move |_| {
        let user = username.read().trim().to_string();
        let pass = password.read().trim().to_string();
        if user.is_empty() || pass.is_empty() {
            err.set(Some("请输入用户名和密码".to_string()));
            return;
        }
        auth.username.set(user);
        auth.logged_in.set(true);
        nav.replace(Route::Dashboard);
    };

    rsx! {
        div { class: "login-shell",
            Surface {
                ContentHeader {
                    title: "用户登录".to_string(),
                    subtitle: "登录后可访问知识库、Agent 和系统管理场景。".to_string(),
                }
                SurfaceHeader {
                    title: "登录凭据".to_string(),
                    subtitle: "演示阶段使用本地登录态，不接后端鉴权。".to_string()
                }
                div { class: "login-form",
                    Field {
                        label: "用户名".to_string(),
                        value: username.read().clone(),
                        placeholder: "admin".to_string(),
                        on_input: move |v: String| username.set(v),
                    }
                    Field {
                        label: "密码".to_string(),
                        value: password.read().clone(),
                        placeholder: "请输入密码".to_string(),
                        on_input: move |v: String| password.set(v),
                    }
                    if let Some(msg) = err.read().as_ref() {
                        div { class: "callout", "{msg}" }
                    }
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        onclick: submit,
                        "登录"
                    }
                }
            }
        }
    }
}
