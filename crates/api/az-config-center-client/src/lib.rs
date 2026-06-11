#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::fmt;

use az_config_center_contract::{
    ApiResponse, ConfigItem, DeleteRequest, DeleteResult, LoginPayload, LoginRequest,
    StatusPayload, ToggleRequest, UpsertRequest,
};
use reqwest::blocking::{Client, Response};
use reqwest::{Method, StatusCode, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

const VALUE_TYPE_TEXT: &str = "text";
const VALUE_TYPE_JSON: &str = "json";
const VALUE_TYPE_NUMBER: &str = "number";
const VALUE_TYPE_BOOLEAN: &str = "boolean";
const VALUE_TYPE_SECRET: &str = "secret";

/// 配置中心 Rust 客户端的统一结果类型。
pub type ConfigCenterResult<T> = Result<T, ConfigCenterError>;

/// 配置中心客户端可能返回的错误。
#[derive(Debug, Error)]
pub enum ConfigCenterError {
    /// 基础 URL 无法被解析为 HTTP URL。
    #[error("配置中心基础 URL 无效：{0}")]
    InvalidBaseUrl(String),
    /// 调用需要登录态，但当前客户端没有 token。
    #[error("配置中心客户端尚未登录")]
    MissingToken,
    /// 调用需要命名空间，但当前客户端尚未选择命名空间。
    #[error("配置中心客户端尚未选择命名空间")]
    MissingNamespace,
    /// 调用方传入了空白配置键。
    #[error("配置键不能为空")]
    BlankKey,
    /// HTTP 传输层失败。
    #[error("请求 {url} 失败：{source}")]
    Transport {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    /// 服务端返回了非 2xx 状态码。
    #[error("请求 {url} 返回 {status}：{body}")]
    Http {
        url: String,
        status: StatusCode,
        body: String,
    },
    /// 服务端业务响应标记为失败。
    #[error("配置中心 API 返回失败：{0}")]
    Api(String),
    /// 服务端成功响应缺少必需数据。
    #[error("配置中心 API 成功响应缺少 data 字段")]
    EmptyData,
    /// 配置值无法按调用方期待的类型解码。
    #[error("配置 {key} 的值无法解析为 {target_type}：{source}")]
    DecodeValue {
        key: String,
        target_type: &'static str,
        #[source]
        source: serde_json::Error,
    },
    /// 配置值不是严格的布尔字符串。
    #[error("配置 {key} 的值不是严格布尔值")]
    InvalidBoolean { key: String },
    /// 配置值不是有效整数。
    #[error("配置 {key} 的值不是有效整数：{source}")]
    InvalidInteger {
        key: String,
        #[source]
        source: std::num::ParseIntError,
    },
    /// 配置值不是有效浮点数。
    #[error("配置 {key} 的值不是有效浮点数：{source}")]
    InvalidFloat {
        key: String,
        #[source]
        source: std::num::ParseFloatError,
    },
}

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
    /// 当 `base_url` 不是合法 URL 时返回 [`ConfigCenterError::InvalidBaseUrl`]。
    pub fn new(base_url: impl AsRef<str>) -> ConfigCenterResult<Self> {
        Self::with_http_client(base_url, Client::new())
    }

    /// 使用调用方提供的 HTTP client 创建配置中心客户端。
    ///
    /// # Errors
    ///
    /// 当 `base_url` 不是合法 URL 时返回 [`ConfigCenterError::InvalidBaseUrl`]。
    pub fn with_http_client(base_url: impl AsRef<str>, http: Client) -> ConfigCenterResult<Self> {
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
    pub fn login(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> ConfigCenterResult<Self> {
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
    pub fn checkout_namespace(&self, namespace: impl AsRef<str>) -> ConfigCenterResult<Self> {
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
    pub fn status(&self) -> ConfigCenterResult<StatusPayload> {
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
    ) -> ConfigCenterResult<Vec<ConfigItem>> {
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
    pub fn get_item(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<ConfigItem>> {
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
            return Err(ConfigCenterError::Api(response.message));
        }
        Ok(response.data.flatten())
    }

    /// 读取文本配置。
    ///
    /// # Errors
    ///
    /// 读取配置项失败时返回错误。
    pub fn get_text(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<String>> {
        Ok(self.get_item(key)?.map(|item| item.config_value))
    }

    /// 读取密钥配置。
    ///
    /// # Errors
    ///
    /// 读取配置项失败时返回错误。
    pub fn get_secret(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<String>> {
        self.get_text(key)
    }

    /// 读取整数配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值不是有效整数时返回错误。
    pub fn get_i64(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<i64>> {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        item.config_value
            .parse::<i64>()
            .map(Some)
            .map_err(|source| ConfigCenterError::InvalidInteger { key, source })
    }

    /// 读取浮点数配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值不是有效浮点数时返回错误。
    pub fn get_f64(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<f64>> {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        item.config_value
            .parse::<f64>()
            .map(Some)
            .map_err(|source| ConfigCenterError::InvalidFloat { key, source })
    }

    /// 读取布尔配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值不是 `true` / `false` 时返回错误。
    pub fn get_bool(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<bool>> {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        match item.config_value.as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err(ConfigCenterError::InvalidBoolean { key }),
        }
    }

    /// 读取 JSON 对象配置。
    ///
    /// # Errors
    ///
    /// 读取失败或配置值无法反序列化为目标类型时返回错误。
    pub fn get_json<T>(&self, key: impl AsRef<str>) -> ConfigCenterResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let key = normalize_key(key.as_ref())?;
        let Some(item) = self.get_item(&key)? else {
            return Ok(None);
        };
        serde_json::from_str(&item.config_value)
            .map(Some)
            .map_err(|source| ConfigCenterError::DecodeValue {
                key,
                target_type: std::any::type_name::<T>(),
                source,
            })
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
    ) -> ConfigCenterResult<ConfigItem> {
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
    ) -> ConfigCenterResult<ConfigItem> {
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
    ) -> ConfigCenterResult<ConfigItem> {
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
    ) -> ConfigCenterResult<ConfigItem> {
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
    ) -> ConfigCenterResult<ConfigItem> {
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
    ) -> ConfigCenterResult<ConfigItem>
    where
        T: Serialize,
    {
        let key = normalize_key(key.as_ref())?;
        let value =
            serde_json::to_string(value).map_err(|source| ConfigCenterError::DecodeValue {
                key: key.clone(),
                target_type: std::any::type_name::<T>(),
                source,
            })?;
        self.set_value(&key, &value, VALUE_TYPE_JSON, description.as_ref())
    }

    /// 按原始请求写入配置。
    ///
    /// # Errors
    ///
    /// 未登录、网络失败或服务端返回失败时返回错误。
    pub fn upsert(&self, request: &UpsertRequest) -> ConfigCenterResult<ConfigItem> {
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
    pub fn toggle(&self, request: &ToggleRequest) -> ConfigCenterResult<ConfigItem> {
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
    pub fn delete(&self, request: &DeleteRequest) -> ConfigCenterResult<DeleteResult> {
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
    ) -> ConfigCenterResult<ConfigItem> {
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
    ) -> ConfigCenterResult<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let response = self.request_envelope(method, path, query, body, require_auth)?;
        if !response.success {
            return Err(ConfigCenterError::Api(response.message));
        }
        response.data.ok_or(ConfigCenterError::EmptyData)
    }

    fn request_envelope<T, B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
        require_auth: bool,
    ) -> ConfigCenterResult<ApiResponse<T>>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = self.build_url(path, query);
        let mut request = self.http.request(method, url.clone());
        if require_auth {
            request = request.bearer_auth(self.require_token()?);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .map_err(|source| ConfigCenterError::Transport {
                url: url.to_string(),
                source,
            })?;
        decode_response(url, response)
    }

    fn build_url(&self, path: &str, query: &[(&str, String)]) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url.set_query(None);
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in query {
                pairs.append_pair(name, value);
            }
        }
        url
    }

    fn require_token(&self) -> ConfigCenterResult<&str> {
        self.token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
            .ok_or(ConfigCenterError::MissingToken)
    }

    fn require_namespace(&self) -> ConfigCenterResult<&str> {
        self.namespace
            .as_deref()
            .filter(|namespace| !namespace.trim().is_empty())
            .ok_or(ConfigCenterError::MissingNamespace)
    }
}

fn decode_response<T>(url: Url, response: Response) -> ConfigCenterResult<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .map_err(|source| ConfigCenterError::Transport {
                url: url.to_string(),
                source,
            })?;
        return Err(ConfigCenterError::Http {
            url: url.to_string(),
            status,
            body,
        });
    }
    let bytes = response
        .bytes()
        .map_err(|source| ConfigCenterError::Transport {
            url: url.to_string(),
            source,
        })?;
    serde_json::from_slice(bytes.as_ref()).map_err(|source| ConfigCenterError::DecodeValue {
        key: url.to_string(),
        target_type: std::any::type_name::<T>(),
        source,
    })
}

fn normalize_base_url(value: &str) -> ConfigCenterResult<Url> {
    let normalized = value.trim().trim_end_matches('/');
    Url::parse(normalized).map_err(|_| ConfigCenterError::InvalidBaseUrl(value.to_owned()))
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_required_namespace(value: &str) -> ConfigCenterResult<String> {
    let namespace = value.trim();
    if namespace.is_empty() {
        return Err(ConfigCenterError::MissingNamespace);
    }
    Ok(namespace.to_owned())
}

fn normalize_key(value: &str) -> ConfigCenterResult<String> {
    let key = value.trim();
    if key.is_empty() {
        return Err(ConfigCenterError::BlankKey);
    }
    Ok(key.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize)]
    struct RedisConfig {
        host: String,
        port: u16,
    }

    #[test]
    fn new_rejects_invalid_base_url() {
        let result = ConfigCenterClient::new("not a url");

        // 关键断言：基础 URL 在客户端创建阶段就被校验，避免请求时才暴露模糊错误。
        assert!(matches!(result, Err(ConfigCenterError::InvalidBaseUrl(_))));
    }

    #[test]
    fn checkout_namespace_requires_login() {
        let client = match ConfigCenterClient::new("http://127.0.0.1:8080") {
            Ok(client) => client,
            Err(error) => panic!("创建客户端失败：{error}"),
        };
        let result = client.checkout_namespace("dev");

        // 关键断言：命名空间绑定必须发生在登录之后，和 Kotlin SDK 链式语义一致。
        assert!(matches!(result, Err(ConfigCenterError::MissingToken)));
    }

    #[test]
    fn build_url_encodes_query_pairs() {
        let client = match ConfigCenterClient::new("http://127.0.0.1:8080/") {
            Ok(client) => client,
            Err(error) => panic!("创建客户端失败：{error}"),
        };
        let url = client.build_url(
            "/api/v1/config/value",
            &[
                ("namespace", "cmp aio.dev".to_owned()),
                ("key", "redis.host".to_owned()),
            ],
        );

        // 关键断言：查询参数交给 URL 类型编码，配置键和命名空间可安全包含空格等字符。
        assert_eq!(
            url.as_str(),
            "http://127.0.0.1:8080/api/v1/config/value?namespace=cmp+aio.dev&key=redis.host"
        );
    }

    #[test]
    fn json_upsert_request_uses_json_value_type() {
        let client = match ConfigCenterClient::new("http://127.0.0.1:8080") {
            Ok(client) => client,
            Err(error) => panic!("创建客户端失败：{error}"),
        };
        let value = RedisConfig {
            host: "127.0.0.1".to_owned(),
            port: 6379,
        };
        let encoded = match serde_json::to_string(&value) {
            Ok(encoded) => encoded,
            Err(error) => panic!("序列化测试配置失败：{error}"),
        };
        let request = UpsertRequest {
            namespace: "dev".to_owned(),
            key: "redis".to_owned(),
            value: encoded,
            value_type: VALUE_TYPE_JSON.to_owned(),
            description: "Redis 配置".to_owned(),
            enabled: true,
            updated_by: client.username().unwrap_or_default().to_owned(),
        };

        // 关键断言：Rust client 的结构化配置写入和 Kotlin SDK 一样使用 json 类型。
        assert_eq!(request.value_type, "json");
    }
}
