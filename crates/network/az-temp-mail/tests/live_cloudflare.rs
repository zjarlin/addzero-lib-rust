use az_temp_mail::{CloudflareTempMailContext, PageRequest, create_mail_tm_api};
use std::env;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
#[ignore = "requires AZ_TEMP_MAIL_CLOUDFLARE_BASE_URL and a disposable live worker"]
fn live_cloudflare_worker_create_login_list_and_optional_cleanup() -> Result<(), Box<dyn Error>> {
    let context = LiveCloudflareContext::from_env()?;
    let api = context.cloudflare.create_api()?;
    let settings = api.open_settings()?;
    assert!(!settings.version.trim().is_empty());

    let address = api.new_address(&context.cloudflare.new_address_request())?;
    assert!(address.address.contains('@'));
    assert!(!address.jwt.trim().is_empty());

    assert!(api.credential_login(&address.jwt)?);

    let address_settings = api.address_settings(&address.jwt)?;
    assert_eq!(address_settings.address, address.address);

    let inbox = api.list_parsed_mails(&address.jwt, PageRequest::new(10, 0))?;
    assert!(inbox.count >= inbox.results.len() as u64);

    if context.delete_address {
        let deleted = api.delete_address(&address.jwt)?;
        assert!(deleted.success);
    }

    Ok(())
}

#[test]
#[ignore = "touches public mail.tm service"]
fn live_mail_tm_create_and_list() -> Result<(), Box<dyn Error>> {
    let api = create_mail_tm_api()?;
    let mailbox = api.create_mailbox_and_login("azit", 16)?;
    assert!(mailbox.address.contains('@'));
    assert!(!mailbox.credential.trim().is_empty());

    let inbox = api.list_messages_by_token(&mailbox.credential, PageRequest::default())?;
    assert!(inbox.count >= inbox.results.len() as u64);

    Ok(())
}

fn required_env(name: &str) -> Result<String, Box<dyn Error>> {
    env::var(name).map_err(|_| format!("{name} is required for this live integration test").into())
}

#[derive(Debug, Clone)]
struct LiveCloudflareContext {
    cloudflare: CloudflareTempMailContext,
    delete_address: bool,
}

impl LiveCloudflareContext {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let address_name = env_optional_non_blank("AZ_TEMP_MAIL_CLOUDFLARE_NAME")
            .unwrap_or(format!("aztest{}", unix_timestamp()?));

        Ok(Self {
            cloudflare: CloudflareTempMailContext {
                base_url: required_env("AZ_TEMP_MAIL_CLOUDFLARE_BASE_URL")?,
                custom_auth: env_optional_non_blank("AZ_TEMP_MAIL_CLOUDFLARE_CUSTOM_AUTH"),
                address_name: Some(address_name),
                address_domain: env_optional_non_blank("AZ_TEMP_MAIL_CLOUDFLARE_DOMAIN"),
                cf_token: env_optional_non_blank("AZ_TEMP_MAIL_CLOUDFLARE_CF_TOKEN"),
                enable_random_subdomain: env_optional_bool(
                    "AZ_TEMP_MAIL_CLOUDFLARE_RANDOM_SUBDOMAIN",
                ),
            },
            delete_address: env_bool("AZ_TEMP_MAIL_CLOUDFLARE_DELETE_ADDRESS"),
        })
    }
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn env_optional_non_blank(name: &str) -> Option<String> {
    env::var(name).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn env_optional_bool(name: &str) -> Option<bool> {
    env::var(name).ok().map(|value| {
        matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
}

fn unix_timestamp() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
