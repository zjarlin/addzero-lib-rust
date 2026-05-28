use crate::http::HttpClient;
use crate::{KiroAuthSupportError, KiroAuthSupportResult, KiroOidcConfig};
use az_derive_aliases::{
    apply, plain_clone_debug, plain_debug, plain_eq, plain_partial_eq, plain_partial_eq_display,
    serde_camel_eq_default, serde_camel_partial_eq_default, serde_code_default_enum,
    serialize_camel_eq,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use uuid::Uuid;

const PERSONAL_START_URL: &str = "https://view.awsapps.com/start";
const SOCIAL_START_URL: &str = "https://view.awsapps.com/start";
const ENTERPRISE_FALLBACK_START_URL: &str = "https://d-906600eb6f.awsapps.com/start";
const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
const DEFAULT_SCOPES: &[&str] = &[
    "codewhisperer:completions",
    "codewhisperer:analysis",
    "codewhisperer:conversations",
];

/// 用于选择 AWS Builder ID start URL 的 Kiro 登录类型。
#[apply(serde_code_default_enum)]
pub enum KiroLoginType {
    /// 个人 AWS Builder ID。
    #[default]
    Personal,
    /// 社交登录 AWS Builder ID 路径。
    Social,
    /// 企业 Identity Center start URL。
    Enterprise,
}

impl KiroLoginType {
    /// 解析发送给 OIDC 注册和设备端点的 start URL。
    #[must_use]
    pub fn resolve_start_url(self, enterprise_start_url: Option<&str>) -> String {
        match self {
            Self::Personal => PERSONAL_START_URL.to_owned(),
            Self::Social => SOCIAL_START_URL.to_owned(),
            Self::Enterprise => enterprise_start_url
                .map(str::trim)
                .filter(|url| url.starts_with("https://"))
                .map(|url| url.trim_end_matches('/').to_owned())
                .unwrap_or_else(|| ENTERPRISE_FALLBACK_START_URL.to_owned()),
        }
    }
}

/// `/client/register` 返回的已注册 OIDC 公共客户端。
#[apply(serde_camel_eq_default)]
pub struct KiroClientRegistration {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    #[serde(default)]
    pub client_id_issued_at: Option<i64>,
    #[serde(default)]
    pub client_secret_expires_at: Option<i64>,
    #[serde(default)]
    pub authorization_endpoint: Option<String>,
    #[serde(default)]
    pub token_endpoint: Option<String>,
}

/// `/device_authorization` 返回的设备授权元数据。
#[apply(serde_camel_eq_default)]
pub struct KiroDeviceAuthorization {
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub interval: Option<u64>,
}

impl KiroDeviceAuthorization {
    /// 返回用户授权期间浏览器应打开的最佳 URL。
    #[must_use]
    pub fn browser_verification_url(&self) -> &str {
        if self.verification_uri_complete.trim().is_empty() {
            &self.verification_uri
        } else {
            &self.verification_uri_complete
        }
    }
}

/// `/token` 返回的 access token 响应，包含错误响应。
#[apply(serde_camel_partial_eq_default)]
pub struct KiroTokenResponse {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl KiroTokenResponse {
    /// 响应包含 access token 时返回 `true`。
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.access_token
            .as_deref()
            .is_some_and(|token| !token.trim().is_empty())
    }
}

/// 后续轮询所需的完整设备流程会话材料。
#[apply(plain_eq)]
pub struct KiroDeviceFlow {
    /// 本 crate 生成的本地 UUID。
    pub uuid: String,
    /// 当前流程使用的登录类型。
    pub login_type: KiroLoginType,
    /// 解析后的 AWS Builder ID start URL。
    pub start_url: String,
    /// 已注册 OIDC 客户端材料。
    pub client: KiroClientRegistration,
    /// 设备授权材料。
    pub authorization: KiroDeviceAuthorization,
    /// 初始轮询间隔。
    pub poll_interval: Duration,
}

impl KiroDeviceFlow {
    /// 返回用户应打开以批准设备流程的 URL。
    #[must_use]
    pub fn verification_url(&self) -> &str {
        self.authorization.browser_verification_url()
    }

    /// 返回 provider 展示的用户代码。
    #[must_use]
    pub fn user_code(&self) -> &str {
        &self.authorization.user_code
    }
}

/// 设备 token 端点的一次轮询结果。
#[apply(plain_partial_eq)]
pub enum KiroTokenPoll {
    /// 用户授权仍在等待中。
    Pending,
    /// 服务端要求降低轮询频率。
    SlowDown {
        /// 调用方下一次应使用的间隔。
        next_interval: Duration,
    },
    /// 设备代码已过期或本地轮询已超时。
    Expired {
        /// 可读原因。
        message: String,
    },
    /// provider 返回了非预期 OAuth 错误。
    Error {
        /// 可读原因。
        message: String,
        /// 原始 token 响应。
        response: KiroTokenResponse,
    },
    /// token 交换已完成。
    Success(KiroTokenResponse),
}

impl KiroTokenPoll {
    /// 状态为成功、过期或错误时返回 `true`。
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Success(_) | Self::Expired { .. } | Self::Error { .. }
        )
    }
}

