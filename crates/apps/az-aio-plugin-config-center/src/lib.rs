#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub mod dotfiles_monitor;
pub mod dotfiles_monitor_diff;
pub mod dotfiles_monitor_types;
pub mod pairing;
pub mod paths;

use std::env;

use az_ai_agent::default_model_for;
use az_assets::{AiModelProviderUpsert, AiProviderKind};
use az_desktop_plugin::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopPageContributionSpec,
    DesktopRenderLayer, DesktopToolbarActionSpec, DesktopViewContext, EventPropagation, Plugin,
};
use az_desktop_plugin_registry::declare_desktop_plugin;
use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, rgb};

use crate::{
    dotfiles_monitor::scan_dotfiles_status,
    pairing::{ensure_local_pairing_device_info, local_pairing_info},
    paths::resolve_config_center_paths,
};

const ENV_DOMAIN_ID: &str = "environment";
const MACHINE_BRANCH_ID: &str = "environment-machine";

declare_desktop_plugin! {
    struct ConfigCenterPlugin {
        lines: Vec<String>,
        last_test_lines: Vec<String>,
    }
}

impl ConfigCenterPlugin {
    const PAGE_ID: &str = "config-center";
    const ROUTE: &str = "/config";
    const ACTION_REFRESH: &str = "config-center.refresh";
    const ACTION_IMPORT_ENV: &str = "config-center.import-env-providers";
    const ACTION_TEST_OPENAI: &str = "config-center.test-openai";
    const ACTION_TEST_ANTHROPIC: &str = "config-center.test-anthropic";
    const ACTION_TEST_GEMINI: &str = "config-center.test-gemini";
    const TOOLBAR_ACTIONS: &[DesktopToolbarActionSpec] = &[
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_REFRESH,
            "Refresh",
            "Reload dotfiles, pairing, paths, and providers",
            10,
        ),
        DesktopToolbarActionSpec::primary(
            Self::ACTION_IMPORT_ENV,
            "Import Env",
            "Import provider secrets from environment variables",
            20,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_TEST_OPENAI,
            "Test OpenAI",
            "Test OpenAI provider connectivity",
            30,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_TEST_ANTHROPIC,
            "Test Anthropic",
            "Test Anthropic provider connectivity",
            40,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_TEST_GEMINI,
            "Test Gemini",
            "Test Gemini provider connectivity",
            50,
        ),
    ];
    const CONTRIBUTION: DesktopPageContributionSpec = DesktopPageContributionSpec {
        domain_id: ENV_DOMAIN_ID,
        domain_label: "Environment",
        domain_order: 30,
        branch_id: MACHINE_BRANCH_ID,
        parent_branch_id: None,
        branch_label: "Machine",
        branch_order: 10,
        page_id: Self::PAGE_ID,
        page_title: "Config Center",
        page_subtitle: "Dotfiles monitor, pairing identity, XDG paths, and model provider configuration.",
        route: Self::ROUTE,
        page_order: 10,
        summary_card_id: "config-center-summary",
        summary_title: "Config Center",
        summary: "Dotfiles conflict audit, pairing identity, XDG/backend panels, and provider config/testing.",
        summary_order: 40,
        toolbar_actions: Self::TOOLBAR_ACTIONS,
    };

    fn refresh(&mut self, ctx: &DesktopExecContext) -> Result<(), String> {
        ensure_local_pairing_device_info().map_err(|err| err.to_string())?;
        let dotfiles = scan_dotfiles_status().map_err(|err| err.to_string())?;
        let pairing = local_pairing_info().map_err(|err| err.to_string())?;
        let xdg_paths = resolve_config_center_paths().map_err(|err| err.to_string())?;
        let providers = ctx.services.list_provider_configs()?;

        let mut lines = vec![
            "Dotfiles".to_string(),
            format!("  - root: {}", dotfiles.root),
            format!("  - watched: {}", dotfiles.watched_files),
            format!("  - changed: {}", dotfiles.changed_files),
            format!("  - conflicts: {}", dotfiles.conflict_files),
            format!("  - devices: {}", dotfiles.devices.len()),
            String::new(),
            "Pairing".to_string(),
            format!("  - device: {}", pairing.device_name),
            format!("  - fingerprint: {}", pairing.fingerprint),
            format!("  - metadata: {}", pairing.metadata_path),
            String::new(),
            "XDG".to_string(),
            format!("  - data: {}", xdg_paths.data_dir),
            format!("  - config: {}", xdg_paths.config_dir),
            format!("  - state: {}", xdg_paths.state_dir),
            format!("  - cache: {}", xdg_paths.cache_dir),
            String::new(),
            "Providers".to_string(),
        ];

        for provider in providers {
            lines.push(format!(
                "  - {} enabled={} key={} model={} base={}",
                provider.provider.as_str(),
                provider.enabled,
                provider.api_key_configured,
                provider.default_model,
                provider.base_url.unwrap_or_else(|| "-".to_string())
            ));
        }
        if !self.last_test_lines.is_empty() {
            lines.push(String::new());
            lines.push("Latest Test".to_string());
            lines.extend(self.last_test_lines.iter().cloned());
        }

        self.lines = lines;
        Ok(())
    }

    fn import_env_providers(&mut self, ctx: &DesktopExecContext) -> Result<String, String> {
        let imported = [
            (
                AiProviderKind::OpenAi,
                env::var("OPENAI_API_KEY").ok(),
                env::var("OPENAI_BASE_URL").ok(),
            ),
            (
                AiProviderKind::Anthropic,
                env::var("ANTHROPIC_API_KEY").ok(),
                env::var("ANTHROPIC_BASE_URL").ok(),
            ),
            (
                AiProviderKind::Gemini,
                env::var("GEMINI_API_KEY")
                    .ok()
                    .or_else(|| env::var("GOOGLE_API_KEY").ok()),
                env::var("GEMINI_BASE_URL").ok(),
            ),
        ]
        .into_iter()
        .filter_map(|(provider, api_key, base_url)| {
            api_key
                .filter(|value| !value.trim().is_empty())
                .map(|api_key| (provider, api_key, base_url))
        })
        .map(|(provider, api_key, base_url)| {
            ctx.services.upsert_provider(AiModelProviderUpsert {
                provider,
                base_url: base_url.filter(|value| !value.trim().is_empty()),
                default_model: default_model_for(provider).to_string(),
                enabled: true,
                api_key: Some(api_key),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

        self.refresh(ctx)?;
        Ok(format!(
            "imported {} provider configs from env",
            imported.len()
        ))
    }

    fn test_provider(
        &mut self,
        provider: AiProviderKind,
        ctx: &DesktopExecContext,
    ) -> Result<String, String> {
        let result = ctx.services.test_provider(provider)?;
        self.last_test_lines = vec![
            format!("  - provider: {}", result.provider),
            format!("  - ok: {}", result.ok),
            format!("  - message: {}", result.message),
        ];
        self.refresh(ctx)?;
        Ok(format!("tested {}", provider.as_str()))
    }

    fn render_report(&self) -> AnyElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0xf8fafc))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child("Config Center"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x475467))
                            .child("Dotfiles, pairing identity, XDG paths, backend capabilities, and AI provider configuration."),
                    ),
            )
            .children(self.lines.iter().map(|line| {
                div()
                    .text_sm()
                    .text_color(rgb(0x101828))
                    .child(line.clone())
            }))
            .into_any_element()
    }
}

