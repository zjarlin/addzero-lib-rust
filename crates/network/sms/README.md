# az-sms

`az-sms` 是一个可复用的短信服务 crate，当前内置 DogeSMS 和 Grizzly SMS API client。

它只封装通用 SMS provider 能力：

- 查询账号 profile
- 购买一次性短信验证号码
- 购买托管/租用号码
- 查询订单状态和短信内容
- 等待短信到达
- 完成、取消、封禁订单
- 查询托管号码 inbox

它不封装账号批量注册、绕过第三方平台验证、浏览器反检测或其他平台规避流程。调用方需要确保用途符合服务商条款和目标平台规则。

## 添加依赖

在 `addzero-lib-rust` workspace 内使用：

```toml
[dependencies]
az-sms = { path = "../../network/sms" }
```

从仓库外部引用当前本地 checkout：

```toml
[dependencies]
az-sms = { path = "/Users/zjarlin/IdeaProjects/zjarlin/addzero-lib-rust/crates/network/sms" }
```

## DogeSMS / DogSMS

DogeSMS 使用 Control API：读接口通过 `X-API-Key` 鉴权，创建或取消订单等写接口还必须带 `Idempotency-Key`。本 crate 提供 native DogeSMS 方法，按 1/2/3 调用即可：

```rust,no_run
use az_sms::dogsms::client::{DogSmsActivationRequest, DogSmsClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("DOGSMS_API_KEY")
        .expect("DOGSMS_API_KEY is required");

    let client = DogSmsClient::from_api_key(api_key)?;

    // 1. 查账户与库存：GET /api/control/balance + GET /api/control/inventory
    let balance = client.balance().await?;
    let inventory = client.inventory("telegram", Some("US")).await?;
    println!("balance: {}", balance.balance);
    println!("inventory rows: {}", inventory.len());

    // 2. 创建订单：POST /api/control/activations
    let request = DogSmsActivationRequest::new("telegram", "US")?.reuse(true);
    let order = client
        .create_activation_with_idempotency_key(request, "order-telegram-us-001")
        .await?;
    println!("request id: {}", order.request_id);
    println!("phone: {}", order.number);

    // 3. 查状态/短信或取消：GET /api/control/activations/{requestId}
    //    PATCH /api/control/activations/{requestId}/cancel
    let order = client.activation(&order.request_id).await?;
    if let Some(message) = order.sms.first() {
        println!("text: {}", message.text);
        println!("code: {:?}", message.code);
    }

    Ok(())
}
```

长期租号使用 native rental API：

```rust,no_run
use az_sms::dogsms::client::{DogSmsClient, DogSmsRentalRequest};

async fn rent(client: &DogSmsClient) -> anyhow::Result<()> {
    let request = DogSmsRentalRequest::new("GB", 3)?.note("project-alpha");
    let rental = client
        .create_rental_with_idempotency_key(request, "rent-gb-3m-001")
        .await?;

    println!("rental id: {}", rental.rental_id);
    println!("phone: {}", rental.number);
    Ok(())
}
```

DogeSMS 官方文档中的 activation/rental ID 是字符串，例如 `act-123` 或 `rent-123`，所以推荐优先使用 `az_sms::dogsms::client` native API。`SmsProvider` trait 只支持 `u64` 订单 ID，只有当 DogeSMS 返回纯数字 `requestId` 时才适合通过统一 trait 调用。

## Grizzly SMS

Grizzly SMS 使用 `api_key` 查询参数，API 形状兼容 sms-activate。一次性号码用同一个 `SmsProvider` trait：

```rust,no_run
use az_sms::grizzlysms::client::GrizzlySmsClient;
use az_sms::model::SmsActivationRequest;
use az_sms::provider::SmsProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("GRIZZLYSMS_API_KEY")
        .expect("GRIZZLYSMS_API_KEY is required");

    let client = GrizzlySmsClient::from_api_key(api_key)?;
    let request = SmsActivationRequest::new("12", "any", "tg")?;
    let order = client.buy_activation_number(request).await?;

    println!("order id: {}", order.id);
    println!("phone: {}", order.phone);
    Ok(())
}
```

Grizzly SMS 官方文档当前没有公开托管/租号 inbox API，因此 `buy_hosting_number` 和 `inbox` 会返回 `anyhow` 错误。

## Provider Factory

需要把 provider 注入到应用服务时，可以使用统一 factory 返回 trait object：

```rust,no_run
use az_sms::dogsms::client::DogSmsConfig;
use az_sms::provider::{SmsProviderConfig, build_sms_provider};

fn provider(api_key: String) -> anyhow::Result<az_sms::provider::BoxSmsProvider> {
    let config = DogSmsConfig::builder(api_key).build()?;
    build_sms_provider(SmsProviderConfig::from(config))
}
```

如果上层服务需要依赖注入，依赖 `SmsProviderFactory` trait 即可，默认实现是 `BuiltinSmsProviderFactory`。

## Live Test

普通测试不会访问真实 SMS provider：

```bash
cargo test -p az-sms
```

DogSMS 的 7 个编号测试文件是 live API 测试，默认 `#[ignore]`，需要真实 `DOGSMS_API_KEY` 后点名执行：

```bash
DOGSMS_API_KEY='...' cargo test -p az-sms --test dogsms_01_balance -- --ignored --nocapture
DOGSMS_API_KEY='...' cargo test -p az-sms --test dogsms_02_services -- --ignored --nocapture
DOGSMS_API_KEY='...' cargo test -p az-sms --test dogsms_03_inventory -- --ignored --nocapture
```

会创建或变更订单的接口需要确认余额和库存后再运行：

```bash
DOGSMS_API_KEY='...' cargo test -p az-sms --test dogsms_04_create_activation -- --ignored --nocapture
DOGSMS_API_KEY='...' DOGSMS_EXISTING_REQUEST_ID='...' cargo test -p az-sms --test dogsms_05_activation_detail -- --ignored --nocapture
DOGSMS_API_KEY='...' DOGSMS_CANCEL_REQUEST_ID='...' cargo test -p az-sms --test dogsms_06_cancel_activation -- --ignored --nocapture
DOGSMS_API_KEY='...' cargo test -p az-sms --test dogsms_07_create_rental -- --ignored --nocapture
```

## API 入口

常用类型：

- `az_sms::dogsms::client::DogSmsClient`
- `az_sms::dogsms::client::DogSmsConfig`
- `az_sms::dogsms::client::DogSmsActivationRequest`
- `az_sms::dogsms::client::DogSmsRentalRequest`
- `az_sms::grizzlysms::client::GrizzlySmsClient`
- `az_sms::grizzlysms::client::GrizzlySmsConfig`
- `az_sms::provider::SmsProvider`
- `az_sms::provider::SmsProviderKind`
- `az_sms::provider::SmsProviderConfig`
- `az_sms::provider::SmsProviderFactory`
- `az_sms::provider::BuiltinSmsProviderFactory`
- `az_sms::provider::build_sms_provider`
- `az_sms::model::SmsActivationRequest`
- `az_sms::model::SmsHostingRequest`
- `az_sms::model::SmsOrder`
- `az_sms::model::SmsMessage`
- `az_sms::model::WaitForSmsOptions`
- 所有可失败 API 统一返回 `anyhow::Result`

`SmsProvider` 是通用 trait，后续接入其他短信服务商时应实现这个 trait，而不是让调用方直接绑定某个供应商。