/// 阻塞式 Kiro 设备流程客户端。
#[apply(plain_clone_debug)]
pub struct KiroDeviceFlowClient {
    config: KiroOidcConfig,
    http: HttpClient,
}

impl KiroDeviceFlowClient {
    /// 根据已校验配置创建客户端。
    pub fn new(config: KiroOidcConfig) -> KiroAuthSupportResult<Self> {
        config.validate()?;
        Ok(Self {
            http: HttpClient::new(&config)?,
            config,
        })
    }

    /// 使用默认 AWS OIDC 端点和轮询选项创建客户端。
    pub fn default_client() -> KiroAuthSupportResult<Self> {
        Self::new(KiroOidcConfig::default())
    }

    /// 为 Kiro/AWS Builder ID start URL 注册公共 OIDC 客户端。
    pub fn register_client(
        &self,
        start_url: impl AsRef<str>,
    ) -> KiroAuthSupportResult<KiroClientRegistration> {
        let body = RegisterClientRequest {
            client_name: self.config.client_name.clone(),
            client_type: "public".to_owned(),
            issuer_url: start_url.as_ref().to_owned(),
            scopes: DEFAULT_SCOPES
                .iter()
                .map(|scope| (*scope).to_owned())
                .collect(),
        };
        let response = self.http.post("/client/register")?.json(&body).send()?;
        let registration: KiroClientRegistration = HttpClient::read_json(response)?;
        if registration.client_id.trim().is_empty() {
            return Err(KiroAuthSupportError::InvalidResponse(
                "register client response missing clientId".to_owned(),
            ));
        }
        Ok(registration)
    }

    /// 为已注册 OIDC 客户端启动设备授权。
    pub fn start_device_authorization(
        &self,
        client: &KiroClientRegistration,
        start_url: impl AsRef<str>,
    ) -> KiroAuthSupportResult<KiroDeviceAuthorization> {
        let body = DeviceAuthorizationRequest {
            client_id: client.client_id.clone(),
            client_secret: client.client_secret.clone(),
            start_url: start_url.as_ref().to_owned(),
        };
        let response = self
            .http
            .post("/device_authorization")?
            .json(&body)
            .send()?;
        let authorization: KiroDeviceAuthorization = HttpClient::read_json(response)?;
        if authorization.browser_verification_url().trim().is_empty() {
            return Err(KiroAuthSupportError::InvalidResponse(
                "device authorization response missing verification URI".to_owned(),
            ));
        }
        if authorization.device_code.trim().is_empty() {
            return Err(KiroAuthSupportError::InvalidResponse(
                "device authorization response missing deviceCode".to_owned(),
            ));
        }
        Ok(authorization)
    }

    /// 为指定登录类型创建完整设备流程。
    pub fn begin_device_flow(
        &self,
        login_type: KiroLoginType,
        enterprise_start_url: Option<&str>,
    ) -> KiroAuthSupportResult<KiroDeviceFlow> {
        let start_url = login_type.resolve_start_url(enterprise_start_url);
        let client = self.register_client(&start_url)?;
        let authorization = self.start_device_authorization(&client, &start_url)?;
        let provider_interval = authorization.interval.unwrap_or_default();
        let poll_interval = self
            .config
            .poll_interval
            .max(Duration::from_secs(provider_interval));
        Ok(KiroDeviceFlow {
            uuid: Uuid::new_v4().to_string(),
            login_type,
            start_url,
            client,
            authorization,
            poll_interval,
        })
    }

    /// 针对给定流程轮询一次 `/token`。
    pub fn poll_token_once(
        &self,
        flow: &KiroDeviceFlow,
        current_interval: Duration,
    ) -> KiroAuthSupportResult<KiroTokenPoll> {
        let body = TokenRequest {
            client_id: flow.client.client_id.clone(),
            client_secret: flow.client.client_secret.clone(),
            device_code: flow.authorization.device_code.clone(),
            grant_type: DEVICE_CODE_GRANT_TYPE.to_owned(),
        };
        let response = self.http.post("/token")?.json(&body).send()?;
        let response: KiroTokenResponse = HttpClient::read_json(response)?;
        Ok(map_token_response(response, current_interval))
    }

