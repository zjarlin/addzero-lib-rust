use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Public settings returned by `/open_api/settings`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

/// Address credential returned by `/api/new_address` and `/api/address_login`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AddressCredential {
    pub jwt: String,
    pub address: String,
    #[serde(default)]
    pub password: Option<String>,
    pub address_id: u64,
}

/// Address settings returned by `/api/settings`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AddressSettings {
    pub address: String,
    #[serde(default)]
    pub send_balance: i64,
}

/// Paginated response shape used by the Cloudflare Temp Email worker.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ListResponse<T> {
    #[serde(default)]
    pub results: Vec<T>,
    #[serde(default)]
    pub count: u64,
}

/// Raw mailbox row from `/api/mails`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

/// Parsed mailbox row from `/api/parsed_mails`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SuccessResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub status: Option<String>,
}

/// Login request for address-password deployments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
