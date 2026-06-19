use serde_json::Value;

/// 当前支持的具体临时邮箱 provider。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TempMailProviderKind {
    /// Self-hosted Cloudflare Worker from `dreamhunter2333/cloudflare_temp_email`.
    Cloudflare,
    /// Hosted mail.tm-compatible API.
    MailTm,
    /// Hosted Emailnator webmail API.
    Emailnator,
}

impl TempMailProviderKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 列表端点使用的分页参数；取值会归一化到常见 provider 限制。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    /// 单页数量，构造时会限制到 `1..=100`。
    pub limit: usize,
    /// 从第几条结果开始读取。
    pub offset: usize,
}

impl PageRequest {
    /// 创建分页请求，并将 `limit` 限制到 `1..=100`。
    pub const fn new(limit: usize, offset: usize) -> Self {
        Self {
            limit: clamp_limit(limit),
            offset,
        }
    }

    pub(crate) fn to_query(self) -> [(&'static str, String); 2] {
        [
            ("limit", self.limit.to_string()),
            ("offset", self.offset.to_string()),
        ]
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        PageRequest {
    limit: 20,
    offset: 0,
}
    }
}

/// provider 中立的邮箱创建请求。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateMailboxRequest {
    /// Preferred local part. Providers may sanitize it or generate a random one.
    pub name: Option<String>,
    /// Preferred domain. Providers may select a default when omitted.
    pub domain: Option<String>,
    /// Optional password length for providers that create password-based accounts.
    pub password_length: usize,
    /// Optional Cloudflare Turnstile token for deployments that require it.
    pub cf_token: Option<String>,
    /// Whether random subdomain creation should be requested when supported.
    pub enable_random_subdomain: bool,
}

impl CreateMailboxRequest {
    /// Creates a request with a preferred local part and provider-selected domain.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            domain: None,
            password_length: 16,
            cf_token: None,
            enable_random_subdomain: false,
        }
    }

    /// Creates a request with a preferred local part and domain.
    pub fn new(name: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            domain: Some(domain.into()),
            password_length: 16,
            cf_token: None,
            enable_random_subdomain: false,
        }
    }

    /// Requests a provider-generated mailbox.
    pub const fn random() -> Self {
        Self {
            name: None,
            domain: None,
            password_length: 16,
            cf_token: None,
            enable_random_subdomain: false,
        }
    }

    /// Sets password length for providers that need a password.
    #[must_use]
    pub const fn password_length(mut self, value: usize) -> Self {
        self.password_length = value;
        self
    }

    /// Sets the Cloudflare Turnstile token when required.
    #[must_use]
    pub fn cf_token(mut self, value: impl Into<String>) -> Self {
        self.cf_token = Some(value.into());
        self
    }

    /// Requests random subdomain creation on providers that support it.
    #[must_use]
    pub const fn enable_random_subdomain(mut self, value: bool) -> Self {
        self.enable_random_subdomain = value;
        self
    }
}

