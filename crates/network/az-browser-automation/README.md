# az-browser-automation

面向表单工作流和授权注册测试的浏览器自动化辅助工具。

## 一句话说明

基于 `headless_chrome` 的浏览器自动化库，提供声明式表单填写、隔离会话管理、浏览器指纹伪装、代理配置和可扩展的注册流程框架。

## 功能

- **声明式表单填写** — 通过 `BrowserAutomation::fill` 配置 `FormFieldDef` 列表，支持 CSS / label / placeholder / role 等多种选择器，自动完成输入、点击、勾选操作。
- **隔离浏览器会话** — `BrowserSession` 为每次自动化启动独立的 Chrome 进程，分配独立的 CDP 端口和用户数据目录，Drop 时自动清理。
- **浏览器指纹管理** — `FingerprintProfile` 内置多套 Windows / macOS / Linux 的真实浏览器模板，覆盖 User-Agent、视口、语言、WebGL、Canvas 噪声、AudioContext、插件列表等，通过 CDP 注入保持一致性。
- **代理支持** — `ProxyConfig` 支持 HTTP 和 SOCKS5 代理，可从 URL 解析（含认证），也支持从代理池文件批量加载。
- **可扩展的注册流程** — `RegistrationFlow` trait 定义了多步骤注册工作流接口，已内置 Kiro（AWS Builder ID）注册流程，支持临时邮箱轮询验证码。
- **AI 平台授权自动化** — `OpenAiAuthAutomation` / `OpenAiRegAutomation` 针对 OpenAI 登录、注册、API Key 申请页面提供自动化适配，遇到验证码或 MFA 时上报手动干预阶段。
- **调试产物** — 调试模式下自动保存截图（PNG）和页面 HTML，便于排查自动化失败。

## 安装

### 作为工作空间路径依赖（monorepo 内部）

```toml
[dependencies]
az-browser-automation = { path = "crates/network/az-browser-automation" }
```

### 从 crates.io 安装

```toml
[dependencies]
az-browser-automation = "0.1"
```

## 用法

### 声明式表单填写

```rust,no_run
use az_browser_automation::browser_automation::{
    BrowserAutomation, BrowserAutomationOptions, FormFieldDef,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = BrowserAutomationOptions {
        headless: true,
        ..Default::default()
    };

    let fields = vec![
        FormFieldDef::input(
            "keyword",
            ["input[name='wd']", "input#kw"],
            "Hello Rust",
        ).required(true),
        FormFieldDef::click("search", ["input#su"]).required(true),
    ];

    BrowserAutomation::fill("https://www.baidu.com", &fields, &options, None)?;
    Ok(())
}
```

### 隔离浏览器会话

```rust,no_run
use az_browser_automation::proxy::ProxyConfig;
use az_browser_automation::session::{BrowserSession, SessionConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用随机指纹和代理启动独立会话
    let proxy = ProxyConfig::from_url("socks5://user:pass@127.0.0.1:1080")?;
    let session = BrowserSession::new(
        SessionConfig::builder()
            .proxy(proxy)
            .headless(true)
            .build(),
    )?;

    session.navigate("https://example.com")?;
    let accessibility_tree = session.snapshot()?;
    session.screenshot("/tmp/screenshot.png")?;
    // 会话结束时 Drop 自动终止 Chrome 进程并清理临时目录
    Ok(())
}
```

### 注册流程（Kiro 示例）

```rust,no_run
use az_browser_automation::ai_reg_auto::kiro::kiro::KiroRegistrationFlow;
use az_browser_automation::registration::RegistrationFlow;
use az_browser_automation::session::{BrowserSession, SessionConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = BrowserSession::new(SessionConfig::default())?;
    let flow = KiroRegistrationFlow::new();
    let result = flow.execute(&session, "test@example.com")?;

    if result.success {
        println!("注册成功，验证码: {:?}", result.verification_code);
    }

    Ok(())
}
```

## 依赖的 crates

| crate | 说明 |
|---|---|
| `headless_chrome` | Chrome DevTools Protocol 客户端，驱动浏览器操作 |
| `az-context` | 线程本地上下文存储工具 |
| `az-sms` | 短信验证码提供方抽象 |
| `az-temp-mail` | 临时邮箱收发抽象（用于注册流程验证码轮询） |
| `reqwest` | HTTP 客户端（查询 CDP 端点） |
| `serde` / `serde_json` | 序列化与反序列化 |
| `regex` | 正则匹配（验证码提取等） |
| `rand` | 随机数生成（指纹池选取、随机延迟） |
| `sha2` | 哈希计算 |
| `base64` | Base64 编解码 |
| `uuid` | 唯一标识（临时目录命名） |
| `dirs` | 平台标准目录路径 |
| `thiserror` | 错误类型派生 |
| `tokio` | 异步运行时（部分场景使用） |
| `tracing` | 结构化日志 |
