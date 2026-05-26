use az_browser_automation::{
    BrowserAutomationContextStore, BrowserAutomationOptions, BrowserMode, CdpEndpoint,
    OpenAiAuthAutomation, OpenAiRecordingOptions, OpenAiRecordingStep, normalize_cdp_http_url,
};
use std::time::Duration;

#[test]
#[ignore = "manual recording test: requires a running CDP Chrome session and human interaction"]
fn openai_login_page_opens_for_manual_recording() {
    BrowserAutomationContextStore::clear();

    let options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        slow_mo_ms: 300,
        timeout_ms: 120_000,
        mode: BrowserMode::Cdp(CdpEndpoint::Http(normalize_cdp_http_url("127.0.0.1:9222"))),
        ..BrowserAutomationOptions::default()
    };

    let recording_options = manual_openai_recording_options();
    BrowserAutomationContextStore::set_start_url(&recording_options.start_url);
    print_recording_plan(&recording_options);

    let result = OpenAiAuthAutomation::record(&recording_options, &options)
        .expect("should run the OpenAI entry recording plan and keep the tab available");
    for step in result.steps {
        eprintln!(
            "{} {} => {:?} | {} | {}",
            step.step.code(),
            step.step.to_string(),
            step.status,
            step.final_url,
            step.message,
        );
    }
}

fn manual_openai_recording_options() -> OpenAiRecordingOptions {
    let mut options = OpenAiRecordingOptions::entry_sign_up();

    if let Some(steps) = parse_recording_steps() {
        options = options.with_steps(steps);
    }

    if let Ok(start_url) = std::env::var("OPENAI_RECORD_START_URL") {
        options = options.with_start_url(start_url);
    }

    if let Some(step_delay) = parse_duration_var("OPENAI_RECORD_STEP_DELAY_MS") {
        options = options.with_step_delay(step_delay);
    }

    if let Some(hold_after_each_step) = parse_duration_var("OPENAI_RECORD_STEP_HOLD_SECS") {
        options = options.with_hold_after_each_step(hold_after_each_step);
    }

    let hold_after_finish =
        parse_duration_var("OPENAI_RECORD_HOLD_SECS").unwrap_or_else(|| Duration::from_secs(600));
    options.with_hold_after_finish(hold_after_finish)
}

fn parse_recording_steps() -> Option<Vec<OpenAiRecordingStep>> {
    let raw = std::env::var("OPENAI_RECORD_STEPS").ok()?;
    let steps = raw
        .split(',')
        .filter_map(OpenAiRecordingStep::from_id)
        .collect::<Vec<_>>();
    if steps.is_empty() { None } else { Some(steps) }
}

fn parse_duration_var(name: &str) -> Option<Duration> {
    let raw = std::env::var(name).ok()?;
    match name {
        "OPENAI_RECORD_STEP_DELAY_MS" => raw.parse::<u64>().ok().map(Duration::from_millis),
        _ => raw.parse::<u64>().ok().map(Duration::from_secs),
    }
}

fn print_recording_plan(options: &OpenAiRecordingOptions) {
    eprintln!("OpenAI recording plan: {}", options.start_url);
    for step in &options.steps {
        eprintln!(
            "{} {} - {}",
            step.code(),
            step.to_string(),
            step.description()
        );
    }
}
