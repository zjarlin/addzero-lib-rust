use crate::config::ApiConfig;
use crate::http::HttpApiClient;
use crate::model::{
    CreateMailboxRequest, ListResponse, PageRequest, TempMailMailbox, TempMailMessageDetail,
    TempMailMessageSummary, TempMailProviderKind, TempMailRecipient,
};
use crate::provider::TempMailProvider;
use crate::util::{random_alpha_numeric, sanitize_local_part, trim_non_blank};
use anyhow::anyhow;
use serde_json::{Value, json};

/// 托管 `mail.tm` 临时邮箱 API 的阻塞客户端。
#[derive(Clone, Debug)]
pub struct MailTmTempMailApi {
    http: HttpApiClient,
}

impl MailTmTempMailApi {
    /// 根据显式 API 配置创建客户端。
    pub fn new(config: ApiConfig) -> anyhow::Result<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    /// 列出可用域名。
    pub fn get_domains(&self) -> anyhow::Result<Vec<MailTmDomain>> {
        let response = self.http.get("/domains")?.send()?;
        let response: HydraCollection<MailTmDomain> = HttpApiClient::read_json(response)?;
        Ok(response.items)
    }

    /// 使用第一个可用域名创建账号和登录 token。
    pub fn create_mailbox_and_login(
        &self,
        prefix: impl AsRef<str>,
        password_length: usize,
    ) -> anyhow::Result<TempMailMailbox> {
        let domains = self
            .get_domains()?
            .into_iter()
            .filter(|domain| domain.is_active)
            .collect::<Vec<_>>();

        let chosen_domain = domains
            .first()
            .map(|domain| domain.domain.clone())
            .ok_or_else(|| anyhow!("invalid response: no active mail.tm domains available"))?;

        let local_part = format!(
            "{}{}",
            sanitize_local_part(prefix.as_ref()),
            random_alpha_numeric(8)
        );
        let address = format!("{local_part}@{chosen_domain}");
        let password = random_alpha_numeric(password_length.max(8));
        let account_id = self.create_account(&address, &password)?;
        let token = self.create_token(&address, &password)?;

        Ok(TempMailMailbox {
            provider: TempMailProviderKind::MailTm,
            address,
            credential: token,
            account_id: Some(account_id),
            password: Some(password),
        })
    }

    /// 创建 mail.tm 账号并返回账号 ID。
    pub fn create_account(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let response = self
            .http
            .post("/accounts")?
            .json(&json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }))
            .send()?;
        let response: MailTmAccountResponse = HttpApiClient::read_json(response)?;

        trim_non_blank(Some(response.id.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                anyhow!(
                    "invalid response: create account failed: id missing for address={}",
                    address.as_ref().trim()
                )
            })
    }

    /// 为账号创建 mail.tm bearer token。
    pub fn create_token(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let response = self
            .http
            .post("/token")?
            .json(&json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }))
            .send()?;
        let response: MailTmTokenResponse = HttpApiClient::read_json(response)?;

        trim_non_blank(Some(response.token.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                anyhow!(
                    "invalid response: create token failed: token missing for address={}",
                    address.as_ref().trim()
                )
            })
    }

    /// 使用 mail.tm bearer token 列出邮件。
    pub fn list_messages_by_token(
        &self,
        token: impl AsRef<str>,
        page: PageRequest,
    ) -> anyhow::Result<ListResponse<TempMailMessageSummary>> {
        let query = [("page", mail_tm_page(page).to_string())];
        let response = HttpApiClient::with_bearer_auth(
            self.http.get_with_query("/messages", &query)?,
            Some(token.as_ref()),
        )
        .send()?;
        let response: HydraCollection<MailTmMessageSummaryRaw> =
            HttpApiClient::read_json(response)?;
        let results = response
            .items
            .into_iter()
            .filter_map(mail_tm_summary_from_raw)
            .collect::<Vec<_>>();
        let count = u64::try_from(results.len()).unwrap_or(u64::MAX);
        Ok(ListResponse { results, count })
    }

    /// 使用 mail.tm bearer token 拉取邮件。
    pub fn get_message_by_token(
        &self,
        token: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> anyhow::Result<TempMailMessageDetail> {
        let path = format!("/messages/{}", message_id.as_ref().trim());
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(token.as_ref())).send()?;
        let response: MailTmMessageDetailRaw = HttpApiClient::read_json(response)?;

        TempMailMessageDetail::try_from(response)
    }
}