/// `/open_api/settings` 返回的公开设置。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempMailSettings {
    /// 站点标题。
    #[serde(default)]
    pub title: String,
    /// 公开公告。
    #[serde(default)]
    pub announcement: String,
    /// 地址名前缀规则。
    #[serde(default)]
    pub prefix: String,
    /// 地址名校验正则。
    #[serde(default)]
    pub address_regex: String,
    /// 地址名最小长度。
    #[serde(default)]
    pub min_address_len: u32,
    /// 地址名最大长度。
    #[serde(default)]
    pub max_address_len: u32,
    /// 默认可用域名列表。
    #[serde(default)]
    pub default_domains: Vec<String>,
    /// 全部可用域名列表。
    #[serde(default)]
    pub domains: Vec<String>,
    /// 支持随机子域名的域名列表。
    #[serde(default)]
    pub random_subdomain_domains: Vec<String>,
    /// 是否需要部署级认证。
    #[serde(default)]
    pub need_auth: bool,
    /// 管理员联系方式。
    #[serde(default)]
    pub admin_contact: String,
    /// 是否允许用户创建邮箱。
    #[serde(default)]
    pub enable_user_create_email: bool,
    /// 是否禁用匿名用户创建邮箱。
    #[serde(default)]
    pub disable_anonymous_user_create_email: bool,
    /// 是否禁用自定义地址名。
    #[serde(default)]
    pub disable_custom_address_name: bool,
    /// 是否允许用户删除邮箱。
    #[serde(default)]
    pub enable_user_delete_email: bool,
    /// 是否启用自动回复。
    #[serde(default)]
    pub enable_auto_reply: bool,
    /// 是否启用 webhook。
    #[serde(default)]
    pub enable_webhook: bool,
    /// 是否启用 S3 存储。
    #[serde(default)]
    pub is_s3_enabled: bool,
    /// 是否启用发信能力。
    #[serde(default)]
    pub enable_send_mail: bool,
    /// worker 版本号。
    #[serde(default)]
    pub version: String,
    /// 是否启用地址密码登录。
    #[serde(default)]
    pub enable_address_password: bool,
    /// 是否启用全局 Turnstile 校验。
    #[serde(default)]
    pub enable_global_turnstile_check: bool,
    /// 新版部署返回但本 crate 尚未建模的字段。
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// `/api/new_address` 请求体。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAddressRequest {
    /// 首选邮箱本地部分。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 首选邮箱域名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Cloudflare Turnstile token。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cf_token: Option<String>,
    /// 是否请求随机子域名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_random_subdomain: Option<bool>,
}

impl NewAddressRequest {
    /// Creates a request with an optional local-part name and domain.
    pub fn new(name: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            domain: Some(domain.into()),
            cf_token: None,
            enable_random_subdomain: None,
        }
    }

    /// Requests a server-generated random local part and default domain.
    pub const fn random() -> Self {
        Self {
            name: None,
            domain: None,
            cf_token: None,
            enable_random_subdomain: None,
        }
    }

    /// Sets the Cloudflare Turnstile token when the deployment requires it.
    #[must_use]
    pub fn cf_token(mut self, value: impl Into<String>) -> Self {
        self.cf_token = Some(value.into());
        self
    }

    /// Enables random subdomain creation on deployments that allow it.
    #[must_use]
    pub const fn enable_random_subdomain(mut self, value: bool) -> Self {
        self.enable_random_subdomain = Some(value);
        self
    }
}

impl From<&CreateMailboxRequest> for NewAddressRequest {
    fn from(value: &CreateMailboxRequest) -> Self {
        match value {
            value => NewAddressRequest {
        name: value.name.clone(),
        domain: value.domain.clone(),
        cf_token: value.cf_token.clone(),
        enable_random_subdomain: Some(value.enable_random_subdomain),
    }
        }
    }
}

/// `/api/new_address` 和 `/api/address_login` 返回的地址凭据。
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct AddressCredential {
    /// 后续访问该邮箱所需的 JWT。
    pub jwt: String,
    /// 完整邮箱地址。
    pub address: String,
    /// worker 返回的可选地址密码。
    #[serde(default)]
    pub password: Option<String>,
    /// worker 内部地址 ID。
    pub address_id: u64,
}

/// provider 中立的邮箱凭据。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TempMailMailbox {
    /// 创建该邮箱的 provider。
    pub provider: TempMailProviderKind,
    /// 完整邮箱地址。
    pub address: String,
    /// 后续请求使用的 provider 凭据，例如 JWT 或 bearer token。
    pub credential: String,
    /// provider 账号 ID。
    pub account_id: Option<String>,
    /// provider 创建或调用方配置的邮箱密码。
    pub password: Option<String>,
}

impl TempMailMailbox {
    /// 根据 Cloudflare 地址凭据创建 provider 中立邮箱。
    pub fn from_cloudflare(value: AddressCredential) -> Self {
        Self {
            provider: TempMailProviderKind::Cloudflare,
            address: value.address,
            credential: value.jwt,
            account_id: Some(value.address_id.to_string()),
            password: value.password,
        }
    }