    /// 持续轮询，直到 provider 返回成功、过期、错误或本地超时。
    pub fn poll_until_terminal(
        &self,
        flow: &KiroDeviceFlow,
    ) -> KiroAuthSupportResult<KiroTokenPoll> {
        let started = Instant::now();
        let mut interval = flow.poll_interval;

        while started.elapsed() < self.config.poll_timeout {
            match self.poll_token_once(flow, interval)? {
                KiroTokenPoll::Pending => {}
                KiroTokenPoll::SlowDown { next_interval } => {
                    interval = next_interval;
                }
                terminal => return Ok(terminal),
            }
            thread::sleep(interval);
        }

        Ok(KiroTokenPoll::Expired {
            message: "poll timeout".to_owned(),
        })
    }
}

/// [`KiroDeviceFlowManager`] 使用的非阻塞会话状态。
#[apply(plain_partial_eq_display)]
pub enum KiroDeviceFlowSessionStatus {
    /// 轮询正在进行。
    #[display("pending")]
    Pending,
    /// token 交换已完成。
    #[display("success")]
    Success(KiroTokenResponse),
    /// 设备代码已过期或本地超时已到达。
    #[display("expired")]
    Expired(String),
    /// provider 或传输层错误。
    #[display("error")]
    Error(String),
    /// 轮询已在本地取消。
    #[display("canceled")]
    Canceled,
}

/// [`KiroDeviceFlowManager::get_status`] 返回的快照。
#[apply(plain_partial_eq)]
pub struct KiroDeviceFlowSessionSnapshot {
    /// 流程的本地 UUID。
    pub uuid: String,
    /// 当前状态。
    pub status: KiroDeviceFlowSessionStatus,
    /// 状态等待中时用户应打开的 URL。
    pub verification_url: String,
    /// provider 展示的用户代码。
    pub user_code: String,
    /// 当前流程选择的登录类型。
    pub login_type: KiroLoginType,
}

#[apply(plain_debug)]
pub struct KiroDeviceFlowSession {
    flow: KiroDeviceFlow,
    status: KiroDeviceFlowSessionStatus,
    cancel_requested: bool,
    handle: Option<JoinHandle<()>>,
}

impl KiroDeviceFlowSession {
    fn snapshot(&self) -> KiroDeviceFlowSessionSnapshot {
        KiroDeviceFlowSessionSnapshot {
            uuid: self.flow.uuid.clone(),
            status: self.status.clone(),
            verification_url: self.flow.verification_url().to_owned(),
            user_code: self.flow.user_code().to_owned(),
            login_type: self.flow.login_type,
        }
    }
}

/// 复刻 Python 模块 `uuid -> status` 工作流的会话管理器。
#[apply(plain_clone_debug)]
pub struct KiroDeviceFlowManager {
    client: KiroDeviceFlowClient,
    sessions: Arc<Mutex<HashMap<String, KiroDeviceFlowSession>>>,
}

