# az-codex-auth-support

从 `codex_auto_register` 提取的安全 Rust 支持代码。

已实现：

- DuckMail API 客户端，用于域名、账户、令牌、消息和轮询。
- 六位邮箱验证码提取。
- RFC 7636 PKCE 辅助生成。
- 兼容 CLIProxyAPI 的 Codex 认证文件 JSON 格式化（基于已有 OAuth 令牌）。
- 可选的多部分认证文件上传到管理端点。

刻意未实现：

- 自动化 OpenAI 或 ChatGPT 账户注册。
- Sentinel 工作量证明逆向工程。
- 浏览器指纹伪造。
- 基于代理的风险控制绕过流程。
- 批量第三方 OAuth 令牌生成。

上述流程存在于 Python 源项目中，但不适合作为可运行的 Rust 自动化工具保留。请仅在已拥有账户和令牌源控制权的情况下，将此 crate 用于合法的邮箱访问和本地认证文件处理。
