use crate::config::ApiConfig;
use crate::http::HttpApiClient;
use crate::input::{required_non_blank, sha256_hex};
use crate::model::{
    AddressCredential, AddressLoginRequest, AddressSettings, CreateMailboxRequest, ListResponse,
    MailRow, NewAddressRequest, PageRequest, ParsedMailRow, SendMailRequest, SuccessResponse,
    TempMailMailbox, TempMailMessageDetail, TempMailMessageSummary, TempMailProviderKind,
    TempMailSettings,
};
use crate::provider::TempMailProvider;
use reqwest::header::ACCEPT;
use serde_json::json;

/// `dreamhunter2333/cloudflare_temp_email` 部署的阻塞 API 客户端。
#[derive(Clone, Debug)]
pub struct CloudflareTempMailApi {
    http: HttpApiClient,
}

impl CloudflareTempMailApi {
    /// 根据显式 worker 配置创建客户端。
    pub fn new(config: ApiConfig) -> anyhow::Result<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    /// 从 `/open_api/settings` 读取公开 worker 设置。
    pub fn open_settings(&self) -> anyhow::Result<TempMailSettings> {
        let response = self.http.get("/open_api/settings")?.send()?;
        HttpApiClient::read_json(response)
    }

    /// 通过 `/api/new_address` 创建地址。
    pub fn new_address(&self, request: &NewAddressRequest) -> anyhow::Result<AddressCredential> {
        let response = self.http.post("/api/new_address")?.json(request).send()?;
        HttpApiClient::read_json(response)
    }

    /// 通过 `/open_api/credential_login` 校验已保存的地址 JWT。
    pub fn credential_login(&self, credential: impl AsRef<str>) -> anyhow::Result<bool> {
        let credential = required_non_blank(credential.as_ref(), "credential")?;
        let response = self
            .http
            .post("/open_api/credential_login")?
            .json(&json!({ "credential": credential }))
            .send()?;
        let response: SuccessResponse = HttpApiClient::read_json(response)?;
        Ok(response.success)
    }

    /// 登录启用地址密码的部署。
    pub fn address_login(
        &self,
        request: &AddressLoginRequest,
    ) -> anyhow::Result<AddressCredential> {
        let response = self.http.post("/api/address_login")?.json(request).send()?;
        HttpApiClient::read_json(response)
    }

