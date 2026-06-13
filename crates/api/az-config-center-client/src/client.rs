use std::fmt;

use anyhow::{Context, Result, bail};
use az_config_center_contract::{
    ApiResponse, ConfigItem, DeleteRequest, DeleteResult, LoginPayload, LoginRequest,
    StatusPayload, ToggleRequest, UpsertRequest,
};
use reqwest::blocking::{Client, Response};
use reqwest::{Method, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::url_builder::build_url;

const VALUE_TYPE_TEXT: &str = "text";
const VALUE_TYPE_JSON: &str = "json";
const VALUE_TYPE_NUMBER: &str = "number";
const VALUE_TYPE_BOOLEAN: &str = "boolean";
const VALUE_TYPE_SECRET: &str = "secret";

/// 配置中心同步客户端。
#[derive(Clone)]
pub struct ConfigCenterClient {
    base_url: Url,
    token: Option<String>,
    username: Option<String>,
    namespace: Option<String>,
    http: Client,
}

impl fmt::Debug for ConfigCenterClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigCenterClient")
            .field("base_url", &self.base_url)
            .field("token_configured", &self.token.is_some())
            .field("username", &self.username)
            .field("namespace", &self.namespace)
            .finish_non_exhaustive()
    }
}

impl ConfigCenterClient {
    /// 使用默认 `reqwest::blocking::Client` 创建配置中心客户端。
    ///
    /// # Errors
    ///
    /// 当 `base_url` 不是合法 URL 时返回错误。
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        Self::with_http_client(base_url, Client::new())
    }

    /// 使用调用方提供的 HTTP client 创建配置中心客户端。
    ///
    /// # Errors
    ///
    /// 当 `base_url` 不是合法 URL 时返回错误。
    pub fn with_http_client(base_url: impl AsRef<str>, http: Client) -> Result<Self> {
        let base_url = normalize_base_url(base_url.as_ref())?;
        Ok(Self {
            base_url,
            token: None,
            username: None,
            namespace: None,
            http,
        })
    }

    /// 返回当前客户端使用的基础 URL。
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// 返回当前登录用户名。
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// 返回当前选择的命名空间。
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// 返回是否已持有登录 token。
    pub fn is_logged_in(&self) -> bool {
        self.token.is_some()
    }

    /// 登录配置中心并返回带 token 的新客户端。
    ///
    /// # Errors
    ///
    /// 网络失败、HTTP 非成功状态、账号密码错误或响应缺少 token 时返回错误。
    pub fn login(&self, username: impl AsRef<str>, password: impl AsRef<str>) -> Result<Self> {
        let request = LoginRequest {
            username: username.as_ref().trim().to_owned(),
            password: password.as_ref().trim().to_owned(),
        };
        let payload: LoginPayload = self.request_data(
            Method::POST,
            "/api/v1/auth/login",
            &[],
            Some(&request),
            false,
        )?;
        let mut next = self.clone();
        next.token = Some(payload.token);
        next.username = Some(payload.username);
        Ok(next)
    }

    /// 选择后续配置读写使用的命名空间，并返回新客户端。
    ///
    /// # Errors
    ///
    /// 未登录或命名空间为空白时返回错误。
    pub fn checkout_namespace(&self, namespace: impl AsRef<str>) -> Result<Self> {
        self.require_token()?;
        let namespace = normalize_required_namespace(namespace.as_ref())?;
        let mut next = self.clone();
        next.namespace = Some(namespace);
        Ok(next)
    }

    /// 查询配置中心健康状态。
    ///
    /// # Errors
    ///
    /// 未登录、网络失败或服务端返回失败时返回错误。
    pub fn status(&self) -> Result<StatusPayload> {
        self.request_data(Method::GET, "/api/v1/config/status", &[], None::<&()>, true)
    }

    /// 列出配置项。
    ///
    /// # Errors
    ///
    /// 未登录、网络失败或服务端返回失败时返回错误。
    pub fn list(
        &self,
        namespace: Option<&str>,
        keyword: Option<&str>,
        include_disabled: bool,
    ) -> Result<Vec<ConfigItem>> {
        let mut query = Vec::new();
        if let Some(namespace) = normalize_optional(namespace) {
            query.push(("namespace", namespace));
        }
        if let Some(keyword) = normalize_optional(keyword) {
            query.push(("keyword", keyword));
        }
        query.push(("include_disabled", include_disabled.to_string()));
        self.request_data(
            Method::GET,
            "/api/v1/config/list",
            &query,
            None::<&()>,
            true,
        )
    }

    /// 读取当前命名空间中的启用配置项。
    ///
    /// 缺失或停用配置返回 `Ok(None)`，与 Kotlin SDK 的 nullable 语义保持一致。
    ///
    /// # Errors
    ///
    /// 未登录、未选择命名空间、配置键为空白、网络失败或服务端返回失败时返回错误。
    pub fn get_item(&self, key: impl AsRef<str>) -> Result<Option<ConfigItem>> {
        let namespace = self.require_namespace()?;
        let key = normalize_key(key.as_ref())?;
        let query = vec![("namespace", namespace.to_owned()), ("key", key)];
        let response = self.request_envelope::<Option<ConfigItem>, ()>(
            Method::GET,
            "/api/v1/config/value",
            &query,
            None,
            true,
        )?;
        if !response.success {
            bail!("配置中心 API 返回失败：{}", response.message);
        }
        Ok(response.data.flatten())
    }

    /// 读取文本配置。
    ///
    /// # Errors
    ///
    /// 读取配置项失败时返回错误。
    pub fn get_text(&self, key: impl AsRef<str>) -> Result<Option<String>> {
        Ok(self.get_item(key)?.map(|item| item.config_value))
    }

    /// 读取密钥配置。
    ///
    /// # Errors
    ///
    /// 读取配置项失败时返回错误。
    pub fn get_secret(&self, key: impl AsRef<str>) -> Result<Option<String>> {
        self.get_text(key)
    }

    /// 读取整数配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值不是有效整数时返回错误。
    pub fn get_i64(&self, key: impl AsRef<str>) -> Result<Option<i64>> {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        item.config_value
            .parse::<i64>()
            .map(Some)
            .with_context(|| format!("配置 {key} 的值不是有效整数"))
    }

    /// 读取浮点数配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值不是有效浮点数时返回错误。
    pub fn get_f64(&self, key: impl AsRef<str>) -> Result<Option<f64>> {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        item.config_value
            .parse::<f64>()
            .map(Some)
            .with_context(|| format!("配置 {key} 的值不是有效浮点数"))
    }

    /// 读取布尔配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值不是 `true` / `false` 时返回错误。
    pub fn get_bool(&self, key: impl AsRef<str>) -> Result<Option<bool>> {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        match item.config_value.as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => bail!("配置 {key} 的值不是严格布尔值"),
        }
    }

    /// 读取 JSON 对象配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值无法反序列化为目标类型时返回错误。
    pub fn get_json<T>(&self, key: impl AsRef<str>) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        serde_json::from_str(&item.config_value)
            .map(Some)
            .with_context(|| format!("配置 {key} 的值无法解析为 {}", std::any::type_name::<T>()))
    }

    /// 写入文本配置。
    ///
    /// # Errors
    ///
    /// 未登录、未选择命名空间、网络失败或服务端返回失败时返回错误。
    pub fn set_text(
        &self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> Result<ConfigItem> {
        self.set_value(
            key.as_ref(),
            value.as_ref(),
            VALUE_TYPE_TEXT,
            description.as_ref(),
        )
    }

    /// 写入密钥配置。
    ///
    /// # Errors
    ///
    /// 未登录、未选择命名空间、网络失败或服务端返回失败时返回错误。
    pub fn set_secret(
        &self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> Result<ConfigItem> {
        self.set_value(
            key.as_ref(),
            value.as_ref(),
            VALUE_TYPE_SECRET,
            description.as_ref(),
        )
    }

    /// 写入整数配置。
    ///
    /// # Errors
    ///
    /// 未登录、未选择命名空间、网络失败或服务端返回失败时返回错误。
    pub fn set_i64(
        &self,
        key: impl AsRef<str>,
        value: i64,
        description: impl AsRef<str>,
    ) -> Result<ConfigItem> {
        self.set_value(
            key.as_ref(),
            &value.to_string(),
            VALUE_TYPE_NUMBER,
            description.as_ref(),
        )
    }

    /// 写入浮点数配置。
    ///
    /// # Errors
    ///
    /// 未登录、未选择命名空间、网络失败或服务端返回失败时返回错误。
    pub fn set_f64(
        &self,
        key: impl AsRef<str>,
        value: f64,
        description: impl AsRef<str>,
    ) -> Result<ConfigItem> {
        self.set_value(
            key.as_ref(),
            &value.to_string(),
            VALUE_TYPE_NUMBER,
            description.as_ref(),
        )
    }

    /// 写入布尔配置。
    ///
    /// # Errors
    ///
    /// 未登录、未选择命名空间、网络失败或服务端返回失败时返回错误。
    pub fn set_bool(
        &self,
        key: impl AsRef<str>,
        value: bool,
        description: impl AsRef<str>,
    ) -> Result<ConfigItem> {
        self.set_value(
            key.as_ref(),
            if value { "true" } else { "false" },
            VALUE_TYPE_BOOLEAN,
            description.as_ref(),
        )
    }

    /// 写入 JSON 对象配置。
    ///
    /// # Errors
    ///
    /// 序列化失败、未登录、未选择命名空间、网络失败或服务端返回失败时返回错误。
    pub fn set_json<T>(
        &self,
        key: impl AsRef<str>,
        value: &T,
        description: impl AsRef<str>,
    ) -> Result<ConfigItem>
    where
        T: Serialize,
    {
        let key = normalize_key(key.as_ref())?;
        let value = serde_json::to_string(value)
            .with_context(|| format!("配置 {key} 的值无法解析为 {}", std::any::type_name::<T>()))?;
        self.set_value(&key, &value, VALUE_TYPE_JSON, description.as_ref())
    }

    /// 按原始请求写入配置。
    ///
    /// # Errors
    ///
    /// 未登录、网络失败或服务端返回失败时返回错误。
    pub fn upsert(&self, request: &UpsertRequest) -> Result<ConfigItem> {
        self.request_data(
            Method::PUT,
            "/api/v1/config/value",
            &[],
            Some(request),
            true,
        )
    }

    /// 修改配置启停状态。
    ///
    /// # Errors
    ///
    /// 未登录、网络失败或服务端返回失败时返回错误。
    pub fn toggle(&self, request: &ToggleRequest) -> Result<ConfigItem> {
        self.request_data(
            Method::POST,
            "/api/v1/config/toggle",
            &[],
            Some(request),
            true,
        )
    }

    /// 删除配置。
    ///
    /// # Errors
    ///
    /// 未登录、网络失败或服务端返回失败时返回错误。
    pub fn delete(&self, request: &DeleteRequest) -> Result<DeleteResult> {
        self.request_data(
            Method::POST,
            "/api/v1/config/delete",
            &[],
            Some(request),
            true,
        )
    }

    fn set_value(
        &self,
        key: &str,
        value: &str,
        value_type: &str,
        description: &str,
    ) -> Result<ConfigItem> {
        let namespace = self.require_namespace()?.to_owned();
        let key = normalize_key(key)?;
        let request = UpsertRequest {
            namespace,
            key,
            value: value.to_owned(),
            value_type: value_type.to_owned(),
            description: description.trim().to_owned(),
            enabled: true,
            updated_by: self.username.clone().unwrap_or_default(),
        };
        self.upsert(&request)
    }

    fn request_data<T, B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
        require_auth: bool,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let response = self.request_envelope(method, path, query, body, require_auth)?;
        if !response.success {
            bail!("配置中心 API 返回失败：{}", response.message);
        }
        response.data.context("配置中心 API 成功响应缺少 data 字段")
    }

    fn request_envelope<T, B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
        require_auth: bool,
    ) -> Result<ApiResponse<T>>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = build_url(&self.base_url, path, query);
        let mut request = self.http.request(method, url.clone());
        if require_auth {
            request = request.bearer_auth(self.require_token()?);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().with_context(|| format!("请求 {url} 失败"))?;
        decode_response(url, response)
    }

    fn require_token(&self) -> Result<&str> {
        self.token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
            .context("配置中心客户端尚未登录")
    }

    fn require_namespace(&self) -> Result<&str> {
        self.namespace
            .as_deref()
            .filter(|namespace| !namespace.trim().is_empty())
            .context("配置中心客户端尚未选择命名空间")
    }
}

fn decode_response<T>(url: Url, response: Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .with_context(|| format!("请求 {url} 失败"))?;
        bail!("请求 {url} 返回 {status}：{body}");
    }
    let bytes = response
        .bytes()
        .with_context(|| format!("请求 {url} 失败"))?;
    serde_json::from_slice(bytes.as_ref())
        .with_context(|| format!("配置 {} 的值无法解析为 {}", url, std::any::type_name::<T>()))
}

fn normalize_base_url(value: &str) -> Result<Url> {
    let normalized = value.trim().trim_end_matches('/');
    Url::parse(normalized).with_context(|| format!("配置中心基础 URL 无效：{value}"))
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_required_namespace(value: &str) -> Result<String> {
    let namespace = value.trim();
    if namespace.is_empty() {
        bail!("配置中心客户端尚未选择命名空间");
    }
    Ok(namespace.to_owned())
}

fn normalize_key(value: &str) -> Result<String> {
    let key = value.trim();
    if key.is_empty() {
        bail!("配置键不能为空");
    }
    Ok(key.to_owned())
}