impl KiroDeviceFlowManager {
    /// 基于阻塞式设备流程客户端创建管理器。
    #[must_use]
    pub fn new(client: KiroDeviceFlowClient) -> Self {
        Self {
            client,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 启动设备流程并返回初始状态快照。
    pub fn create_deviceflow_auth(
        &self,
        login_type: KiroLoginType,
        enterprise_start_url: Option<&str>,
    ) -> KiroAuthSupportResult<KiroDeviceFlowSessionSnapshot> {
        let flow = self
            .client
            .begin_device_flow(login_type, enterprise_start_url)?;
        let uuid = flow.uuid.clone();

        {
            let mut guard = self.sessions.lock().map_err(lock_error)?;
            guard.insert(
                uuid.clone(),
                KiroDeviceFlowSession {
                    flow: flow.clone(),
                    status: KiroDeviceFlowSessionStatus::Pending,
                    cancel_requested: false,
                    handle: None,
                },
            );
        }

        let client = self.client.clone();
        let sessions = Arc::clone(&self.sessions);
        let thread_uuid = uuid.clone();
        let handle = thread::spawn(move || {
            let terminal = match client.poll_until_terminal(&flow) {
                Ok(KiroTokenPoll::Success(response)) => {
                    KiroDeviceFlowSessionStatus::Success(response)
                }
                Ok(KiroTokenPoll::Expired { message }) => {
                    KiroDeviceFlowSessionStatus::Expired(message)
                }
                Ok(KiroTokenPoll::Error { message, .. }) => {
                    KiroDeviceFlowSessionStatus::Error(message)
                }
                Ok(KiroTokenPoll::Pending | KiroTokenPoll::SlowDown { .. }) => {
                    KiroDeviceFlowSessionStatus::Expired(
                        "poll finished without terminal status".to_owned(),
                    )
                }
                Err(error) => KiroDeviceFlowSessionStatus::Error(error.to_string()),
            };
            let Ok(mut guard) = sessions.lock() else {
                return;
            };
            let Some(session) = guard.get_mut(&thread_uuid) else {
                return;
            };
            if session.cancel_requested {
                return;
            }
            session.status = terminal;
        });

        let mut guard = self.sessions.lock().map_err(lock_error)?;
        let session = guard.get_mut(&uuid).ok_or_else(|| {
            KiroAuthSupportError::InvalidResponse("device flow session disappeared".to_owned())
        })?;
        session.handle = Some(handle);
        Ok(session.snapshot())
    }

    /// 按流程 UUID 获取当前状态快照。
    pub fn get_status(
        &self,
        uuid: impl AsRef<str>,
    ) -> KiroAuthSupportResult<Option<KiroDeviceFlowSessionSnapshot>> {
        let guard = self.sessions.lock().map_err(lock_error)?;
        Ok(guard
            .get(uuid.as_ref())
            .map(KiroDeviceFlowSession::snapshot))
    }

    /// 在本地取消等待中的会话。
    pub fn cancel(&self, uuid: impl AsRef<str>) -> KiroAuthSupportResult<bool> {
        let mut guard = self.sessions.lock().map_err(lock_error)?;
        let Some(session) = guard.get_mut(uuid.as_ref()) else {
            return Ok(false);
        };
        session.cancel_requested = true;
        if matches!(session.status, KiroDeviceFlowSessionStatus::Pending) {
            session.status = KiroDeviceFlowSessionStatus::Canceled;
        }
        Ok(true)
    }

    /// 从管理器中移除会话。
    pub fn cleanup(&self, uuid: impl AsRef<str>) -> KiroAuthSupportResult<bool> {
        let mut guard = self.sessions.lock().map_err(lock_error)?;
        Ok(guard.remove(uuid.as_ref()).is_some())
    }
}

#[apply(serialize_camel_eq)]
struct RegisterClientRequest {
    client_name: String,
    client_type: String,
    issuer_url: String,
    scopes: Vec<String>,
}

#[apply(serialize_camel_eq)]
struct DeviceAuthorizationRequest {
    client_id: String,
    client_secret: String,
    start_url: String,
}

#[apply(serialize_camel_eq)]
struct TokenRequest {
    client_id: String,
    client_secret: String,
    device_code: String,
    grant_type: String,
}

fn map_token_response(response: KiroTokenResponse, current_interval: Duration) -> KiroTokenPoll {
    if response.is_success() {
        return KiroTokenPoll::Success(response);
    }

    match response.error.as_deref().unwrap_or_default() {
        "authorization_pending" => KiroTokenPoll::Pending,
        "slow_down" => KiroTokenPoll::SlowDown {
            next_interval: (current_interval + Duration::from_secs(2)).min(Duration::from_secs(10)),
        },
        "expired_token" => KiroTokenPoll::Expired {
            message: response
                .error_description
                .clone()
                .unwrap_or_else(|| "device code expired".to_owned()),
        },
        "" => KiroTokenPoll::Error {
            message: "token response did not contain accessToken or OAuth error".to_owned(),
            response,
        },
        _ => KiroTokenPoll::Error {
            message: response
                .error_description
                .clone()
                .or_else(|| response.error.clone())
                .unwrap_or_else(|| "unknown token error".to_owned()),
            response,
        },
    }
}

fn lock_error<T>(_error: T) -> KiroAuthSupportError {
    KiroAuthSupportError::InvalidConfig("device flow session lock poisoned".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{KiroDeviceFlowClient, KiroLoginType, KiroTokenPoll, map_token_response};
    use crate::KiroOidcConfig;
    use std::time::Duration;

    #[test]
    fn enterprise_start_url_requires_https() {
        assert_eq!(
            KiroLoginType::Enterprise.resolve_start_url(Some("http://bad")),
            "https://d-906600eb6f.awsapps.com/start"
        );
        assert_eq!(
            KiroLoginType::Enterprise.resolve_start_url(Some("https://example.awsapps.com/start/")),
            "https://example.awsapps.com/start"
        );
    }

    #[test]
    fn slow_down_increases_poll_interval() {
        let poll = map_token_response(
            super::KiroTokenResponse {
                error: Some("slow_down".to_owned()),
                ..super::KiroTokenResponse::default()
            },
            Duration::from_secs(2),
        );

        assert_eq!(
            poll,
            KiroTokenPoll::SlowDown {
                next_interval: Duration::from_secs(4)
            }
        );
    }

    #[test]
    fn client_builds_from_default_config() {
        let client = KiroDeviceFlowClient::new(KiroOidcConfig::default());

        assert!(client.is_ok());
    }
}
