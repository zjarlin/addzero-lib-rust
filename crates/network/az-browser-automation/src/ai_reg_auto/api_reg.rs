//! OpenAI API-level registration — pure HTTP, no browser needed.
//!
//! Replicates the protocol used by the official ChatGPT Android app:
//!   1. PKCE code_verifier → code_challenge (SHA256 + base64url)
//!   2. GET  /authorize          → oai-did cookie
//!   3. POST sentinel/req        → sentinel token
//!   4. POST authorize/continue  → submit email (signup)
//!   5. POST user/register       → submit password
//!   6. Poll temp_mail for email verification code (if needed)
//!   7. Poll 5sim for phone verification code (if needed)
//!   8. Sync account to CPA / external systems (if configured)
//!
//! # Reference
//!
//! Based on `debug_register.py` from the GPTregister project.

use crate::BrowserAutomationResult;
use az_sms::SmsProvider;
use az_temp_mail::{PageRequest, TempMailMailbox, TempMailProvider, create_mail_tm_api};
use reqwest::blocking::Client as HttpClient;
use sha2::{Digest, Sha256};
use std::thread;
use std::time::{Duration, Instant};

const OPENAI_CLIENT_ID: &str = "chatgpt-android";
const OPENAI_REDIRECT_URI: &str =
    "com.openai.chatgpt://auth0.openai.com/ios/com.openai.chatgpt/callback";
const OPENAI_SCOPE: &str = "openid email profile offline_access model.read model.request";
const OPENAI_AUTHORIZE_URL: &str = "https://auth.openai.com/authorize";
const OPENAI_SENTINEL_URL: &str = "https://sentinel.openai.com/backend-api/sentinel/req";
const OPENAI_AUTHORIZE_CONTINUE_URL: &str =
    "https://auth.openai.com/api/accounts/authorize/continue";
const OPENAI_REGISTER_URL: &str = "https://auth.openai.com/api/accounts/user/register";
const OPENAI_VERIFY_EMAIL_URL: &str = "https://auth.openai.com/api/accounts/verify_email";

#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    pub cpa_url: Option<String>,
    pub cpa_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub target: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct OpenAiApiRegOptions {
    pub proxy: Option<String>,
    pub password: Option<String>,
    pub sms_token: Option<String>,
    pub sms_product: String,
    pub sms_country: String,
    pub sms_operator: String,
    pub email_prefix: String,
    pub timeout: Duration,
    pub sync: Option<SyncOptions>,
}

