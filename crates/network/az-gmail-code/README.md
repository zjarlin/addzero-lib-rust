# az-gmail-code

Authorized Gmail API client for reading verification codes from Gmail mailboxes owned by the caller.

This crate is not a bypass or third-party "receive SMS/code" service. It expects a Gmail OAuth access token for a mailbox the caller controls, then searches messages through the Gmail API and extracts short numeric verification codes from message bodies.

## Coordinate

Workspace crate:

```toml
az-gmail-code = { workspace = true }
```

## Example

```rust,no_run
use az_gmail_code::{GmailCodeClient, GmailCodeQuery};

# fn run() -> az_gmail_code::GmailCodeResult<()> {
let client = GmailCodeClient::new("ya29.access-token")?;
let code = client.find_latest_code(
    GmailCodeQuery::new()
        .from("security@example.com")
        .subject("verification")
        .newer_than("10m")
        .unread(true),
)?;

if let Some(code) = code {
    println!("{} from {}", code.code, code.message_id);
}
# Ok(())
# }
```

## Runtime Constraints

- Requires Gmail API OAuth access to the target mailbox.
- Uses Gmail's `users.messages.list` and `users.messages.get` endpoints.
- Keep OAuth scopes as narrow as the workflow allows, normally `https://www.googleapis.com/auth/gmail.readonly`.

## Getting a Gmail OAuth Access Token

OAuth is implemented separately in `az-oauth2` because it is not Gmail-specific.

```rust,no_run
use az_oauth2::{
    AuthorizationCodeOptions, GoogleOAuth2, OAuth2Client,
};

# fn run() -> Result<(), Box<dyn std::error::Error>> {
let oauth = OAuth2Client::new(
    GoogleOAuth2::installed_app("google-client-id")
        .scope(GoogleOAuth2::GMAIL_READONLY_SCOPE)
        .build()?,
)?;

let session = oauth.begin_loopback_authorization(
    AuthorizationCodeOptions::new()
        .access_type("offline")
        .prompt("consent"),
)?;

println!("Open: {}", session.authorization_url);
let callback = session.wait_for_callback(std::time::Duration::from_secs(300))?;
let token = oauth.exchange_authorization_code(
    &callback.code,
    &callback.redirect_uri,
    Some(&session.pkce),
)?;

let gmail = az_gmail_code::GmailCodeClient::new(token.require_access_token()?)?;
# Ok(())
# }
```