impl TempMailProvider for MailTmTempMailApi {
    fn provider_kind(&self) -> TempMailProviderKind {
        TempMailProviderKind::MailTm
    }

    fn create_mailbox(&self, request: &CreateMailboxRequest) -> anyhow::Result<TempMailMailbox> {
        self.create_mailbox_and_login(
            request.name.as_deref().unwrap_or("az"),
            request.password_length,
        )
    }

    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> anyhow::Result<ListResponse<TempMailMessageSummary>> {
        self.list_messages_by_token(&mailbox.credential, page)
    }

    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> anyhow::Result<Option<TempMailMessageDetail>> {
        self.get_message_by_token(&mailbox.credential, message_id)
            .map(Some)
    }
}

/// 创建托管 `mail.tm` API 客户端。
///
/// 这里不会设置 `Accept: application/json`，因为 mail.tm 在该 header 存在时会返回纯 JSON 数组，
/// 而不是 `{"hydra:member": [...]}` 集合包裹，从而破坏 `HydraCollection` 反序列化。
pub fn create_mail_tm_api() -> anyhow::Result<MailTmTempMailApi> {
    let config = ApiConfig::builder("https://api.mail.tm").build()?;
    MailTmTempMailApi::new(config)
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct MailTmDomain {
    pub id: String,
    pub domain: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
}

#[derive(Debug, serde::Deserialize)]
struct HydraCollection<T> {
    #[serde(rename = "hydra:member", default = "Vec::new")]
    items: Vec<T>,
}

#[derive(Debug, serde::Deserialize)]
struct MailTmAccountResponse {
    #[serde(default)]
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct MailTmTokenResponse {
    #[serde(default)]
    token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct MailTmSenderRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct MailTmRecipientRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct MailTmMessageSummaryRaw {
    #[serde(default)]
    id: String,
    from: MailTmSenderRaw,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    intro: String,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}

#[derive(Debug, serde::Deserialize)]
struct MailTmMessageDetailRaw {
    #[serde(default)]
    id: String,
    from: MailTmSenderRaw,
    #[serde(default)]
    to: Vec<MailTmRecipientRaw>,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    html: Value,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}

fn mail_tm_summary_from_raw(raw: MailTmMessageSummaryRaw) -> Option<TempMailMessageSummary> {
    let id = raw.id.trim().to_owned();
    if id.is_empty() {
        return None;
    }

    Some(TempMailMessageSummary {
        id,
        from_address: raw.from.address,
        from_name: raw.from.name,
        subject: raw.subject,
        intro: raw.intro,
        created_at: raw.created_at,
    })
}

impl TryFrom<MailTmMessageDetailRaw> for TempMailMessageDetail {
    type Error = anyhow::Error;

    fn try_from(raw: MailTmMessageDetailRaw) -> Result<Self, Self::Error> {
        let html = match raw.html {
            Value::String(value) => value,
            Value::Array(values) => values
                .into_iter()
                .find_map(|item| item.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(String::new),
            Value::Null => String::new(),
            other => {
                anyhow::bail!(
                    "invalid response: mail.tm html field should be string or array, got {other}"
                );
            }
        };

        Ok(Self {
            id: raw.id,
            from_address: raw.from.address,
            from_name: raw.from.name,
            to: raw
                .to
                .into_iter()
                .map(|recipient| TempMailRecipient {
                    address: recipient.address,
                    name: recipient.name,
                })
                .collect(),
            subject: raw.subject,
            text: raw.text,
            html,
            raw: String::new(),
            created_at: raw.created_at,
        })
    }
}

const fn mail_tm_page(page: PageRequest) -> usize {
    let normalized = page.offset / page.limit;
    normalized + 1
}
