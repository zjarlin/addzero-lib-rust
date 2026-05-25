# az-sms

`az-sms` 是一个可复用的短信服务 crate，当前内置 5sim v1 和 Grizzly SMS API client。

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
az-sms = { path = "../../network/az-sms" }
```

从仓库外部引用当前本地 checkout：

```toml
[dependencies]
az-sms = { path = "/Users/zjarlin/IdeaProjects/zjarlin/addzero-lib-rust/crates/network/az-sms" }
```

## 准备 5sim Token

1. 登录 [5sim.net](https://5sim.net)。
2. 打开个人 profile。
3. 找到 `Get API key` 或 API token 入口。
4. 复制 token。
5. 不要把 token 写进源码、README、测试文件或 Git。

推荐放到环境变量：

```bash
export FIVESIM_TOKEN='your-5sim-token'
```

可以先用 curl 验证 token：

```bash
curl "https://5sim.net/v1/user/profile" \
  -H "Authorization: Bearer $FIVESIM_TOKEN" \
  -H "Accept: application/json"
```

## 查询 Profile

```rust,no_run
use az_sms::fivesim::FivesimClient;

#[tokio::main]
async fn main() -> az_sms::error::SmsResult<()> {
    let token = std::env::var("FIVESIM_TOKEN")
        .expect("FIVESIM_TOKEN is required");

    let client = FivesimClient::from_token(token)?;
    let profile = client.profile().await?;

    println!("id: {}", profile.id);
    println!("balance: {}", profile.balance);
    Ok(())
}
```

## 购买一次性号码并等待短信

```rust,no_run
use az_sms::fivesim::FivesimClient;
use az_sms::model::SmsActivationRequest;
use az_sms::provider::SmsProvider;

#[tokio::main]
async fn main() -> az_sms::error::SmsResult<()> {
    let token = std::env::var("FIVESIM_TOKEN")
        .expect("FIVESIM_TOKEN is required");

    let client = FivesimClient::from_token(token)?;
    let request = SmsActivationRequest::new("usa", "any", "telegram")?;
    let order = client.buy_activation_number(request).await?;

    println!("order id: {}", order.id);
    println!("phone: {}", order.phone);

    let order = match client.wait_for_sms(order.id, Default::default()).await {
        Ok(order) => order,
        Err(error) => {
            let _ = client.cancel_order(order.id).await;
            return Err(error);
        }
    };

    if let Some(message) = order.sms.first() {
        println!("text: {}", message.text);
        println!("code: {:?}", message.code);
    }

    client.finish_order(order.id).await?;
    Ok(())
}
```

`SmsActivationRequest::new(country, operator, product)` 三个参数对应 5sim 的国家、运营商和产品名：

- `country` 示例：`usa`
- `operator` 示例：`any`
- `product` 示例：`telegram`

具体可用值以 5sim 后台和 5sim API 文档为准。

## 购买托管号码

```rust,no_run
use az_sms::fivesim::FivesimClient;
use az_sms::model::SmsHostingRequest;
use az_sms::provider::SmsProvider;

#[tokio::main]
async fn main() -> az_sms::error::SmsResult<()> {
    let token = std::env::var("FIVESIM_TOKEN")
        .expect("FIVESIM_TOKEN is required");

    let client = FivesimClient::from_token(token)?;
    let order = client
        .buy_hosting_number(SmsHostingRequest::new("usa", "any", "3hours")?)
        .await?;

    println!("order id: {}", order.id);
    println!("phone: {}", order.phone);

    let inbox = client.inbox(order.id).await?;
    println!("messages: {}", inbox.total);

    Ok(())
}
```

## 自定义配置

```rust,no_run
use az_sms::fivesim::{FivesimClient, FivesimConfig};
use std::time::Duration;

fn build_client(token: String) -> az_sms::error::SmsResult<FivesimClient> {
    let config = FivesimConfig::builder(token)
        .request_timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("my-app/0.1.0")
        .build()?;

    FivesimClient::new(config)
}
```

默认 base URL 是：

```text
https://5sim.net/v1/
```

只有在测试代理、私有网关或 5sim API 版本迁移时才需要覆盖 `base_url`。

## Grizzly SMS

Grizzly SMS 使用 `api_key` 查询参数，API 形状兼容 sms-activate。一次性号码用同一个 `SmsProvider` trait：

```rust,no_run
use az_sms::grizzlysms::GrizzlySmsClient;
use az_sms::model::SmsActivationRequest;
use az_sms::provider::SmsProvider;

#[tokio::main]
async fn main() -> az_sms::error::SmsResult<()> {
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

Grizzly SMS 官方文档当前没有公开托管/租号 inbox API，因此 `buy_hosting_number` 和 `inbox` 会返回 `SmsError::UnsupportedOperation`。

## Live Test

默认测试不会访问 5sim，也不需要 token：

```bash
cargo test -p az-sms
```

如果要验证真实 token 和 5sim 认证链路，显式运行 ignored live test：

```bash
export FIVESIM_TOKEN='your-5sim-token'
cargo test -p az-sms --test live_fivesim -- --ignored
```

当前 live test 只调用 `profile()`，不会购买号码，也不会扣费。

## API 入口

常用类型：

- `az_sms::fivesim::FivesimClient`
- `az_sms::fivesim::FivesimConfig`
- `az_sms::grizzlysms::GrizzlySmsClient`
- `az_sms::grizzlysms::GrizzlySmsConfig`
- `az_sms::provider::SmsProvider`
- `az_sms::model::SmsActivationRequest`
- `az_sms::model::SmsHostingRequest`
- `az_sms::model::SmsOrder`
- `az_sms::model::SmsMessage`
- `az_sms::model::WaitForSmsOptions`
- `az_sms::error::SmsError`
- `az_sms::error::SmsResult`

`SmsProvider` 是通用 trait，后续接入其他短信服务商时应实现这个 trait，而不是让调用方直接绑定某个供应商。
