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

/// 单个内置临时邮箱 provider 的配置。
#[apply(plain_eq)]
pub enum TempMailProviderConfig {
    /// 兼容 `dreamhunter2333/cloudflare_temp_email` 的 Cloudflare Worker 部署。
    Cloudflare(ApiConfig),
    /// 托管或自托管的 mail.tm 兼容 API。
    MailTm(ApiConfig),
    /// 托管的 Emailnator webmail API。
    Emailnator(ApiConfig),
}

impl_enum_kind!(TempMailProviderConfig => TempMailProviderKind, kind {
    Self::Cloudflare(_) => TempMailProviderKind::Cloudflare,
    Self::MailTm(_) => TempMailProviderKind::MailTm,
    Self::Emailnator(_) => TempMailProviderKind::Emailnator,
});

/// 临时邮箱 provider 的通用收信契约。
///
/// 该 trait 只建模稳定的跨 provider 能力：创建地址、列出邮件、拉取单封邮件。
/// Cloudflare 发信权限等 provider 专属能力保留在具体客户端上。
pub trait TempMailProvider: Send + Sync {
    /// 标识当前 provider 实现。
    fn provider_kind(&self) -> TempMailProviderKind;

    /// 创建邮箱，并返回后续调用所需的 provider 凭据。
    fn create_mailbox(&self, request: &CreateMailboxRequest) -> TempMailResult<TempMailMailbox>;

    /// 列出已创建邮箱中的邮件。
    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>>;

    /// 按 provider 消息 ID 拉取单封邮件。
    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> TempMailResult<Option<TempMailMessageDetail>>;
}

/// 应用边界使用的 boxed 临时邮箱 provider 对象。
pub type BoxTempMailProvider = Box<dyn TempMailProvider + Send + Sync>;

/// 用于依赖注入式创建临时邮箱 provider 的工厂抽象。
pub trait TempMailProviderFactory: Send + Sync {
    /// 根据 provider 专属配置构造 provider trait object。
    fn build_provider(&self, config: TempMailProviderConfig)
    -> TempMailResult<BoxTempMailProvider>;
}

/// 本 crate 内置 provider 的默认工厂。
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

/// 根据 provider 专属配置构造 provider trait object。
pub fn build_temp_mail_provider(
    config: TempMailProviderConfig,
) -> TempMailResult<BoxTempMailProvider> {
    BuiltinTempMailProviderFactory.build_provider(config)
}
