use crate::dogsms::client::{DogSmsClient, DogSmsConfig};
use crate::grizzlysms::client::{GrizzlySmsClient, GrizzlySmsConfig};
use crate::model::{
    SmsActivationRequest, SmsHostingRequest, SmsInbox, SmsOrder, WaitForSmsOptions,
};
use az_derive_aliases::{
    apply, from_plain_eq, impl_enum_kind, plain_default_copy_eq, serde_code_enum,
};
use anyhow::bail;
use std::time::Instant;

/// 内置 SMS provider 标识。
#[apply(serde_code_enum)]
pub enum SmsProviderKind {
    /// DogeSMS Control API。
    #[serde(rename = "dogsms")]
    #[strum(serialize = "dogsms")]
    DogSms,
    /// Grizzly SMS 兼容 sms-activate 的 API。
    GrizzlySms,
}

/// 单个内置 SMS provider 的配置。
#[apply(from_plain_eq)]
pub enum SmsProviderConfig {
    /// DogeSMS Control API 配置。
    DogSms(DogSmsConfig),
    /// Grizzly SMS API 配置。
    GrizzlySms(GrizzlySmsConfig),
}

impl_enum_kind!(SmsProviderConfig => SmsProviderKind, kind {
    Self::DogSms(_) => SmsProviderKind::DogSms,
    Self::GrizzlySms(_) => SmsProviderKind::GrizzlySms,
});

/// 应用边界使用的 boxed provider 对象。
pub type BoxSmsProvider = Box<dyn SmsProvider + Send + Sync>;

/// 用于依赖注入式创建 SMS provider 的工厂抽象。
pub trait SmsProviderFactory: Send + Sync {
    /// 根据 provider 专属配置构造 provider trait object。
    fn build_provider(&self, config: SmsProviderConfig) -> anyhow::Result<BoxSmsProvider>;
}

/// 本 crate 内置 provider 的默认工厂。
#[apply(plain_default_copy_eq)]
pub struct BuiltinSmsProviderFactory;

impl SmsProviderFactory for BuiltinSmsProviderFactory {
    fn build_provider(&self, config: SmsProviderConfig) -> anyhow::Result<BoxSmsProvider> {
        match config {
            SmsProviderConfig::DogSms(config) => Ok(Box::new(DogSmsClient::new(config)?)),
            SmsProviderConfig::GrizzlySms(config) => Ok(Box::new(GrizzlySmsClient::new(config)?)),
        }
    }
}

/// 根据 provider 专属配置构造 provider trait object。
pub fn build_sms_provider(config: SmsProviderConfig) -> anyhow::Result<BoxSmsProvider> {
    BuiltinSmsProviderFactory.build_provider(config)
}

/// SMS provider 实现的通用异步接口。
#[async_trait::async_trait]
pub trait SmsProvider: Send + Sync {
    /// 购买一次性短信验证号码。
    async fn buy_activation_number(&self, request: SmsActivationRequest) -> anyhow::Result<SmsOrder>;

    /// 在 provider 支持时购买托管或租用号码。
    async fn buy_hosting_number(&self, request: SmsHostingRequest) -> anyhow::Result<SmsOrder>;

    /// 获取当前订单状态和已关联的短信内容。
    async fn check_order(&self, order_id: u64) -> anyhow::Result<SmsOrder>;

    /// 将订单标记为成功完成。
    async fn finish_order(&self, order_id: u64) -> anyhow::Result<SmsOrder>;

    /// 取消不再需要使用的订单。
    async fn cancel_order(&self, order_id: u64) -> anyhow::Result<SmsOrder>;

    /// 因号码或收到的短信不可用而封禁/拒绝订单。
    async fn ban_order(&self, order_id: u64) -> anyhow::Result<SmsOrder>;

    /// 获取托管或租用订单的短信 inbox。
    async fn inbox(&self, order_id: u64) -> anyhow::Result<SmsInbox>;

    /// 轮询 `check_order`，直到短信到达、订单关闭或超时。
    async fn wait_for_sms(&self, order_id: u64, options: WaitForSmsOptions) -> anyhow::Result<SmsOrder> {
        options.validate()?;
        let deadline = Instant::now() + options.timeout;

        loop {
            let order = self.check_order(order_id).await?;
            if !order.sms.is_empty() {
                return Ok(order);
            }
            if order.status.is_terminal() {
                bail!(
                    "order {order_id} closed before SMS arrived: {:?}",
                    order.status
                );
            }
            if Instant::now() >= deadline {
                bail!(
                    "timed out waiting for SMS on order {order_id} after {}s",
                    options.timeout.as_secs()
                );
            }
            tokio::time::sleep(options.interval).await;
        }
    }
}
