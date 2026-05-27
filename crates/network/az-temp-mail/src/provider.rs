use crate::TempMailResult;
use crate::client::CloudflareTempMailApi;
use crate::config::ApiConfig;
use crate::emailnator::EmailnatorTempMailApi;
use crate::mail_tm::MailTmTempMailApi;
use crate::model::{
    CreateMailboxRequest, ListResponse, PageRequest, TempMailMailbox, TempMailMessageDetail,
    TempMailMessageSummary, TempMailProviderKind,
};
use az_derive_aliases::{apply, impl_enum_kind, plain_default_copy_eq, plain_eq};

/// Configuration for one built-in temporary email provider.
#[apply(plain_eq)]
pub enum TempMailProviderConfig {
    /// Cloudflare Worker deployment compatible with `dreamhunter2333/cloudflare_temp_email`.
    Cloudflare(ApiConfig),
    /// Hosted or self-hosted mail.tm-compatible API.
    MailTm(ApiConfig),
    /// Hosted Emailnator webmail API.
    Emailnator(ApiConfig),
}

impl_enum_kind!(TempMailProviderConfig => TempMailProviderKind, kind {
    Self::Cloudflare(_) => TempMailProviderKind::Cloudflare,
    Self::MailTm(_) => TempMailProviderKind::MailTm,
    Self::Emailnator(_) => TempMailProviderKind::Emailnator,
});

/// Common receive-mail contract for temporary email providers.
///
/// The trait intentionally models only the stable cross-provider surface:
/// create an address, list messages, and fetch a message. Provider-specific
/// capabilities such as Cloudflare send-mail access stay on concrete clients.
pub trait TempMailProvider: Send + Sync {
    /// Identifies the provider implementation.
    fn provider_kind(&self) -> TempMailProviderKind;

    /// Creates a mailbox and returns the provider credential needed later.
    fn create_mailbox(&self, request: &CreateMailboxRequest) -> TempMailResult<TempMailMailbox>;

    /// Lists messages for a previously created mailbox.
    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>>;

    /// Fetches a single message by provider message id.
    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> TempMailResult<Option<TempMailMessageDetail>>;
}

/// Boxed temporary email provider object used at application boundaries.
pub type BoxTempMailProvider = Box<dyn TempMailProvider + Send + Sync>;

/// Factory abstraction for dependency-injected temporary email provider creation.
pub trait TempMailProviderFactory: Send + Sync {
    /// Build a provider trait object from a provider-specific config.
    fn build_provider(&self, config: TempMailProviderConfig)
    -> TempMailResult<BoxTempMailProvider>;
}

/// Factory for the providers compiled into this crate.
#[apply(plain_default_copy_eq)]
pub struct BuiltinTempMailProviderFactory;

impl TempMailProviderFactory for BuiltinTempMailProviderFactory {
    fn build_provider(
        &self,
        config: TempMailProviderConfig,
    ) -> TempMailResult<BoxTempMailProvider> {
        match config {
            TempMailProviderConfig::Cloudflare(config) => {
                Ok(Box::new(CloudflareTempMailApi::new(config)?))
            }
            TempMailProviderConfig::MailTm(config) => Ok(Box::new(MailTmTempMailApi::new(config)?)),
            TempMailProviderConfig::Emailnator(config) => {
                Ok(Box::new(EmailnatorTempMailApi::new(config)?))
            }
        }
    }
}

/// Build a provider trait object from a provider-specific config.
pub fn build_temp_mail_provider(
    config: TempMailProviderConfig,
) -> TempMailResult<BoxTempMailProvider> {
    BuiltinTempMailProviderFactory.build_provider(config)
}
