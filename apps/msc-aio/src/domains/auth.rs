use dioxus::prelude::*;
use dioxus_components::{ContentHeader, Field, Surface, SurfaceHeader, WorkbenchButton};
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdMoonStar, LdSun},
};

use crate::app::Route;
use crate::state::{AuthSession, ThemePrefs};

#[component]
pub fn LoginPage() -> Element {
    let mut auth = use_context::<AuthSession>();
    let mut theme = use_context::<ThemePrefs>();
    let auth_api = use_context::<crate::state::AppServices>().auth_api.clone();

    let nav = use_navigator();
    let mut username = use_signal(|| {
        if cfg!(debug_assertions) {
            "admin".to_string()
        } else {
            String::new()
        }
    });
    let mut password = use_signal(|| {
        if cfg!(debug_assertions) {
            "admin".to_string()
        } else {
            String::new()
        }
    });
    let mut err = use_signal::<Option<String>>(|| None);
    let mut submitting = use_signal(|| false);
    let is_dark = *theme.dark_mode.read();
    let session_hint = if cfg!(debug_assertions) {
        "开发模式默认 admin/admin。".to_string()
    } else {
        "签名 Cookie 单管理员会话。".to_string()
    };
    let redirect_nav = nav;

    use_effect(move || {
        if *auth.ready.read() && *auth.logged_in.read() {
            redirect_nav.replace(Route::Home);
        }
    });

    let submit = move |_| {
        let user = username.read().trim().to_string();
        let pass = password.read().trim().to_string();
        if user.is_empty() || pass.is_empty() {
            err.set(Some("请输入用户名和密码".to_string()));
            return;
        }
        submitting.set(true);
        let auth_api = auth_api.clone();
        spawn(async move {
            match auth_api
                .login(addzero_agent_runtime_contract::LoginRequest {
                    username: user.clone(),
                    password: pass,
                })
                .await
            {
                Ok(session) if session.authenticated => {
                    auth.username.set(session.username.unwrap_or(user));
                    auth.logged_in.set(true);
                    auth.ready.set(true);
                    err.set(None);
                    nav.replace(Route::Home);
                }
                Ok(_) => {
                    err.set(Some("登录失败：服务端没有返回有效会话".to_string()));
                }
                Err(error) => {
                    err.set(Some(error.to_string()));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "login-shell",
            div { class: "login-shell__theme-toggle",
                WorkbenchButton {
                    class: "icon-button".to_string(),
                    onclick: move |_| theme.dark_mode.set(!is_dark),
                    title: if is_dark { "切换到白天模式" } else { "切换到黑夜模式" },
                    if is_dark {
                        Icon { width: 16, height: 16, icon: LdSun }
                    } else {
                        Icon { width: 16, height: 16, icon: LdMoonStar }
                    }
                }
            }
            Surface {
                ContentHeader {
                    title: "用户登录".to_string(),
                    subtitle: "进入当前工作台。".to_string(),
                }
                SurfaceHeader {
                    title: "登录凭据".to_string(),
                    subtitle: session_hint
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
                        class: "action-button action-button--primary".to_string(),
                        onclick: submit,
                        if *submitting.read() { "登录中…" } else { "登录" }
                    }
                }
            }
        }
    }
}
