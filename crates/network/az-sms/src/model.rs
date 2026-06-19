use anyhow::bail;
use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::time::Duration;

/// 一次性短信验证号码购买请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmsActivationRequest {
    /// provider 国家代码，或 provider 专属的 `any`。
    pub country: String,
    /// provider 运营商代码，或 provider 专属的 `any`。
    pub operator: String,
    /// provider 产品或服务名称。
    pub product: String,
    /// provider 支持时启用来电转发。
    pub forwarding: Option<bool>,
    /// provider 专属复用流程中的已有手机号。
    pub number: Option<String>,
    /// provider 支持时请求可复用号码。
    pub reuse: Option<bool>,
    /// provider 支持时请求语音验证。
    pub voice: Option<bool>,
    /// provider 推荐码。
    #[serde(rename = "ref")]
    pub ref_code: Option<String>,
}

impl SmsActivationRequest {
    /// 创建已校验的一次性号码请求。
    pub fn new(
        country: impl Into<String>,
        operator: impl Into<String>,
        product: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let request = Self {
            country: country.into(),
            operator: operator.into(),
            product: product.into(),
            forwarding: None,
            number: None,
            reuse: None,
            voice: None,
            ref_code: None,
        };
        request.validate()?;
        Ok(request)
    }

    /// 设置来电转发。
    pub fn forwarding(mut self, value: bool) -> Self {
        self.forwarding = Some(value);
        self
    }

    /// 设置 provider 专属的手机号复用值。
    pub fn number(mut self, value: impl Into<String>) -> Self {
        self.number = Some(value.into());
        self
    }

    /// 启用或禁用号码复用。
    pub fn reuse(mut self, value: bool) -> Self {
        self.reuse = Some(value);
        self
    }

    /// 启用或禁用语音验证。
    pub fn voice(mut self, value: bool) -> Self {
        self.voice = Some(value);
        self
    }

    /// 设置 provider 推荐码。
    pub fn ref_code(mut self, value: impl Into<String>) -> Self {
        self.ref_code = Some(value.into());
        self
    }

    /// 发送给 provider 前校验本地不变量。
    pub fn validate(&self) -> anyhow::Result<()> {
        validate_non_blank("country", &self.country)?;
        validate_non_blank("operator", &self.operator)?;
        validate_non_blank("product", &self.product)?;
        validate_optional_non_blank("number", self.number.as_deref())?;
        validate_optional_non_blank("ref_code", self.ref_code.as_deref())?;
        Ok(())
    }
}

/// 长时托管/租用短信号码购买请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmsHostingRequest {
    /// provider 国家代码，或 provider 专属的 `any`。
    pub country: String,
    /// provider 运营商代码，或 provider 专属的 `any`。
    pub operator: String,
    /// provider 产品或服务名称。
    pub product: String,
}

impl SmsHostingRequest {
    /// 创建已校验的托管/租用号码请求。
    pub fn new(
        country: impl Into<String>,
        operator: impl Into<String>,
        product: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let request = Self {
            country: country.into(),
            operator: operator.into(),
            product: product.into(),
        };
        request.validate()?;
        Ok(request)
    }

    /// 发送给 provider 前校验本地不变量。
    pub fn validate(&self) -> anyhow::Result<()> {
        validate_non_blank("country", &self.country)?;
        validate_non_blank("operator", &self.operator)?;
        validate_non_blank("product", &self.product)?;
        Ok(())
    }
}

/// provider 订单状态。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SmsOrderStatus {
    /// provider 正在准备号码。
    Pending,
    /// 已收到短信，或订单正在等待短信回执。
    Received,
    /// 订单已取消。
    Canceled,
    /// provider 已将订单置为超时。
    Timeout,
    /// 订单已成功完成。
    Finished,
    /// 订单已被封禁或拒绝。
    Banned,
    /// provider 返回了本 crate 尚未识别的状态。
    #[serde(other)]
    Unknown,
}

impl SmsOrderStatus {
    /// 判断该状态是否会关闭订单生命周期。
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Canceled | Self::Timeout | Self::Finished | Self::Banned
        )
    }
}

/// provider 返回的单条短信。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmsMessage {
    /// provider 提供时的短信 ID。
    #[serde(default, alias = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// provider 创建时间戳。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// 发送方侧消息时间戳。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// 发送方名称或手机号。
    #[serde(default)]
    pub sender: String,
    /// 原始短信正文。
    #[serde(default)]
    pub text: String,
    /// provider 可用时提取出的验证码。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// SMS provider 订单数据。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SmsOrder {
    /// provider 订单 ID。
    pub id: u64,
    /// 租用到的手机号。
    pub phone: String,
    /// provider 运营商名称。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    /// provider 产品或服务名称。
    pub product: String,
    /// provider 订单价格。
    pub price: f64,
    /// 当前 provider 订单状态。
    pub status: SmsOrderStatus,
    /// provider 过期时间戳。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    /// 关联到该订单的短信列表。
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub sms: Vec<SmsMessage>,
    /// provider 创建时间戳。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// 是否启用了转发。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forwarding: Option<bool>,
    /// 启用转发时的目标号码。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forwarding_number: Option<String>,
    /// provider 国家名称。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

/// 暴露租用号码 inbox 的 provider 所返回的 SMS inbox。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmsInbox {
    /// provider 返回的短信列表。
    #[serde(rename = "Data", default)]
    pub messages: Vec<SmsMessage>,
    /// provider 报告的短信总数。
    #[serde(rename = "Total", default)]
    pub total: usize,
}

/// 最小 provider 账号 profile。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SmsProfile {
    /// provider 账号 ID。
    pub id: u64,
    /// provider 账号邮箱。
    #[serde(default)]
    pub email: String,
    /// 当前账号余额。
    pub balance: f64,
    /// provider 账号评分。
    #[serde(default)]
    pub rating: f64,
    /// provider 暴露时的冻结余额。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_balance: Option<f64>,
}

/// [`crate::provider::SmsProvider::wait_for_sms`] 使用的轮询选项。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaitForSmsOptions {
    /// 最大轮询时长。
    pub timeout: Duration,
    /// 两次 `check_order` 调用之间的等待时间。
    pub interval: Duration,
}

impl Default for WaitForSmsOptions {
    fn default() -> Self {
        WaitForSmsOptions {
    timeout: Duration::from_secs(300),
    interval: Duration::from_secs(5),
}
    }
}

impl WaitForSmsOptions {
    /// 创建轮询选项，并校验两个时长均非零。
    pub fn new(timeout: Duration, interval: Duration) -> anyhow::Result<Self> {
        let options = Self { timeout, interval };
        options.validate()?;
        Ok(options)
    }

    /// 校验本地轮询不变量。
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.timeout.is_zero() {
            bail!("invalid request: timeout cannot be zero");
        }
        if self.interval.is_zero() {
            bail!("invalid request: interval cannot be zero");
        }
        Ok(())
    }
}

fn validate_non_blank(name: &str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        bail!("invalid request: {name} cannot be blank");
    }
    Ok(())
}

fn validate_optional_non_blank(name: &str, value: Option<&str>) -> anyhow::Result<()> {
    if value.is_some_and(|item| item.trim().is_empty()) {
        bail!("invalid request: {name} cannot be blank");
    }
    Ok(())
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(|items| items.unwrap_or_default())
}

#[allow(dead_code)]
fn reject_unknown<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        Err(de::Error::custom("value cannot be blank"))
    } else {
        Ok(value)
    }
}