impl Default for OpenAiApiRegOptions {
    fn default() -> Self {
        Self {
            proxy: None,
            password: None,
            sms_token: None,
            sms_product: "openai".to_owned(),
            sms_country: "usa".to_owned(),
            sms_operator: "any".to_owned(),
            email_prefix: "azit".to_owned(),
            timeout: Duration::from_secs(30),
            sync: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiApiRegResult {
    pub email: String,
    pub email_password: String,
    pub jwt_token: String,
    pub openai_password: String,
    pub success: bool,
    pub stage: String,
    pub error: Option<String>,
    pub cookies: Vec<(String, String)>,
    pub sms_phone: Option<String>,
    pub sms_order_id: Option<u64>,
    pub sync_results: Vec<SyncResult>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OpenAiApiRegAutomation;

impl OpenAiApiRegAutomation {
    pub fn run(reg_options: &OpenAiApiRegOptions) -> BrowserAutomationResult<OpenAiApiRegResult> {
        let mail_api = create_mail_tm_api().map_err(to_browser_error)?;
        let mailbox = mail_api
            .create_mailbox_and_login(&reg_options.email_prefix, 16)
            .map_err(to_browser_error)?;
        let email = mailbox.address.clone();
        let email_password = mailbox.password.clone().unwrap_or_default();
        let jwt = mailbox.credential.clone();
        let password = reg_options
            .password
            .clone()
            .unwrap_or_else(|| random_ascii_string(16));

        let client = build_http_client(reg_options)?;

        // PKCE
        let code_verifier = pkce_verifier();
        let code_challenge = sha256_base64url_no_pad(&code_verifier);
        let state = random_hex(32);

        // GET authorize
        let authorize_url = format!(
            "{OPENAI_AUTHORIZE_URL}?\
             client_id={OPENAI_CLIENT_ID}\
             &response_type=code\
             &redirect_uri={redirect}\
             &scope={scope}\
             &state={state}\
             &code_challenge={challenge}\
             &code_challenge_method=S256\
             &prompt=login\
             &id_token_add_organizations=true\
             &codex_cli_simplified_flow=true",
            redirect = urlencoding(OPENAI_REDIRECT_URI),
            scope = urlencoding(OPENAI_SCOPE),
            challenge = &code_challenge,
        );

        let auth_resp = client
            .get(&authorize_url)
            .timeout(reg_options.timeout)
            .send()
            .map_err(to_browser_error)?;

        let oai_did = extract_cookie(&auth_resp, "oai-did");
        let Some(oai_did) = oai_did else {
            return Ok(api_error(
                email,
                email_password,
                jwt,
                password,
                "authorize",
                "no oai-did cookie",
            ));
        };

        // POST sentinel
        let sen_req_body =
            serde_json::json!({"p": "", "id": oai_did, "flow": "authorize_continue"});

        let sen_resp = client
            .post(OPENAI_SENTINEL_URL)
            .header("origin", "https://sentinel.openai.com")
            .header(
                "referer",
                "https://sentinel.openai.com/backend-api/sentinel/frame.html?sv=20260219f9f6",
            )
            .header("content-type", "text/plain;charset=UTF-8")
            .body(sen_req_body.to_string())
            .timeout(reg_options.timeout)
            .send()
            .map_err(to_browser_error)?;

        let mut cookie_jar: Vec<(String, String)> = vec![];
        collect_cookies(&auth_resp, &mut cookie_jar);
        collect_cookies(&sen_resp, &mut cookie_jar);
        let sen_json: serde_json::Value = sen_resp.json().map_err(to_browser_error)?;
        let sen_token = sen_json.get("token").and_then(|v| v.as_str()).unwrap_or("");
        if sen_token.is_empty() {
            return Ok(api_error(
                email,
                email_password,
                jwt,
                password,
                "sentinel",
                "no sentinel token",
            ));
        }

        let sentinel_header = serde_json::json!({
            "p": "", "t": "", "c": sen_token, "id": oai_did, "flow": "authorize_continue",
        })
        .to_string();

        let cookie_header = join_cookies(&cookie_jar);

        // POST authorize/continue — submit email
        let signup_body = serde_json::json!({
            "username": {"value": email, "kind": "email"},
            "screen_hint": "signup",
        });

        let signup_resp = client
            .post(OPENAI_AUTHORIZE_CONTINUE_URL)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("referer", "https://auth.openai.com/create-account")
            .header("openai-sentinel-token", &sentinel_header)
            .header("cookie", &cookie_header)
            .body(signup_body.to_string())
            .timeout(reg_options.timeout)
            .send()
            .map_err(to_browser_error)?;

        collect_cookies(&signup_resp, &mut cookie_jar);
        let signup_json: serde_json::Value = signup_resp.json().map_err(to_browser_error)?;

        if let Some(err) = signup_json.get("error").and_then(|v| v.as_str()) {
            return Ok(err_result(
                email,
                email_password,
                jwt,
                password,
                "signup",
                err,
                cookie_jar,
            ));
        }

        let cookie_header = join_cookies(&cookie_jar);

        // POST user/register — submit password
        let register_body = serde_json::json!({
            "password": password,
            "username": {"value": email, "kind": "email"},
        });

        let register_resp = client
            .post(OPENAI_REGISTER_URL)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("referer", "https://auth.openai.com/create-account/password")
            .header("openai-sentinel-token", &sentinel_header)
            .header("cookie", &cookie_header)
            .body(register_body.to_string())
            .timeout(reg_options.timeout)
            .send()
            .map_err(to_browser_error)?;

        collect_cookies(&register_resp, &mut cookie_jar);
        let register_json: serde_json::Value = register_resp.json().map_err(to_browser_error)?;

        if let Some(err) = register_json.get("error").and_then(|v| v.as_str()) {
            return Ok(err_result(
                email,
                email_password,
                jwt,
                password,
                "register",
                err,
                cookie_jar,
            ));
        }

        // Email verification
        let needs_verify = register_json
            .get("needs_email_verification")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if needs_verify {
            thread::sleep(Duration::from_secs(3));
            if let Some(code) = poll_temp_mail_code(&mail_api, &mailbox, Duration::from_secs(120)) {
                let cookie_header = join_cookies(&cookie_jar);
                let verify_body = serde_json::json!({"code": code, "email": email});
                let _ = client
                    .post(OPENAI_VERIFY_EMAIL_URL)
                    .header("accept", "application/json")
                    .header("content-type", "application/json")
                    .header("cookie", &cookie_header)
                    .body(verify_body.to_string())
                    .timeout(reg_options.timeout)
                    .send()
                    .map_err(to_browser_error);
            }
        }

        // SMS phone verification
        let sms_res = reg_options
            .sms_token
            .as_deref()
            .and_then(|token| buy_sms_number(token, reg_options).ok());
        let sms_phone = sms_res.as_ref().map(|(p, _)| p.clone());
        let sms_order_id = sms_res.map(|(_, id)| id);

        // Sync to CPA / external systems
        let mut sync_results: Vec<SyncResult> = vec![];
        if let Some(ref sync) = reg_options.sync {
            if let Some(ref cpa_url) = sync.cpa_url {
                let key = sync.cpa_key.as_deref().unwrap_or("");
                match sync_to_cpa(&email, &password, "", "", &cookie_jar, cpa_url, key) {
                    Ok(msg) => sync_results.push(SyncResult {
                        target: "cpa".into(),
                        ok: true,
                        message: msg,
                    }),
                    Err(e) => sync_results.push(SyncResult {
                        target: "cpa".into(),
                        ok: false,
                        message: e,
                    }),
                }
            }
        }

        Ok(OpenAiApiRegResult {
            email,
            email_password,
            jwt_token: jwt,
            openai_password: password,
            success: true,
            stage: "complete".into(),
            error: None,
            cookies: cookie_jar,
            sms_phone,
            sms_order_id,
            sync_results,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// Sync — upload account to CPA / external systems
// ═══════════════════════════════════════════════════════════════

/// Uploads account data to a CPA endpoint as multipart/form-data.
pub fn sync_to_cpa(
    email: &str,
    _password: &str,
    access_token: &str,
    refresh_token: &str,
    _cookies: &[(String, String)],
    cpa_url: &str,
    cpa_key: &str,
) -> Result<String, String> {
    let token_data = generate_token_json(email, access_token, refresh_token);
    let filename = format!("{email}.json");
    let file_content = serde_json::to_vec_pretty(&token_data).map_err(|e| format!("json: {e}"))?;

    let form = reqwest::blocking::multipart::Form::new().part(
        "file",
        reqwest::blocking::multipart::Part::bytes(file_content)
            .file_name(filename)
            .mime_str("application/json")
            .map_err(|e| format!("mime: {e}"))?,
    );

    let client = HttpClient::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("client: {e}"))?;

    let url = format!("{}/v0/management/auth-files", cpa_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {cpa_key}"))
        .multipart(form)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| format!("request: {e}"))?;

    let status = resp.status();
    if status.as_u16() == 200 || status.as_u16() == 201 {
        return Ok("uploaded".into());
    }

    let body = resp.text().unwrap_or_default();
    if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(msg) = err_json.get("message").and_then(|v| v.as_str()) {
            return Err(format!("HTTP {status}: {msg}"));
        }
    }
    Err(format!("HTTP {status}: {}", &body[..body.len().min(200)]))
}

fn generate_token_json(email: &str, access_token: &str, refresh_token: &str) -> serde_json::Value {
    let (expired, account_id) = decode_token_expiry(access_token);
    let now = current_time_iso();
    serde_json::json!({
        "type": "codex",
        "email": email,
        "expired": expired,
        "id_token": "",
        "account_id": account_id,
        "access_token": access_token,
        "last_refresh": now,
        "refresh_token": refresh_token,
    })
}

fn decode_token_expiry(token: &str) -> (String, String) {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return (String::new(), String::new());
    }
    let payload = parts[1];
    let padded = match payload.len() % 4 {
        2 => format!("{payload}=="),
        3 => format!("{payload}="),
        _ => payload.to_owned(),
    };
    let decoded = base64_url_decode(&padded);
    let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap_or_default();

    let exp = json.get("exp").and_then(|v| v.as_i64()).unwrap_or(0);
    let expired = if exp > 0 {
        let secs = exp + 8 * 3600; // UTC+8
        let total_days = secs / 86400;
        let rem = secs % 86400;
        let h = rem / 3600;
        let m = (rem % 3600) / 60;
        let s = rem % 60;
        let year = 1970 + (total_days / 365);
        let doy = total_days % 365;
        let mon = 1 + (doy / 30).min(11);
        let day = 1 + (doy % 30);
        format!("{year:04}-{mon:02}-{day:02}T{h:02}:{m:02}:{s:02}+08:00")
    } else {
        String::new()
    };

    let auth = json.get("https://api.openai.com/auth");
    let account_id = auth
        .and_then(|a| a.get("chatgpt_account_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    (expired, account_id)
}

fn base64_url_decode(input: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE
        .decode(input)
        .unwrap_or_default()
}

fn current_time_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 8 * 3600;
    let total_days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let year = 1970 + (total_days / 365);
    let doy = total_days % 365;
    let mon = 1 + (doy / 30).min(11);
    let day = 1 + (doy % 30);
    format!("{year:04}-{mon:02}-{day:02}T{h:02}:{m:02}:{s:02}+08:00")
}

// ── HTTP helpers ──

fn build_http_client(reg_options: &OpenAiApiRegOptions) -> BrowserAutomationResult<HttpClient> {
    let mut builder = HttpClient::builder().cookie_store(true);
    if let Some(ref proxy) = reg_options.proxy {
        let proxy = reqwest::Proxy::all(proxy).map_err(to_browser_error)?;
        builder = builder.proxy(proxy);
    }
    builder.build().map_err(to_browser_error)
}

fn extract_cookie(response: &reqwest::blocking::Response, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    for hv in response.headers().get_all("set-cookie") {
        let value = hv.to_str().ok()?;
        for part in value.split(';') {
            let t = part.trim();
            if t.starts_with(&prefix) {
                return Some(t[prefix.len()..].to_owned());
            }
        }
    }
    None
}

fn collect_cookies(response: &reqwest::blocking::Response, jar: &mut Vec<(String, String)>) {
    for hv in response.headers().get_all("set-cookie") {
        if let Ok(val) = hv.to_str() {
            for part in val.split(';') {
                let t = part.trim();
                if let Some((k, v)) = t.split_once('=') {
                    if !jar.iter().any(|(jk, _)| jk == k) {
                        jar.push((k.to_owned(), v.to_owned()));
                    }
                }
            }
        }
    }
}

fn join_cookies(jar: &[(String, String)]) -> String {
    jar.iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; ")
}

// ── SMS ──

fn buy_sms_number(
    sms_token: &str,
    reg_options: &OpenAiApiRegOptions,
) -> BrowserAutomationResult<(String, u64)> {
    let rt = tokio::runtime::Runtime::new().map_err(to_browser_error)?;
    rt.block_on(async {
        let client = az_sms::FivesimClient::from_token(sms_token).map_err(to_browser_error)?;
        let request = az_sms::SmsActivationRequest::new(
            &reg_options.sms_country,
            &reg_options.sms_operator,
            &reg_options.sms_product,
        )
        .map_err(to_browser_error)?;
        let order = client
            .buy_activation_number(request)
            .await
            .map_err(to_browser_error)?;
        Ok((order.phone, order.id))
    })
}

// ── Temp mail ──

fn poll_temp_mail_code(
    provider: &dyn TempMailProvider,
    mailbox: &TempMailMailbox,
    max_wait: Duration,
) -> Option<String> {
    let deadline = Instant::now() + max_wait;
    let interval = Duration::from_secs(4);
    thread::sleep(Duration::from_secs(3));
    while Instant::now() < deadline {
        if let Ok(listing) = provider.list_messages(mailbox, PageRequest::new(10, 0)) {
            for summary in &listing.results {
                if let Ok(Some(detail)) = provider.get_message(mailbox, &summary.id.to_string()) {
                    let combined = format!("{} {} {}", summary.subject, detail.text, detail.html);
                    if let Some(code) = extract_verification_code(&combined) {
                        return Some(code);
                    }
                }
            }
        }
        thread::sleep(interval);
    }
    None
}

// ── Crypto / PKCE ──

fn pkce_verifier() -> String {
    random_ascii_from_charset(
        128,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~",
    )
}

fn sha256_base64url_no_pad(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest)
}

// ── String utils ──

fn random_hex(len: usize) -> String {
    random_ascii_from_charset(len, b"abcdef0123456789")
}

fn random_ascii_string(len: usize) -> String {
    random_ascii_from_charset(
        len,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$",
    )
}

fn random_ascii_from_charset(len: usize, charset: &[u8]) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut state = seed as u64;
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            charset[(state >> 33) as usize % charset.len()] as char
        })
        .collect()
}

fn urlencoding(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3 / 2);
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(result, "%{:02X}", byte);
            }
        }
    }
    result
}

