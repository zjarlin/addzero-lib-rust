#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod skill_scanner;

use az_assets::{AssetKind, AssetUpsert};
use az_desktop_plugin::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopRenderLayer,
    DesktopToolbarActionSpec, DesktopViewContext, EventPropagation, Plugin,
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

    fn scan_skills(&mut self, ctx: &DesktopExecContext) -> Result<String, String> {
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
        Ok(format!("merged {total} scanned skills into az_assets"))
    }

    fn seed_compose_asset(&mut self, ctx: &DesktopExecContext) -> Result<String, String> {
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
        Ok("seeded compose asset into az_assets".to_string())
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
        ctx.register_domain(KNOWLEDGE_DOMAIN_ID, "Knowledge", 20, Self::ROUTE);
        ctx.register_branch(
            ASSET_BRANCH_ID,
            KNOWLEDGE_DOMAIN_ID,
            None::<String>,
            "Assets",
            10,
        );
        ctx.register_page(
            Self::PAGE_ID,
            KNOWLEDGE_DOMAIN_ID,
            Some(ASSET_BRANCH_ID),
            "Asset Hub",
            "Asset feed, skill scan, compose assets, and subtype-backed metadata.",
            Self::ROUTE,
            10,
        );
        ctx.register_route_toolbar_actions(
            Self::ROUTE,
            [
                DesktopToolbarActionSpec::secondary(
                    Self::ACTION_REFRESH,
                    "Refresh",
                    "Reload assets",
                    10,
                ),
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
            ],
        );
        ctx.register_summary_card(
            "asset-hub-summary",
            "Asset Hub",
            "Asset editor/feed, skill scan, compose assets, tag filters, and detail surfaces.",
            Self::ROUTE,
            20,
        );
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
                let outcome = match action_id.as_str() {
                    Self::ACTION_REFRESH => self
                        .refresh(ctx)
                        .map(|()| "asset-hub refreshed".to_string()),
                    Self::ACTION_SCAN_SKILLS => self.scan_skills(ctx),
                    Self::ACTION_SEED_COMPOSE => self.seed_compose_asset(ctx),
                    _ => Ok(String::new()),
                };
                match outcome {
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
