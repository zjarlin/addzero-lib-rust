use crate::http::HttpApiClient;
use crate::model::{
    AddressCredential, AddressLoginRequest, AddressSettings, CreateMailboxRequest, ListResponse,
    MailRow, NewAddressRequest, PageRequest, ParsedMailRow, SendMailRequest, SuccessResponse,
    TempMailMailbox, TempMailMessageDetail, TempMailMessageSummary, TempMailProviderKind,
    TempMailSettings,
};
use crate::provider::TempMailProvider;
use crate::util::{required_non_blank, sha256_hex};
use crate::{ApiConfig, TempMailResult};
use reqwest::header::ACCEPT;
use serde_json::json;

/// Blocking API client for a `dreamhunter2333/cloudflare_temp_email` deployment.
#[derive(Debug, Clone)]
pub struct CloudflareTempMailApi {
    http: HttpApiClient,
}

impl CloudflareTempMailApi {
    /// Creates a client from explicit worker configuration.
    pub fn new(config: ApiConfig) -> TempMailResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    /// Reads public worker settings from `/open_api/settings`.
    pub fn open_settings(&self) -> TempMailResult<TempMailSettings> {
        let response = self.http.get("/open_api/settings")?.send()?;
        HttpApiClient::read_json(response)
    }

    /// Creates an address through `/api/new_address`.
    pub fn new_address(&self, request: &NewAddressRequest) -> TempMailResult<AddressCredential> {
        let response = self.http.post("/api/new_address")?.json(request).send()?;
        HttpApiClient::read_json(response)
    }

    /// Verifies a saved address JWT through `/open_api/credential_login`.
    pub fn credential_login(&self, credential: impl AsRef<str>) -> TempMailResult<bool> {
        let credential = required_non_blank(credential.as_ref(), "credential")?;
        let response = self
            .http
            .post("/open_api/credential_login")?
            .json(&json!({ "credential": credential }))
            .send()?;
        let response: SuccessResponse = HttpApiClient::read_json(response)?;
        Ok(response.success)
    }

    /// Logs in to an address-password deployment.
    pub fn address_login(
        &self,
        request: &AddressLoginRequest,
    ) -> TempMailResult<AddressCredential> {
        let response = self.http.post("/api/address_login")?.json(request).send()?;
        HttpApiClient::read_json(response)
    }