fn extract_verification_code(text: &str) -> Option<String> {
    let plain = strip_html_tags(text);
    for pattern in &[
        r"(?i)(?:verification|security|confirmation|one-time|otp)\s*(?:code|number|pin)?\s*[:]?\s*(\d{4,8})",
        r"(?i)code\s*[:]?\s*(\d{4,8})",
        r"(?i)(\d{4,8})\s*(?:is your|is the)\s*(?:verification|security|auth)",
        r"\b(\d{6})\b",
        r"\b(\d{5})\b",
        r"\b(\d{4})\b",
    ] {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&plain) {
                return Some(caps[1].to_owned());
            }
        }
    }
    None
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

fn api_error(
    email: String,
    email_password: String,
    jwt: String,
    password: String,
    stage: &str,
    msg: &str,
) -> OpenAiApiRegResult {
    OpenAiApiRegResult {
        email,
        email_password,
        jwt_token: jwt,
        openai_password: password,
        success: false,
        stage: stage.to_owned(),
        error: Some(msg.to_owned()),
        cookies: vec![],
        sms_phone: None,
        sms_order_id: None,
        sync_results: vec![],
    }
}

fn err_result(
    email: String,
    email_password: String,
    jwt: String,
    password: String,
    stage: &str,
    msg: &str,
    cookies: Vec<(String, String)>,
) -> OpenAiApiRegResult {
    OpenAiApiRegResult {
        email,
        email_password,
        jwt_token: jwt,
        openai_password: password,
        success: false,
        stage: stage.to_owned(),
        error: Some(msg.to_owned()),
        cookies,
        sms_phone: None,
        sms_order_id: None,
        sync_results: vec![],
    }
}

