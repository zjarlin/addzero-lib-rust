use crate::client::{TempMailApi, create_temp_mail_api};
use crate::cloudflare::CloudflareTempMailContext;
use crate::config::ApiConfig;
use crate::emailnator::{EmailnatorTempMailApi, create_emailnator_api};
use crate::mail_tm::{MailTmTempMailApi, create_mail_tm_api};
use crate::provider::{
    BoxTempMailProvider, TempMailProviderConfig, TempMailProviderFactory, build_temp_mail_provider,
};

/// 用于构造临时邮箱客户端的命名空间式入口。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TempMail;

impl TempMail {
    /// 创建已部署 Cloudflare Temp Email worker 的客户端。
    pub fn cloudflare(base_url: impl Into<String>) -> anyhow::Result<TempMailApi> {
        create_temp_mail_api(base_url)
    }

    /// 根据显式配置创建客户端。
    pub fn cloudflare_with_config(config: ApiConfig) -> anyhow::Result<TempMailApi> {
        TempMailApi::new(config)
    }

    /// 根据较高层的 Cloudflare worker 上下文创建客户端。
    pub fn cloudflare_with_context(
        context: &CloudflareTempMailContext,
    ) -> anyhow::Result<TempMailApi> {
        context.create_api()
    }

    /// 创建托管 mail.tm 兼容 provider 客户端。
    pub fn mail_tm() -> anyhow::Result<MailTmTempMailApi> {
        create_mail_tm_api()
    }

    /// 根据显式配置创建 mail.tm 兼容客户端。
    pub fn mail_tm_with_config(config: ApiConfig) -> anyhow::Result<MailTmTempMailApi> {
        MailTmTempMailApi::new(config)
    }

    /// 创建托管 Emailnator 临时邮箱服务客户端。
    pub fn emailnator() -> anyhow::Result<EmailnatorTempMailApi> {
        create_emailnator_api()
    }

    /// 根据显式配置创建 Emailnator 客户端。
    pub fn emailnator_with_config(config: ApiConfig) -> anyhow::Result<EmailnatorTempMailApi> {
        EmailnatorTempMailApi::new(config)
    }

    /// 根据 provider 专属配置构造 boxed provider。
    pub fn provider(config: TempMailProviderConfig) -> anyhow::Result<BoxTempMailProvider> {
        build_temp_mail_provider(config)
    }

    /// 通过注入工厂构造 boxed provider。
    pub fn provider_with_factory(
        factory: &dyn TempMailProviderFactory,
        config: TempMailProviderConfig,
    ) -> anyhow::Result<BoxTempMailProvider> {
        factory.build_provider(config)
    }
}
