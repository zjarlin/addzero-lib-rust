# tool-creates

把 `addzero-lib-jvm/lib/tool-jvm/network-call` 里适合公开、适合沉淀为 Rust 工具库的常见 HTTP API，统一收口成一个可复用的创建器 crate。

当前优先落了两组常见能力：

- Maven Central 检索与文件下载
- mail.tm 临时邮箱

## 添加依赖

如果你在这个 workspace 里直接使用：

```toml
[dependencies]
tool-creates = { path = "../tool-creates" }
```

如果你从仓库外部引用当前本地 checkout：

```toml
[dependencies]
tool-creates = { path = "/absolute/path/to/addzero-lib-rust/crates/tool-creates" }
```

## 基础用法

```rust
use tool_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maven = Creates::maven_central()?;
    let latest = maven.get_latest_version("com.google.guava", "guava")?;
    println!("latest guava version: {latest:?}");

    let temp_mail = Creates::temp_mail()?;
    let mailbox = temp_mail.create_mailbox_and_login("demo", 12)?;
    println!("mailbox: {}", mailbox.address);

    Ok(())
}
```

## Maven Central

```rust
use tool_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::maven_central()?;

    let artifacts = api.search_by_group_id("com.google.guava", 5)?;
    for artifact in artifacts {
        println!(
            "{}:{} -> {:?}",
            artifact.group_id,
            artifact.artifact_id,
            artifact.resolved_version()
        );
    }

    let pom = api.download_file(
        "com.google.guava",
        "guava",
        "33.2.1-jre",
        "guava-33.2.1-jre.pom",
    )?;
    println!("downloaded {} bytes", pom.len());
    Ok(())
}
```

已封装的方法：

- `search_by_group_id`
- `search_by_artifact_id`
- `search_by_coordinates`
- `search_all_versions`
- `search_by_full_coordinates`
- `search_by_class_name`
- `search_by_fully_qualified_class_name`
- `search_by_sha1`
- `search_by_tag`
- `search_by_keyword`
- `get_latest_version`
- `get_latest_version_by_group_id`
- `download_file`

## Temp Mail

```rust
use tool_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::temp_mail()?;

    let mailbox = api.create_mailbox_and_login("az", 12)?;
    println!("address: {}", mailbox.address);

    let messages = api.list_messages(&mailbox.token, 1)?;
    for message in messages {
        println!("message: {} {}", message.id, message.subject);
    }

    Ok(())
}
```

已封装的方法：

- `get_domains`
- `create_mailbox_and_login`
- `create_account`
- `create_token`
- `list_messages`
- `get_message`

## 自定义配置

```rust
use std::time::Duration;
use tool_creates::{ApiConfig, Creates};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ApiConfig::builder("https://search.maven.org")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(15))
        .user_agent("my-app/1.0.0")
        .build()?;

    let api = Creates::maven_central_with_config(config)?;
    let versions = api.search_all_versions("com.google.guava", "guava", 20)?;
    println!("versions: {}", versions.len());
    Ok(())
}
```

## 范围说明

这次没有直接把 `network-call` 全量照搬到 Rust：

- 浏览器自动化、支付、私有供应商接入这类模块，不适合在当前仓库做一层“看起来统一、实际不可复用”的硬迁移
- `tool-api-weather` 这类依赖特定站点抓取细节和 cookie 的实现，稳定性不足，不适合作为通用公开 API 默认暴露
- 后续如果确认某个接口长期稳定，再按 crate 继续补进 `tool-creates`
