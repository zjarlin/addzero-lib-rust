use az_derive_aliases::{
    apply, deserialize_eq, deserialize_partial_eq, plain_copy_eq, plain_eq, serde_code_enum,
    serde_partial_eq, serialize_eq,
};
use serde_json::Value;

/// Supported concrete temporary email providers.
#[apply(serde_code_enum)]
pub enum TempMailProviderKind {
    /// Self-hosted Cloudflare Worker from `dreamhunter2333/cloudflare_temp_email`.
    Cloudflare,
    /// Hosted mail.tm-compatible API.
    MailTm,
    /// Hosted Emailnator webmail API.
    Emailnator,
}

/// Pagination used by list endpoints. Values are normalized to common provider limits.
#[apply(plain_copy_eq)]
pub struct PageRequest {
    pub limit: usize,
    pub offset: usize,
}

impl PageRequest {
    /// Creates a request after clamping `limit` to `1..=100`.
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
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

/// Provider-neutral request for creating a mailbox.
#[apply(plain_eq)]
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

/// Public settings returned by `/open_api/settings`.
#[apply(serde_partial_eq)]
#[serde(rename_all = "camelCase")]
pub struct TempMailSettings {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub announcement: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub address_regex: String,
    #[serde(default)]
    pub min_address_len: u32,
    #[serde(default)]
    pub max_address_len: u32,
    #[serde(default)]
    pub default_domains: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub random_subdomain_domains: Vec<String>,
    #[serde(default)]
    pub need_auth: bool,
    #[serde(default)]
    pub admin_contact: String,
    #[serde(default)]
    pub enable_user_create_email: bool,
    #[serde(default)]
    pub disable_anonymous_user_create_email: bool,
    #[serde(default)]
    pub disable_custom_address_name: bool,
    #[serde(default)]
    pub enable_user_delete_email: bool,
    #[serde(default)]
    pub enable_auto_reply: bool,
    #[serde(default)]
    pub enable_webhook: bool,
    #[serde(default)]
    pub is_s3_enabled: bool,
    #[serde(default)]
    pub enable_send_mail: bool,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub enable_address_password: bool,
    #[serde(default)]
    pub enable_global_turnstile_check: bool,
    /// Fields from newer deployments that are not yet modeled.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Request body for `/api/new_address`.
#[apply(serialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct NewAddressRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cf_token: Option<String>,
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
        Self {
            name: value.name.clone(),
            domain: value.domain.clone(),
            cf_token: value.cf_token.clone(),
            enable_random_subdomain: Some(value.enable_random_subdomain),
        }
    }
}

/// Address credential returned by `/api/new_address` and `/api/address_login`.
#[apply(deserialize_eq)]
pub struct AddressCredential {
    pub jwt: String,
    pub address: String,
    #[serde(default)]
    pub password: Option<String>,
    pub address_id: u64,
}

/// Provider-neutral mailbox credential.
#[apply(plain_eq)]
pub struct TempMailMailbox {
    pub provider: TempMailProviderKind,
    pub address: String,
    pub credential: String,
    pub account_id: Option<String>,
    pub password: Option<String>,
}

impl TempMailMailbox {
    /// Creates a provider-neutral mailbox from a Cloudflare address credential.
    pub fn from_cloudflare(value: AddressCredential) -> Self {
        Self {
            provider: TempMailProviderKind::Cloudflare,
            address: value.address,
            credential: value.jwt,
            account_id: Some(value.address_id.to_string()),
            password: value.password,
        }
    }

    /// Creates a provider-neutral mailbox for token-based providers.
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

/// Address settings returned by `/api/settings`.
#[apply(deserialize_eq)]
pub struct AddressSettings {
    pub address: String,
    #[serde(default)]
    pub send_balance: i64,
}

/// Paginated response shape used by the Cloudflare Temp Email worker.
#[apply(deserialize_eq)]
#[serde(bound(deserialize = "T: ::serde::Deserialize<'de>"))]
pub struct ListResponse<T> {
    #[serde(default)]
    pub results: Vec<T>,
    #[serde(default)]
    pub count: u64,
}

/// Raw mailbox row from `/api/mails`.
#[apply(deserialize_eq)]
pub struct MailRow {
    pub id: u64,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub metadata: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Provider-neutral message summary.
#[apply(plain_eq)]
pub struct TempMailMessageSummary {
    pub id: String,
    pub from_address: String,
    pub from_name: String,
    pub subject: String,
    pub intro: String,
    pub created_at: String,
}

impl From<MailRow> for TempMailMessageSummary {
    fn from(value: MailRow) -> Self {
        Self {
            id: value.id.to_string(),
            from_address: value.source.unwrap_or_default(),
            from_name: String::new(),
            subject: extract_raw_subject(value.raw.as_deref()).unwrap_or_default(),
            intro: String::new(),
            created_at: value.created_at.unwrap_or_default(),
        }
    }
}

/// Provider-neutral message detail.
#[apply(plain_eq)]
pub struct TempMailMessageDetail {
    pub id: String,
    pub from_address: String,
    pub from_name: String,
    pub to: Vec<TempMailRecipient>,
    pub subject: String,
    pub text: String,
    pub html: String,
    pub raw: String,
    pub created_at: String,
}

impl From<MailRow> for TempMailMessageDetail {
    fn from(value: MailRow) -> Self {
        let subject = extract_raw_subject(value.raw.as_deref()).unwrap_or_default();
        Self {
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

/// Provider-neutral email recipient.
#[apply(plain_eq)]
pub struct TempMailRecipient {
    pub address: String,
    pub name: String,
}

/// Parsed mailbox row from `/api/parsed_mails`.
#[apply(deserialize_partial_eq)]
pub struct ParsedMailRow {
    pub id: u64,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub html: String,
    #[serde(default)]
    pub attachments: Vec<ParsedMailAttachment>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Parsed attachment metadata returned by `/api/parsed_mails`.
#[apply(deserialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMailAttachment {
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub disposition: String,
    #[serde(default)]
    pub size: u64,
}

/// Request body for `/api/send_mail`.
#[apply(serialize_eq)]
pub struct SendMailRequest {
    pub from_name: String,
    pub to_mail: String,
    pub to_name: String,
    pub subject: String,
    pub content: String,
    pub is_html: bool,
}

impl SendMailRequest {
    /// Creates a minimal plain-text send-mail request.
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

    /// Creates a minimal HTML send-mail request.
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

    /// Sets the display name of the sender address.
    #[must_use]
    pub fn from_name(mut self, value: impl Into<String>) -> Self {
        self.from_name = value.into();
        self
    }

    /// Sets the display name of the recipient.
    #[must_use]
    pub fn to_name(mut self, value: impl Into<String>) -> Self {
        self.to_name = value.into();
        self
    }
}

/// Generic success response returned by mutation endpoints.
#[apply(deserialize_eq)]
pub struct SuccessResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub status: Option<String>,
}

/// Login request for address-password deployments.
#[apply(serialize_eq)]
pub struct AddressLoginRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cf_token: Option<String>,
}

impl AddressLoginRequest {
    /// Creates a login request with a frontend-compatible SHA-256 password hash.
    pub fn hashed(email: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password_hash.into(),
            cf_token: None,
        }
    }

    /// Adds a Cloudflare Turnstile token when required by the deployment.
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