    /// 为基于 token 的 provider 创建 provider 中立邮箱。
    pub fn token(
        provider: TempMailProviderKind,
        address: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            address: address.into(),
            credential: token.into(),
            account_id: None,
            password: None,
        }
    }
}

/// `/api/settings` 返回的地址设置。
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct AddressSettings {
    /// 当前地址。
    pub address: String,
    /// 当前地址剩余发信额度。
    #[serde(default)]
    pub send_balance: i64,
}

/// Cloudflare Temp Email worker 使用的分页响应结构。
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(bound(deserialize = "T: ::serde::Deserialize<'de>"))]
pub struct ListResponse<T> {
    /// 当前页结果。
    #[serde(default)]
    pub results: Vec<T>,
    /// provider 报告的总数。
    #[serde(default)]
    pub count: u64,
}

/// `/api/mails` 返回的原始邮箱记录。
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct MailRow {
    /// worker 内部邮件 ID。
    pub id: u64,
    /// 原始邮件 message-id。
    #[serde(default)]
    pub message_id: Option<String>,
    /// 发件人地址或原始来源字段。
    #[serde(default)]
    pub source: Option<String>,
    /// 收件地址。
    #[serde(default)]
    pub address: Option<String>,
    /// 原始邮件内容。
    #[serde(default)]
    pub raw: Option<String>,
    /// worker 返回的元数据字符串。
    #[serde(default)]
    pub metadata: Option<String>,
    /// 创建时间。
    #[serde(default)]
    pub created_at: Option<String>,
    /// 新版部署返回但本 crate 尚未建模的字段。
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// provider 中立的邮件摘要。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TempMailMessageSummary {
    /// provider 消息 ID。
    pub id: String,
    /// 发件人地址。
    pub from_address: String,
    /// 发件人显示名。
    pub from_name: String,
    /// 邮件主题。
    pub subject: String,
    /// 邮件摘要或预览。
    pub intro: String,
    /// 创建时间。
    pub created_at: String,
}

impl From<MailRow> for TempMailMessageSummary {
    fn from(value: MailRow) -> Self {
        match value {
            value => TempMailMessageSummary {
        id: value.id.to_string(),
        from_address: value.source.unwrap_or_default(),
        from_name: String::new(),
        subject: extract_raw_subject(value.raw.as_deref()).unwrap_or_default(),
        intro: String::new(),
        created_at: value.created_at.unwrap_or_default(),
    }
        }
    }
}

/// provider 中立的邮件详情。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TempMailMessageDetail {
    /// provider 消息 ID。
    pub id: String,
    /// 发件人地址。
    pub from_address: String,
    /// 发件人显示名。
    pub from_name: String,
    /// 收件人列表。
    pub to: Vec<TempMailRecipient>,
    /// 邮件主题。
    pub subject: String,
    /// 纯文本正文。
    pub text: String,
    /// HTML 正文。
    pub html: String,
    /// 原始邮件内容。
    pub raw: String,
    /// 创建时间。
    pub created_at: String,
}

impl From<MailRow> for TempMailMessageDetail {
    fn from(value: MailRow) -> Self {
        match value {
            value => {
        let subject = extract_raw_subject(value.raw.as_deref()).unwrap_or_default();
        TempMailMessageDetail {
            id: value.id.to_string(),
            from_address: value.source.unwrap_or_default(),
            from_name: String::new(),
            to: value
                .address
                .map(|address| {
                    vec![TempMailRecipient {
                        address,
                        name: String::new(),
                    }]
                })
                .unwrap_or_default(),
            subject,
            text: String::new(),
            html: String::new(),
            raw: value.raw.unwrap_or_default(),
            created_at: value.created_at.unwrap_or_default(),
        }
    }
        }
    }
}

/// provider 中立的邮件收件人。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TempMailRecipient {
    /// 收件人地址。
    pub address: String,
    /// 收件人显示名。
    pub name: String,
}