    /// Logs in by hashing a plain password in the same way as the upstream frontend.
    pub fn address_login_plain_password(
        &self,
        email: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> TempMailResult<AddressCredential> {
        let email = required_non_blank(email.as_ref(), "email")?;
        let password_hash = sha256_hex(password.as_ref());
        self.address_login(&AddressLoginRequest::hashed(email, password_hash))
    }

    /// Changes an address password. The worker expects the frontend SHA-256 hash.
    pub fn change_password(
        &self,
        address_jwt: impl AsRef<str>,
        new_password_hash: impl AsRef<str>,
    ) -> TempMailResult<SuccessResponse> {
        let new_password = required_non_blank(new_password_hash.as_ref(), "new_password_hash")?;
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("/api/address_change_password")?,
            Some(address_jwt.as_ref()),
        )
        .json(&json!({ "new_password": new_password }))
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Hashes a plain password and changes the address password.
    pub fn change_plain_password(
        &self,
        address_jwt: impl AsRef<str>,
        new_password: impl AsRef<str>,
    ) -> TempMailResult<SuccessResponse> {
        self.change_password(address_jwt, sha256_hex(new_password.as_ref()))
    }

    /// Reads address settings and send balance from `/api/settings`.
    pub fn address_settings(
        &self,
        address_jwt: impl AsRef<str>,
    ) -> TempMailResult<AddressSettings> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.get("/api/settings")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Lists raw inbox messages from `/api/mails`.
    pub fn list_mails(
        &self,
        address_jwt: impl AsRef<str>,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<MailRow>> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.get("/api/mails")?,
            Some(address_jwt.as_ref()),
        )
        .query(&page.to_query())
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Fetches one raw inbox message by id from `/api/mail/:id`.
    pub fn get_mail(
        &self,
        address_jwt: impl AsRef<str>,
        mail_id: u64,
    ) -> TempMailResult<Option<MailRow>> {
        let path = format!("/api/mail/{mail_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(address_jwt.as_ref()))
                .send()?;
        HttpApiClient::read_json(response)
    }

    /// Lists server-side parsed inbox messages from `/api/parsed_mails`.
    pub fn list_parsed_mails(
        &self,
        address_jwt: impl AsRef<str>,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<ParsedMailRow>> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.get("/api/parsed_mails")?,
            Some(address_jwt.as_ref()),
        )
        .query(&page.to_query())
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Fetches one server-side parsed inbox message by id from `/api/parsed_mail/:id`.
    pub fn get_parsed_mail(
        &self,
        address_jwt: impl AsRef<str>,
        mail_id: u64,
    ) -> TempMailResult<Option<ParsedMailRow>> {
        let path = format!("/api/parsed_mail/{mail_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(address_jwt.as_ref()))
                .send()?;
        HttpApiClient::read_json(response)
    }

    /// Deletes one inbox message from `/api/mails/:id`.
    pub fn delete_mail(
        &self,
        address_jwt: impl AsRef<str>,
        mail_id: u64,
    ) -> TempMailResult<SuccessResponse> {
        let path = format!("/api/mails/{mail_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.delete(&path)?, Some(address_jwt.as_ref()))
                .send()?;
        HttpApiClient::read_json(response)
    }

    /// Clears all inbox rows for the current address.
    pub fn clear_inbox(&self, address_jwt: impl AsRef<str>) -> TempMailResult<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.delete("/api/clear_inbox")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Deletes the current address and its mailbox data.
    pub fn delete_address(&self, address_jwt: impl AsRef<str>) -> TempMailResult<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.delete("/api/delete_address")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Requests send-mail access for the current address.
    pub fn request_send_mail_access(
        &self,
        address_jwt: impl AsRef<str>,
    ) -> TempMailResult<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("/api/request_send_mail_access")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Sends mail from the current address through `/api/send_mail`.
    pub fn send_mail(
        &self,
        address_jwt: impl AsRef<str>,
        request: &SendMailRequest,
    ) -> TempMailResult<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("/api/send_mail")?,
            Some(address_jwt.as_ref()),
        )
        .json(request)
        .send()?;
        HttpApiClient::read_json(response)
    }

    /// Clears sent items for the current address.
    pub fn clear_sent_items(
        &self,
        address_jwt: impl AsRef<str>,
    ) -> TempMailResult<SuccessResponse> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.delete("/api/clear_sent_items")?,
            Some(address_jwt.as_ref()),
        )
        .send()?;
        HttpApiClient::read_json(response)
    }
}

/// Creates a Temp Email client for a deployed Cloudflare worker.
impl TempMailProvider for CloudflareTempMailApi {
    fn provider_kind(&self) -> TempMailProviderKind {
        TempMailProviderKind::Cloudflare
    }

    fn create_mailbox(&self, request: &CreateMailboxRequest) -> TempMailResult<TempMailMailbox> {
        self.new_address(&NewAddressRequest::from(request))
            .map(TempMailMailbox::from_cloudflare)
    }

    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>> {
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
    ) -> TempMailResult<Option<TempMailMessageDetail>> {
        let Ok(mail_id) = message_id.trim().parse::<u64>() else {
            return Ok(None);
        };
        self.get_mail(&mailbox.credential, mail_id)
            .map(|mail| mail.map(TempMailMessageDetail::from))
    }
}

/// Backward-compatible alias for the default concrete temp-mail implementation.
pub type TempMailApi = CloudflareTempMailApi;

pub fn create_temp_mail_api(base_url: impl Into<String>) -> TempMailResult<CloudflareTempMailApi> {
    let config = ApiConfig::builder(base_url)
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    CloudflareTempMailApi::new(config)
}
