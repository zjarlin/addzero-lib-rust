# az-oauth2

Provider-neutral OAuth2 helpers for installed apps, CLI tools, and limited-input device flows.

The crate implements reusable OAuth mechanics only:

- PKCE verifier/challenge generation
- authorization-code URL construction
- loopback redirect listener for desktop/CLI apps
- authorization code exchange
- refresh token exchange
- device authorization and polling

Provider-specific behavior belongs in thin config helpers. Google support here is limited to official endpoint/scope constants and a config constructor.

## Gmail readonly token example

```rust,no_run
use az_oauth2::{
    AuthorizationCodeOptions, GoogleOAuth2, OAuth2Client, OAuth2Config,
};

# fn run() -> az_oauth2::OAuth2Result<()> {
let config = GoogleOAuth2::installed_app("google-client-id")
    .scope(GoogleOAuth2::GMAIL_READONLY_SCOPE)
    .build()?;
let client = OAuth2Client::new(config)?;

let session = client.begin_loopback_authorization(
    AuthorizationCodeOptions::new()
        .access_type("offline")
        .prompt("consent"),
)?;

println!("Open: {}", session.authorization_url);
let callback = session.wait_for_callback(std::time::Duration::from_secs(300))?;
let token = client.exchange_authorization_code(
    &callback.code,
    &callback.redirect_uri,
    Some(&session.pkce),
)?;

println!("{}", token.require_access_token()?);
# Ok(())
# }
```

Use the returned access token with `az-gmail-code`.