/// `/api/parsed_mails` 返回的已解析邮箱记录。
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct ParsedMailRow {
    /// worker 内部邮件 ID。
    pub id: u64,
    /// 原始邮件 message-id。
    #[serde(default)]
    pub message_id: Option<String>,
    /// 发件人地址或原始来源字段。
    #[serde(default)]
    pub source: Option<String>,
    /// 收件地址。
    #[serde(default)]
    pub address: Option<String>,
    /// 发件人显示字段。
    #[serde(default)]
    pub sender: String,
    /// 邮件主题。
    #[serde(default)]
    pub subject: String,
    /// 纯文本正文。
    #[serde(default)]
    pub text: String,
    /// HTML 正文。
    #[serde(default)]
    pub html: String,
    /// 附件元数据列表。
    #[serde(default)]
    pub attachments: Vec<ParsedMailAttachment>,
    /// 创建时间。
    #[serde(default)]
    pub created_at: Option<String>,
    /// 新版部署返回但本 crate 尚未建模的字段。
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// `/api/parsed_mails` 返回的已解析附件元数据。
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMailAttachment {
    /// 文件名。
    #[serde(default)]
    pub filename: String,
    /// MIME 类型。
    #[serde(default)]
    pub mime_type: String,
    /// Content-Disposition 值。
    #[serde(default)]
    pub disposition: String,
    /// 附件大小。
    #[serde(default)]
    pub size: u64,
}

/// `/api/send_mail` 请求体。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SendMailRequest {
    /// 发件人显示名。
    pub from_name: String,
    /// 收件人邮箱。
    pub to_mail: String,
    /// 收件人显示名。
    pub to_name: String,
    /// 邮件主题。
    pub subject: String,
    /// 邮件正文。
    pub content: String,
    /// `true` 表示正文按 HTML 发送。
    pub is_html: bool,
}

impl SendMailRequest {
    /// 创建最小纯文本发信请求。
    pub fn text(
        to_mail: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            from_name: String::new(),
            to_mail: to_mail.into(),
            to_name: String::new(),
            subject: subject.into(),
            content: content.into(),
            is_html: false,
        }
    }

    /// 创建最小 HTML 发信请求。
    pub fn html(
        to_mail: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            from_name: String::new(),
            to_mail: to_mail.into(),
            to_name: String::new(),
            subject: subject.into(),
            content: content.into(),
            is_html: true,
        }
    }

    /// 设置发件人显示名。
    #[must_use]
    pub fn from_name(mut self, value: impl Into<String>) -> Self {
        self.from_name = value.into();
        self
    }

    /// 设置收件人显示名。
    #[must_use]
    pub fn to_name(mut self, value: impl Into<String>) -> Self {
        self.to_name = value.into();
        self
    }
}

/// 写操作端点返回的通用成功响应。
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct SuccessResponse {
    /// 操作是否成功。
    #[serde(default)]
    pub success: bool,
    /// provider 返回的可选状态文本。
    #[serde(default)]
    pub status: Option<String>,
}

/// 启用地址密码部署的登录请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct AddressLoginRequest {
    /// 登录邮箱地址。
    pub email: String,
    /// 前端兼容格式的密码哈希。
    pub password: String,
    /// Cloudflare Turnstile token。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cf_token: Option<String>,
}

impl AddressLoginRequest {
    /// 使用前端兼容的 SHA-256 密码哈希创建登录请求。
    pub fn hashed(email: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password_hash.into(),
            cf_token: None,
        }
    }

    /// 在部署要求时添加 Cloudflare Turnstile token。
    #[must_use]
    pub fn cf_token(mut self, value: impl Into<String>) -> Self {
        self.cf_token = Some(value.into());
        self
    }
}

fn extract_raw_subject(raw: Option<&str>) -> Option<String> {
    raw.and_then(|raw| {
        raw.lines().find_map(|line| {
            line.strip_prefix("Subject:")
                .or_else(|| line.strip_prefix("subject:"))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
    })
}

const fn clamp_limit(limit: usize) -> usize {
    if limit == 0 {
        1
    } else if limit > 100 {
        100
    } else {
        limit
    }
}
