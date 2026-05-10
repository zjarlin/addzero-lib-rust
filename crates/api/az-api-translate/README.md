# az-api-translate

支持多提供商的翻译 API 客户端，内置 MyMemory 免费翻译实现。

## 功能

- 定义通用 [`TranslateClient`] trait，对不同翻译服务统一调用方式
- 内置 MyMemory 免费翻译客户端（`MyMemoryClient`）
- 语言代码遵循 ISO 639-1 标准（如 `en`、`zh-CN`、`ja`）
- 支持带额外选项的翻译请求（`TranslateOptions`）
- 自动语言检测能力（`DetectedLanguage`）
- 完善的错误处理：网络错误、语言对不支持、文本过长、认证失败、速率限制等

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-api-translate = { path = "../az-api-translate" }   # workspace 内部引用
# 或发布后：
# az-api-translate = "0.1"                              # crates.io 引用
```

## 用法

```rust
use az_api_translate::{MyMemoryClient, TranslateClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MyMemoryClient::new("user@example.com");
    let result = client.translate("Hello, world!", "en", "zh-CN").await?;
    println!("{}", result.translated_text);
    Ok(())
}
```

## 依赖的 crates

无外部 workspace 内部 crate 依赖；使用 `reqwest`、`serde`、`tokio`、`urlencoding` 等通用库。
