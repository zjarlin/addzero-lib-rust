//! 真实调用 mail.tm 创建临时邮箱，打印用户名、密码、JWT token 等完整登录态。
//!
//! ```bash
//! cargo run -p az-temp-mail --example create_and_show
//! ```

use az_derive_aliases::{apply, deserialize_debug, plain_debug};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 创建临时邮箱 (mail.tm) ===\n");

    let client = Client::builder().build()?;

    // Step 1: 获取域名列表
    println!("[1/3] 获取可用域名...");
    let resp = client.get("https://api.mail.tm/domains").send()?;
    let raw_text = resp.text()?;
    println!("  响应: {}...", &raw_text[..raw_text.len().min(300)]);

    let domains_resp: DomainsResponse = serde_json::from_str(&raw_text)?;
    let domains = &domains_resp.hydra_member;
    println!("  获取到 {} 个域名", domains.len());
    for d in domains {
        println!(
            "    - {} (active={}, private={})",
            d.domain, d.is_active, d.is_private
        );
    }

    let domain = domains
        .iter()
        .find(|d| d.is_active && !d.is_private)
        .ok_or("没有可用的活跃域名")?;

    let local_part = format!("azit{}", random_str(8));
    let address = format!("{local_part}@{}", domain.domain);
    let password = random_str(16);
    println!("  生成邮箱: {}", address);

    // Step 2: 创建账号
    println!("\n[2/3] 创建账号...");
    let resp = client
        .post("https://api.mail.tm/accounts")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "address": &address,
            "password": &password,
        }))
        .send()?;
    let raw_text = resp.text()?;
    println!("  响应: {}...", &raw_text[..raw_text.len().min(300)]);

    let account_resp: AccountResponse = serde_json::from_str(&raw_text)?;
    let account_id = &account_resp.id;
    println!("  账号 ID: {}", account_id);

    // Step 3: 创建 token
    println!("\n[3/3] 获取 JWT Token...");
    let resp = client
        .post("https://api.mail.tm/token")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "address": &address,
            "password": &password,
        }))
        .send()?;
    let raw_text = resp.text()?;
    println!("  响应: {}...", &raw_text[..raw_text.len().min(300)]);

    let token_resp: TokenResponse = serde_json::from_str(&raw_text)?;
    let jwt = &token_resp.token;
    println!("  JWT 前半段: {}...", &jwt[..jwt.len().min(60)]);

    // ===== 汇总 =====
    println!("\n╔══════════════════════════════════════════╗");
    println!("║        临时邮箱登录态信息                  ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║ 📧 邮箱地址: {:<30}║", address);
    println!("║ 🔑 密码:     {:<30}║", password);
    println!("║ 🪪 JWT Token:                            ║");
    // 分行打印 JWT
    for chunk in jwt.as_bytes().chunks(40) {
        let line = std::str::from_utf8(chunk).unwrap_or("(invalid utf8)");
        println!("║     {:<40}║", line);
    }
    println!("║ 🆔 Account ID: {:<28}║", account_id);
    println!("╚══════════════════════════════════════════╝");

    // ===== 检查收件箱 =====
    println!("\n📬 当前收件箱:");
    match check_inbox(&client, jwt) {
        Ok(messages) => {
            println!("  共 {} 封邮件", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                println!("  [{i}] from={} subject={}", msg.from_address, msg.subject);
            }
            if messages.is_empty() {
                println!("  (收件箱为空，你可以现在发一封测试邮件到 {address})");
            }
        }
        Err(e) => println!("  ⚠️ 检查收件箱失败: {e}"),
    }

    // ===== 轮询新邮件 =====
    println!("\n⏳ 轮询等待新邮件（最多 120s，Ctrl-C 退出）...");
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(120);
    let poll_interval = Duration::from_secs(5);
    let initial_count = match check_inbox(&client, jwt) {
        Ok(msgs) => msgs.len(),
        Err(_) => 0,
    };

    while start.elapsed() < timeout {
        match check_inbox(&client, jwt) {
            Ok(current) => {
                let new_count = current.len() as i64 - initial_count as i64;
                if new_count > 0 {
                    println!("\n🎉 收到 {} 封新邮件！", new_count);
                    for msg in &current {
                        // 获取详情
                        match get_message_detail(&client, jwt, &msg.id) {
                            Ok(detail) => {
                                println!("──────────────────────────────────────");
                                println!("ID:      {}", detail.id);
                                println!("From:    {}", detail.from);
                                println!("Subject: {}", detail.subject);
                                println!("Date:    {}", detail.created_at);
                                if !detail.text.is_empty() {
                                    println!("Text:\n{}", detail.text);
                                }
                                if !detail.html.is_empty() {
                                    println!(
                                        "HTML (前300字):\n{}",
                                        &detail.html[..detail.html.len().min(300)]
                                    );
                                }
                                println!("──────────────────────────────────────");
                            }
                            Err(e) => println!("  ⚠️ 获取详情失败: {e}"),
                        }
                    }
                    break;
                }
                let elapsed = start.elapsed().as_secs();
                println!(
                    "  [{elapsed}s] 收件箱仍为 {} 封，5s 后重试...",
                    current.len()
                );
            }
            Err(e) => {
                let elapsed = start.elapsed().as_secs();
                println!("  [{elapsed}s] 检查收件箱出错: {e}");
            }
        }
        thread::sleep(poll_interval);
    }

    if start.elapsed() >= timeout {
        println!("\n⏰ 超时，未收到新邮件。你仍可向 {address} 发信。");
    }

    println!("\n=== 登录态（可保存复用）===");
    println!("邮箱:  {address}");
    println!("密码:  {password}");
    println!("JWT:   {jwt}");

    Ok(())
}

