#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod skill_scanner;

use az_assets::{AssetKind, AssetUpsert};
use az_desktop_plugin::{
    DesktopActionOutcome, DesktopEvent, DesktopExecContext, DesktopInitContext,
    DesktopPageContributionSpec, DesktopRenderLayer, DesktopToolbarActionSpec, DesktopViewContext,
    EventPropagation, Plugin,
};
use az_desktop_plugin_registry::declare_desktop_plugin;
use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, rgb};
use serde_json::json;

use crate::skill_scanner::scan_skill_assets;

const KNOWLEDGE_DOMAIN_ID: &str = "knowledge";
const ASSET_BRANCH_ID: &str = "knowledge-assets";

declare_desktop_plugin! {
    struct AssetHubPlugin {
        lines: Vec<String>,
    }
}

impl AssetHubPlugin {
    const PAGE_ID: &str = "asset-hub";
    const ROUTE: &str = "/assets";
    const ACTION_REFRESH: &str = "asset-hub.refresh";
    const ACTION_SCAN_SKILLS: &str = "asset-hub.scan-skills";
    const ACTION_SEED_COMPOSE: &str = "asset-hub.seed-compose";
    const TOOLBAR_ACTIONS: &[DesktopToolbarActionSpec] = &[
        DesktopToolbarActionSpec::secondary(Self::ACTION_REFRESH, "Refresh", "Reload assets", 10),
        DesktopToolbarActionSpec::primary(
            Self::ACTION_SCAN_SKILLS,
            "Scan Skills",
            "Scan ~/.agents/skills and merge into az_assets",
            20,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_SEED_COMPOSE,
            "Seed Compose",
            "Insert a compose asset using stable subtype metadata",
            30,
        ),
    ];
    const CONTRIBUTION: DesktopPageContributionSpec = DesktopPageContributionSpec {
        domain_id: KNOWLEDGE_DOMAIN_ID,
        domain_label: "Knowledge",
        domain_order: 20,
        branch_id: ASSET_BRANCH_ID,
        parent_branch_id: None,
        branch_label: "Assets",
        branch_order: 10,
        page_id: Self::PAGE_ID,
        page_title: "Asset Hub",
        page_subtitle: "Asset feed, skill scan, compose assets, and subtype-backed metadata.",
        route: Self::ROUTE,
        page_order: 10,
        summary_card_id: "asset-hub-summary",
        summary_title: "Asset Hub",
        summary: "Asset editor/feed, skill scan, compose assets, tag filters, and detail surfaces.",
        summary_order: 20,
        toolbar_actions: Self::TOOLBAR_ACTIONS,
    };

    fn refresh(&mut self, ctx: &DesktopExecContext) -> Result<(), String> {
        let assets = ctx.services.list_assets(None)?;
        let mut lines = vec![
            format!("assets: {}", assets.len()),
            String::new(),
            "Kinds".to_string(),
        ];

        for kind in [
            AssetKind::Capture,
            AssetKind::Note,
            AssetKind::Skill,
            AssetKind::Software,
            AssetKind::Package,
        ] {
            let count = assets.iter().filter(|asset| asset.kind == kind).count();
            lines.push(format!("  - {}: {}", kind.as_str(), count));
        }

        lines.push(String::new());
        lines.push("Recent".to_string());
        for asset in assets.iter().take(12) {
            let subtype = asset
                .metadata
                .get("subtype")
                .and_then(|value| value.as_str())
                .unwrap_or("-");
            lines.push(format!(
                "  - {} [{} / {}] tags={}",
                asset.title,
                asset.kind.as_str(),
                subtype,
                asset.tags.join(",")
            ));
        }

        self.lines = lines;
        Ok(())
    }

    fn scan_skills(&mut self, ctx: &DesktopExecContext) -> Result<DesktopActionOutcome, String> {
        let skills = scan_skill_assets().map_err(|err| err.to_string())?;
        let total = skills.len();
        for skill in skills {
            ctx.services.upsert_asset(AssetUpsert {
                id: None,
                kind: AssetKind::Skill,
                title: skill.name,
                body: skill.content,
                tags: skill.tags,
                status: "active".to_string(),
                metadata: json!({
                    "subtype": "skill",
                    "source": skill.source,
                    "origin": skill.origin,
                    "asset_type": skill.asset_type,
                    "systems": skill.systems,
                }),
            })?;
        }
        self.refresh(ctx)?;
        Ok(DesktopActionOutcome::notified(format!(
            "merged {total} scanned skills into az_assets"
        )))
    }

    fn seed_compose_asset(
        &mut self,
        ctx: &DesktopExecContext,
    ) -> Result<DesktopActionOutcome, String> {
        ctx.services.upsert_asset(AssetUpsert {
            id: None,
            kind: AssetKind::Package,
            title: "demo-compose.yaml".to_string(),
            body: "services:\n  demo:\n    image: nginx:latest\n    ports:\n      - \"8080:80\"\n"
                .to_string(),
            tags: vec!["compose".to_string(), "demo".to_string()],
            status: "active".to_string(),
            metadata: json!({
                "subtype": "compose",
                "source": "asset-hub",
                "origin": "seed",
            }),
        })?;
        self.refresh(ctx)?;
        Ok(DesktopActionOutcome::notified(
            "seeded compose asset into az_assets",
        ))
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
                            .child("Asset Hub"),
                    )
                    .child(div().text_sm().text_color(rgb(0x475467)).child(
                        "Assets, skill scan merge, compose assets, and subtype-backed metadata.",
                    )),
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
    > for AssetHubPlugin
{
    fn name(&self) -> &'static str {
        "asset-hub"
    }

    fn setup(&mut self, ctx: &mut DesktopInitContext) {
        ctx.register_page_contribution(Self::CONTRIBUTION);
    }

    fn on_event(&mut self, event: &DesktopEvent, ctx: &mut DesktopExecContext) -> EventPropagation {
        if matches!(event, DesktopEvent::Startup) {
            let _ = self.refresh(ctx);
        } else if event.refreshes_route(Self::ROUTE) || event.is_global_refresh() {
            if let Err(err) = self.refresh(ctx) {
                ctx.notify(err);
            }
        } else if let Some(action_id) = event.action_id_for_route(Self::ROUTE) {
            let outcome = match action_id {
                Self::ACTION_REFRESH => self
                    .refresh(ctx)
                    .map(|()| DesktopActionOutcome::notified("asset-hub refreshed")),
                Self::ACTION_SCAN_SKILLS => self.scan_skills(ctx),
                Self::ACTION_SEED_COMPOSE => self.seed_compose_asset(ctx),
                _ => Ok(DesktopActionOutcome::Ignored),
            };
            return ctx.apply_action_outcome(outcome);
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
    use az_assets::{AssetKind, AssetService, AssetUpsert};
    use serde_json::json;

    #[test]
    fn compose_assets_use_stable_subtype_metadata() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let service = AssetService::memory_only(None);
            let saved = service
                .upsert_asset(AssetUpsert {
                    id: None,
                    kind: AssetKind::Package,
                    title: "demo-compose.yaml".to_string(),
                    body: "services: {}".to_string(),
                    tags: vec!["compose".to_string()],
                    status: "active".to_string(),
                    metadata: json!({ "subtype": "compose" }),
                })
                .await
                .unwrap();

            assert_eq!(saved.kind, AssetKind::Package);
            assert_eq!(
                saved
                    .metadata
                    .get("subtype")
                    .and_then(|value| value.as_str()),
                Some("compose")
            );
        });
    }
}