impl
    Plugin<
        DesktopInitContext,
        DesktopEvent,
        DesktopExecContext,
        DesktopViewContext,
        DesktopRenderLayer,
    > for ConfigCenterPlugin
{
    fn name(&self) -> &'static str {
        "config-center"
    }

    fn setup(&mut self, ctx: &mut DesktopInitContext) {
        ctx.register_page_contribution(Self::CONTRIBUTION);
    }

    fn on_event(&mut self, event: &DesktopEvent, ctx: &mut DesktopExecContext) -> EventPropagation {
        match event {
            DesktopEvent::Startup => {
                let _ = self.refresh(ctx);
            }
            DesktopEvent::RouteChanged { route }
            | DesktopEvent::RefreshRequested { route: Some(route) }
                if route == Self::ROUTE =>
            {
                if let Err(err) = self.refresh(ctx) {
                    ctx.notify(err);
                }
            }
            DesktopEvent::RefreshRequested { route: None } => {
                if let Err(err) = self.refresh(ctx) {
                    ctx.notify(err);
                }
            }
            DesktopEvent::ActionInvoked { route, action_id } if route == Self::ROUTE => {
                let result = match action_id.as_str() {
                    Self::ACTION_REFRESH => self
                        .refresh(ctx)
                        .map(|()| "config-center refreshed".to_string()),
                    Self::ACTION_IMPORT_ENV => self.import_env_providers(ctx),
                    Self::ACTION_TEST_OPENAI => self.test_provider(AiProviderKind::OpenAi, ctx),
                    Self::ACTION_TEST_ANTHROPIC => {
                        self.test_provider(AiProviderKind::Anthropic, ctx)
                    }
                    Self::ACTION_TEST_GEMINI => self.test_provider(AiProviderKind::Gemini, ctx),
                    _ => Ok(String::new()),
                };
                match result {
                    Ok(message) if !message.is_empty() => {
                        ctx.notify(message);
                        return EventPropagation::Stop;
                    }
                    Ok(_) => {}
                    Err(err) => {
                        ctx.notify(err);
                        return EventPropagation::Stop;
                    }
                }
            }
            _ => {}
        }
        EventPropagation::Continue
    }

    fn render(&mut self, ctx: &mut DesktopViewContext) -> Option<AnyElement> {
        (ctx.shell.current_route == Self::ROUTE).then(|| self.render_report())
    }

    fn priority(&self) -> i32 {
        100
    }

    fn render_layer(&self) -> DesktopRenderLayer {
        DesktopRenderLayer::Main
    }
}

#[cfg(test)]
mod tests {
    use az_assets::{AiModelProviderUpsert, AiProviderKind, AssetService, SecretCipher};

    #[test]
    fn provider_config_round_trip_preserves_secret_state() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let cipher = SecretCipher::from_master_key("0123456789abcdef0123456789abcdef").unwrap();
            let service = AssetService::memory_only(Some(cipher));
            let saved = service
                .upsert_provider(AiModelProviderUpsert {
                    provider: AiProviderKind::OpenAi,
                    base_url: Some("https://api.openai.com/v1".to_string()),
                    default_model: "gpt-4.1-mini".to_string(),
                    enabled: true,
                    api_key: Some("sk-demo".to_string()),
                })
                .await
                .unwrap();

            assert!(saved.api_key_configured);
            assert_eq!(saved.default_model, "gpt-4.1-mini");
        });
    }
}