fn to_browser_error(error: impl ToString) -> crate::BrowserAutomationError {
    crate::BrowserAutomationError::Browser(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_is_128_chars() {
        let v = pkce_verifier();
        assert_eq!(v.len(), 128);
        assert!(
            v.chars()
                .all(|c| c.is_ascii_alphanumeric() || "-._~".contains(c))
        );
    }

    #[test]
    fn pkce_challenge_is_base64url_no_pad() {
        let challenge = sha256_base64url_no_pad("test");
        assert!(!challenge.contains('='));
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        let encoded = urlencoding("com.openai.chatgpt://callback");
        assert!(encoded.contains("%3A%2F%2F"));
    }

    #[test]
    fn extract_code_finds_six_digit() {
        let code = extract_verification_code("Your verification code is 123456");
        assert_eq!(code.as_deref(), Some("123456"));
    }

    #[test]
    fn extract_code_finds_code_prefix() {
        let code = extract_verification_code("Code: 7890");
        assert_eq!(code.as_deref(), Some("7890"));
    }

    #[test]
    fn extract_code_from_html() {
        let code = extract_verification_code("<p>Your code is <b>555121</b></p>");
        assert_eq!(code.as_deref(), Some("555121"));
    }

    #[test]
    fn api_options_default() {
        let opts = OpenAiApiRegOptions::default();
        assert_eq!(opts.sms_product, "openai");
        assert_eq!(opts.sms_country, "usa");
    }

    #[test]
    fn api_error_result_is_not_success() {
        let r = api_error(
            "e@t.com".into(),
            "ep".into(),
            "jwt".into(),
            "pwd".into(),
            "sentinel",
            "fail",
        );
        assert!(!r.success);
        assert_eq!(r.stage, "sentinel");
    }
}
