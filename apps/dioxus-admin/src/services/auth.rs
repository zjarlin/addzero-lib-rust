use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;

use addzero_agent_runtime_contract::{LoginRequest, SessionUser};
use thiserror::Error;

use super::skills::LocalBoxFuture;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AuthServiceError {
    #[error("{0}")]
    Message(String),
}

impl AuthServiceError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type AuthServiceResult<T> = Result<T, AuthServiceError>;

const DEV_ADMIN_USERNAME: &str = "admin";
#[cfg(target_arch = "wasm32")]
const DEV_ADMIN_PASSWORD: &str = "admin";

pub trait AuthApi: 'static {
    fn current_session(&self) -> LocalBoxFuture<'_, AuthServiceResult<SessionUser>>;

    fn login(&self, input: LoginRequest) -> LocalBoxFuture<'_, AuthServiceResult<SessionUser>>;

    fn logout(&self) -> LocalBoxFuture<'_, AuthServiceResult<()>>;
}

pub type SharedAuthApi = Rc<dyn AuthApi>;

pub fn default_auth_api() -> SharedAuthApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserAuthApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedAuthApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserAuthApi;

#[cfg(target_arch = "wasm32")]
impl AuthApi for BrowserAuthApi {
    fn current_session(&self) -> LocalBoxFuture<'_, AuthServiceResult<SessionUser>> {
        Box::pin(async move {
            match super::browser_http::get_json::<SessionUser>("/api/admin/session").await {
                Ok(session) if session.authenticated => Ok(session),
                Ok(session) => try_browser_dev_login().await.unwrap_or(Ok(session)),
                Err(err) => {
                    if let Some(session) = try_browser_dev_login().await.transpose()? {
                        Ok(session)
                    } else if cfg!(debug_assertions) {
                        Ok(dev_session())
                    } else {
                        Err(AuthServiceError::new(err))
                    }
                }
            }
        })
    }

    fn login(&self, input: LoginRequest) -> LocalBoxFuture<'_, AuthServiceResult<SessionUser>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/session/login", &input)
                .await
                .map_err(AuthServiceError::new)
        })
    }

    fn logout(&self) -> LocalBoxFuture<'_, AuthServiceResult<()>> {
        Box::pin(async move {
            let payload = serde_json::json!({});
            super::browser_http::post_json::<_, SessionUser>("/api/admin/session/logout", &payload)
                .await
                .map(|_| ())
                .map_err(AuthServiceError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static CURRENT_USER: RefCell<Option<String>> = RefCell::new(default_embedded_user());
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedAuthApi;

#[cfg(not(target_arch = "wasm32"))]
impl AuthApi for EmbeddedAuthApi {
    fn current_session(&self) -> LocalBoxFuture<'_, AuthServiceResult<SessionUser>> {
        Box::pin(async move {
            let username = CURRENT_USER.with(|slot| slot.borrow().clone());
            Ok(SessionUser {
                authenticated: username.is_some(),
                username,
            })
        })
    }

    fn login(&self, input: LoginRequest) -> LocalBoxFuture<'_, AuthServiceResult<SessionUser>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let cookie = backend
                .admin_auth
                .authenticate(&input)
                .ok_or_else(|| AuthServiceError::new("用户名或密码不正确"))?;
            let _ = cookie;
            let username = input.username.trim().to_string();
            CURRENT_USER.with(|slot| slot.replace(Some(username.clone())));
            Ok(SessionUser {
                authenticated: true,
                username: Some(username),
            })
        })
    }

    fn logout(&self) -> LocalBoxFuture<'_, AuthServiceResult<()>> {
        Box::pin(async move {
            CURRENT_USER.with(|slot| slot.replace(None));
            Ok(())
        })
    }
}

#[cfg(target_arch = "wasm32")]
async fn try_browser_dev_login() -> Option<AuthServiceResult<SessionUser>> {
    if !cfg!(debug_assertions) {
        return None;
    }

    Some(
        super::browser_http::post_json(
            "/api/admin/session/login",
            &LoginRequest {
                username: DEV_ADMIN_USERNAME.to_string(),
                password: DEV_ADMIN_PASSWORD.to_string(),
            },
        )
        .await
        .map_err(AuthServiceError::new),
    )
}

#[cfg(target_arch = "wasm32")]
fn dev_session() -> SessionUser {
    SessionUser {
        authenticated: true,
        username: Some(DEV_ADMIN_USERNAME.to_string()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_embedded_user() -> Option<String> {
    cfg!(debug_assertions).then(|| DEV_ADMIN_USERNAME.to_string())
}