    /// 按上游前端同样方式哈希明文密码后登录。
    pub fn address_login_plain_password(
        &self,
        email: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> anyhow::Result<AddressCredential> {
        let email = required_non_blank(email.as_ref(), "email")?;
        let password_hash = sha256_hex(password.as_ref());
        self.address_login(&AddressLoginRequest::hashed(email, password_hash))
    }

    /// 修改地址密码；worker 期望收到前端格式的 SHA-256 哈希。
    pub fn change_password(
        &self,
        address_jwt: impl AsRef<str>,
        new_password_hash: impl AsRef<str>,
    ) -> anyhow::Result<SuccessResponse> {
        let new_password = required_non_blank(new_password_hash.as_ref(), "new_password_hash")?;
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("/api/address_change_password")?,
            Some(address_jwt.as_ref()),
        )
        .json(&json!({ "new_password": new_password }))
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 哈希明文密码并修改地址密码。
    pub fn change_plain_password(
        &self,
        address_jwt: impl AsRef<str>,
        new_password: impl AsRef<str>,
    ) -> anyhow::Result<SuccessResponse> {
        self.change_password(address_jwt, sha256_hex(new_password.as_ref()))
    }

    /// 从 `/api/settings` 读取地址设置和发信余额。
    pub fn address_settings(
        &self,
        address_jwt: impl AsRef<str>,
    ) -> anyhow::Result<AddressSettings> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.get("/api/settings")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 从 `/api/mails` 列出原始收件箱邮件。
    pub fn list_mails(
        &self,
        address_jwt: impl AsRef<str>,
        page: PageRequest,
    ) -> anyhow::Result<ListResponse<MailRow>> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.get("/api/mails")?,
            Some(address_jwt.as_ref()),
        )
        .query(&page.to_query())
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 从 `/api/mail/:id` 按 ID 拉取单封原始邮件。
    pub fn get_mail(
        &self,
        address_jwt: impl AsRef<str>,
        mail_id: u64,
    ) -> anyhow::Result<Option<MailRow>> {
        let path = format!("/api/mail/{mail_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(address_jwt.as_ref()))
                .send()?;
        HttpApiClient::read_json(response)
    }

    /// 从 `/api/parsed_mails` 列出服务端解析后的收件箱邮件。
    pub fn list_parsed_mails(
        &self,
        address_jwt: impl AsRef<str>,
        page: PageRequest,
    ) -> anyhow::Result<ListResponse<ParsedMailRow>> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.get("/api/parsed_mails")?,
            Some(address_jwt.as_ref()),
        )
        .query(&page.to_query())
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 从 `/api/parsed_mail/:id` 按 ID 拉取单封服务端解析邮件。
    pub fn get_parsed_mail(
        &self,
        address_jwt: impl AsRef<str>,
        mail_id: u64,
    ) -> anyhow::Result<Option<ParsedMailRow>> {
        let path = format!("/api/parsed_mail/{mail_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(address_jwt.as_ref()))
                .send()?;
        HttpApiClient::read_json(response)
    }

    /// 从 `/api/mails/:id` 删除单封收件箱邮件。
    pub fn delete_mail(
        &self,
        address_jwt: impl AsRef<str>,
        mail_id: u64,
    ) -> anyhow::Result<SuccessResponse> {
        let path = format!("/api/mails/{mail_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.delete(&path)?, Some(address_jwt.as_ref()))
                .send()?;
        HttpApiClient::read_json(response)
    }

    /// 清空当前地址的所有收件箱记录。
    pub fn clear_inbox(&self, address_jwt: impl AsRef<str>) -> anyhow::Result<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.delete("/api/clear_inbox")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 删除当前地址及其邮箱数据。
    pub fn delete_address(&self, address_jwt: impl AsRef<str>) -> anyhow::Result<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.delete("/api/delete_address")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 为当前地址申请发信权限。
    pub fn request_send_mail_access(
        &self,
        address_jwt: impl AsRef<str>,
    ) -> anyhow::Result<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("/api/request_send_mail_access")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 通过 `/api/send_mail` 从当前地址发信。
    pub fn send_mail(
        &self,
        address_jwt: impl AsRef<str>,
        request: &SendMailRequest,
    ) -> anyhow::Result<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("/api/send_mail")?,
            Some(address_jwt.as_ref()),
        )
        .json(request)
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// 清空当前地址的已发送记录。
    pub fn clear_sent_items(
        &self,
        address_jwt: impl AsRef<str>,
    ) -> anyhow::Result<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.delete("/api/clear_sent_items")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }
}

/// 为已部署的 Cloudflare worker 创建 Temp Email 客户端。
impl TempMailProvider for CloudflareTempMailApi {
    fn provider_kind(&self) -> TempMailProviderKind {
        TempMailProviderKind::Cloudflare
    }

    fn create_mailbox(&self, request: &CreateMailboxRequest) -> anyhow::Result<TempMailMailbox> {
        self.new_address(&NewAddressRequest::from(request))
            .map(TempMailMailbox::from_cloudflare)
    }

    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> anyhow::Result<ListResponse<TempMailMessageSummary>> {
        let response = self.list_mails(&mailbox.credential, page)?;
        Ok(ListResponse {
            results: response
                .results
                .into_iter()
                .map(TempMailMessageSummary::from)
                .collect(),
            count: response.count,
        })
    }

    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> anyhow::Result<Option<TempMailMessageDetail>> {
        let Ok(mail_id) = message_id.trim().parse::<u64>() else {
            return Ok(None);
        };
        self.get_mail(&mailbox.credential, mail_id)
            .map(|mail| mail.map(TempMailMessageDetail::from))
    }
}

/// 默认具体临时邮箱实现的向后兼容别名。
pub type TempMailApi = CloudflareTempMailApi;

pub fn create_temp_mail_api(base_url: impl Into<String>) -> anyhow::Result<CloudflareTempMailApi> {
    let config = ApiConfig::builder(base_url)
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    CloudflareTempMailApi::new(config)
}
