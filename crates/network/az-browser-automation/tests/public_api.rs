use az_browser_automation::*;

#[test]
fn debug_mode_forces_headful_browser() {
    let options = BrowserAutomationOptions {
        debug: true,
        headless: true,
        ..BrowserAutomationOptions::default()
    };

    // Debug sessions must stay visible even when callers also set headless.
    assert!(!options.effective_headless());
}

#[test]
fn context_store_round_trip_keeps_start_url() {
    BrowserAutomationContextStore::clear();
    BrowserAutomationContextStore::set_start_url("www.baidu.com");

    // Bare hosts must be normalized so downstream browser navigation gets a valid URL.
    assert_eq!(
        BrowserAutomationContextStore::start_url().as_deref(),
        Some("https://www.baidu.com")
    );

    BrowserAutomationContextStore::clear();
    // Clearing the global store must isolate later automation runs and tests.
    assert!(BrowserAutomationContextStore::get().is_none());
}

#[test]
fn form_field_builders_default_to_expected_field_types() {
    let input = FormFieldDef::input("keyword", ["input[name='wd']"], "rust").required(true);
    let click = FormFieldDef::click("search", ["#su"]);
    let check = FormFieldDef::check("remember", ["#remember"]);

    // The input shortcut must keep producing the variant consumed by text entry.
    assert_eq!(input.field_type, FieldType::Input);
    // The fluent required flag is part of the public builder contract.
    assert!(input.required);
    // The click shortcut must not drift into a generic input field.
    assert_eq!(click.field_type, FieldType::Click);
    // The check shortcut must keep mapping to checkbox/toggle handling.
    assert_eq!(check.field_type, FieldType::Check);
}

#[test]
fn browser_defaults_to_cdp_mode() {
    let options = BrowserAutomationOptions::default();

    // Existing callers rely on the local CDP endpoint without passing options.
    assert_eq!(
        options.mode,
        BrowserMode::Cdp(CdpEndpoint::Http("http://127.0.0.1:9222".to_owned()))
    );
}

#[test]
fn normalize_cdp_http_url_adds_scheme_and_trims_slash() {
    // Bare CDP endpoints must become comparable absolute HTTP URLs.
    assert_eq!(
        normalize_cdp_http_url("127.0.0.1:9222/"),
        "http://127.0.0.1:9222"
    );
    // Already absolute endpoints must only lose trailing separators.
    assert_eq!(
        normalize_cdp_http_url("http://localhost:9333/"),
        "http://localhost:9333"
    );
}

#[test]
fn cdp_port_parser_extracts_port_from_http_url() {
    // Port extraction drives the browser process attachment path.
    assert_eq!(parse_cdp_port("http://127.0.0.1:9222"), Some(9222));
    // A trailing slash from copied DevTools URLs must not break parsing.
    assert_eq!(parse_cdp_port("http://localhost:9333/"), Some(9333));
}

#[test]
fn fingerprint_random_profile_uses_supported_desktop_platform() {
    let profile = FingerprintProfile::random();

    // Randomized fingerprints must stay within platform values accepted by browsers.
    assert!(matches!(
        profile.platform.as_str(),
        "Win32" | "MacIntel" | "Linux x86_64"
    ));
}

#[test]
fn fingerprint_pool_templates_are_distinct() {
    let unique = fingerprint::SELECTION_POOL
        .iter()
        .map(|profile| {
            (
                profile.user_agent,
                profile.platform,
                profile.viewport,
                profile.timezone,
            )
        })
        .collect::<std::collections::HashSet<_>>();

    // Duplicate templates reduce fingerprint diversity and make randomization weaker.
    assert_eq!(unique.len(), fingerprint::SELECTION_POOL.len());
}

#[test]
fn proxy_url_parser_supports_socks5_credentials() -> BrowserAutomationResult<()> {
    let proxy = ProxyConfig::from_url("socks5://user:pass@localhost:1080")?;

    // SOCKS5 URLs must select the proxy backend required by browser launch flags.
    assert_eq!(proxy.proxy_type, ProxyType::Socks5);
    // Host parsing must strip userinfo and scheme before connection setup.
    assert_eq!(proxy.host, "localhost");
    // Port parsing must preserve the explicit endpoint chosen by the caller.
    assert_eq!(proxy.port, 1080);
    // Username extraction must keep authenticated proxy URLs usable.
    assert_eq!(proxy.username.as_deref(), Some("user"));
    // Password extraction must remain paired with the parsed username.
    assert_eq!(proxy.password.as_deref(), Some("pass"));
    Ok(())
}

#[test]
fn registration_code_parser_extracts_six_digit_code() {
    // Registration flows depend on extracting the canonical six-digit challenge code.
    assert_eq!(
        extract_verification_code("Use 654321 to continue.").as_deref(),
        Some("654321")
    );
}

#[test]
fn openai_auth_options_support_manual_recording_defaults() {
    let options = OpenAiAuthOptions::open_login().with_hold_for(std::time::Duration::from_secs(5));

    // Manual recording should still work without embedding real credentials in tests.
    assert_eq!(options.flow, OpenAiAuthFlow::Login);
    assert_eq!(options.start_url, OpenAiAuthOptions::LOGIN_URL);
    assert_eq!(options.hold_for, Some(std::time::Duration::from_secs(5)));
}

#[test]
fn openai_auth_result_marks_only_authenticated_as_complete() {
    let result = OpenAiAuthResult {
        stage: OpenAiAuthStage::VerificationRequired,
        final_url: "https://auth.openai.com/log-in".to_owned(),
        page_title: "OpenAI".to_owned(),
        message: "manual verification required".to_owned(),
    };

    // Verification and CAPTCHA states must stay explicit manual stops.
    assert!(!result.is_complete());
}

#[test]
fn openai_recording_defaults_to_step1_step2_step3() {
    let options = OpenAiRecordingOptions::entry_sign_up();

    // Manual recording should expose a stable entry-flow sequence for incremental debugging.
    assert_eq!(
        options.steps,
        vec![
            OpenAiRecordingStep::OpenEntryPage,
            OpenAiRecordingStep::ClickLogin,
            OpenAiRecordingStep::ClickSignUp,
        ]
    );
}

#[test]
fn openai_recording_step_parser_accepts_step_ids() {
    // Step ids are used by the manual recording test to run partial plans.
    assert_eq!(
        OpenAiRecordingStep::from_id("step3"),
        Some(OpenAiRecordingStep::ClickSignUp)
    );
}
