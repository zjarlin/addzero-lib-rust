# az-codex-auth-support

Safe Rust support code extracted from `codex_auto_register`.

Implemented:

- DuckMail API client for domains, accounts, tokens, messages, and polling.
- Six-digit email verification-code extraction.
- RFC 7636 PKCE helper generation.
- CLIProxyAPI-compatible Codex auth-file JSON shaping from existing OAuth tokens.
- Optional multipart auth-file upload to a management endpoint.

Intentionally not implemented:

- Automated OpenAI or ChatGPT account registration.
- Sentinel proof-of-work reverse engineering.
- Browser fingerprint impersonation.
- Proxy-based risk-control bypass flows.
- Bulk third-party OAuth token generation.

Those flows were present in the Python source project, but they are not appropriate
to preserve as a runnable Rust automation tool. Use this crate only for legitimate
mailbox access and local auth-file handling where you already control the account
and token source.