fn random_str(len: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut state = seed as u64;
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            chars[(state >> 33) as usize % chars.len()] as char
        })
        .collect()
}

#[apply(deserialize_debug)]
struct DomainsResponse {
    #[serde(rename = "hydra:member")]
    hydra_member: Vec<DomainItem>,
}

#[apply(deserialize_debug)]
struct DomainItem {
    domain: String,
    #[serde(rename = "isActive")]
    is_active: bool,
    #[serde(rename = "isPrivate")]
    is_private: bool,
}

#[apply(deserialize_debug)]
struct AccountResponse {
    id: String,
}

#[apply(deserialize_debug)]
struct TokenResponse {
    token: String,
}

#[apply(plain_debug)]
struct MessageSummary {
    id: String,
    from_address: String,
    subject: String,
}

#[apply(plain_debug)]
struct MessageDetail {
    id: String,
    from: String,
    subject: String,
    text: String,
    html: String,
    created_at: String,
}

fn check_inbox(client: &Client, jwt: &str) -> Result<Vec<MessageSummary>, Box<dyn Error>> {
    let resp = client
        .get("https://api.mail.tm/messages?page=1")
        .bearer_auth(jwt)
        .send()?;
    let raw = resp.text()?;

    #[derive(Deserialize)]
    struct HydraMessages {
        #[serde(rename = "hydra:member", default)]
        member: Vec<RawMessage>,
    }
    #[derive(Deserialize)]
    struct RawMessage {
        #[serde(default)]
        id: String,
        from: RawSender,
        #[serde(default)]
        subject: String,
    }
    #[derive(Deserialize)]
    struct RawSender {
        #[serde(default)]
        address: String,
    }

    let parsed: HydraMessages = serde_json::from_str(&raw)?;
    Ok(parsed
        .member
        .into_iter()
        .map(|m| MessageSummary {
            id: m.id,
            from_address: m.from.address,
            subject: m.subject,
        })
        .collect())
}

fn get_message_detail(
    client: &Client,
    jwt: &str,
    message_id: &str,
) -> Result<MessageDetail, Box<dyn Error>> {
    let resp = client
        .get(&format!("https://api.mail.tm/messages/{message_id}"))
        .bearer_auth(jwt)
        .send()?;
    let raw = resp.text()?;

    #[derive(Deserialize)]
    struct RawDetail {
        #[serde(default)]
        id: String,
        from: RawSender2,
        #[serde(default)]
        subject: String,
        #[serde(default)]
        text: String,
        #[serde(default)]
        html: serde_json::Value,
        #[serde(rename = "createdAt", default)]
        created_at: String,
    }
    #[derive(Deserialize)]
    struct RawSender2 {
        #[serde(default)]
        address: String,
        #[serde(default)]
        name: String,
    }

    let parsed: RawDetail = serde_json::from_str(&raw)?;
    let html = match &parsed.html {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    };

    Ok(MessageDetail {
        id: parsed.id,
        from: format!("{} <{}>", parsed.from.name, parsed.from.address),
        subject: parsed.subject,
        text: parsed.text,
        html,
        created_at: parsed.created_at,
    })
}
