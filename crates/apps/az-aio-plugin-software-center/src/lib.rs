#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod installer_scanner;
pub mod installer_scanner_utils;
pub mod paths;

use az_desktop_plugin::{
    DesktopActionOutcome, DesktopEvent, DesktopExecContext, DesktopInitContext,
    DesktopPageContributionSpec, DesktopRenderLayer, DesktopToolbarActionSpec, DesktopViewContext,
    EventPropagation, Plugin,
};
use az_desktop_plugin_registry::declare_desktop_plugin;
use az_software_catalog::SoftwareCatalogDto;
use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, rgb};

use crate::installer_scanner::{InstallerPackage, organize_installers, scan_installers};

const KNOWLEDGE_DOMAIN_ID: &str = "knowledge";
const SOFTWARE_BRANCH_ID: &str = "knowledge-software";

declare_desktop_plugin! {
    struct SoftwareCenterPlugin {
        lines: Vec<String>,
        scanned: Vec<InstallerPackage>,
    }
}

impl SoftwareCenterPlugin {
    const PAGE_ID: &str = "software-center";
    const ROUTE: &str = "/software";
    const ACTION_REFRESH: &str = "software-center.refresh";
    const ACTION_SCAN: &str = "software-center.scan-installers";
    const ACTION_ORGANIZE: &str = "software-center.organize-installers";
    const TOOLBAR_ACTIONS: &[DesktopToolbarActionSpec] = &[
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_REFRESH,
            "Refresh",
            "Reload catalog and installer scan",
            10,
        ),
        DesktopToolbarActionSpec::primary(
            Self::ACTION_SCAN,
            "Scan",
            "Scan Downloads and Desktop for installers",
            20,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_ORGANIZE,
            "Organize",
            "Archive detected installers into software-center storage",
            30,
        ),
    ];
    const CONTRIBUTION: DesktopPageContributionSpec = DesktopPageContributionSpec {
        domain_id: KNOWLEDGE_DOMAIN_ID,
        domain_label: "Knowledge",
        domain_order: 20,
        branch_id: SOFTWARE_BRANCH_ID,
        parent_branch_id: None,
        branch_label: "Software",
        branch_order: 20,
        page_id: Self::PAGE_ID,
        page_title: "Software Center",
        page_subtitle: "Installer scan, organize/archive, and catalog-linked package detail surfaces.",
        route: Self::ROUTE,
        page_order: 20,
        summary_card_id: "software-center-summary",
        summary_title: "Software Center",
        summary: "Installer scan, archive flow, and az_software_catalog linkage.",
        summary_order: 30,
        toolbar_actions: Self::TOOLBAR_ACTIONS,
    };

    fn refresh(&mut self, ctx: &DesktopExecContext) -> Result<(), String> {
        let catalog = ctx.services.software_catalog()?;
        let scanned = scan_installers().map_err(|err| err.to_string())?;
        self.scanned = scanned.clone();
        self.lines = build_report_lines(&catalog, &scanned);
        Ok(())
    }

    fn organize(&mut self, ctx: &DesktopExecContext) -> Result<DesktopActionOutcome, String> {
        let organized = organize_installers().map_err(|err| err.to_string())?;
        let count = organized.len();
        self.refresh(ctx)?;
        Ok(DesktopActionOutcome::notified(format!(
            "organized {count} installers into archive targets"
        )))
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
                            .child("Software Center"),
                    )
                    .child(div().text_sm().text_color(rgb(0x475467)).child(
                        "Installer scan, archive targets, and linkage into az_software_catalog.",
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

fn build_report_lines(catalog: &SoftwareCatalogDto, scanned: &[InstallerPackage]) -> Vec<String> {
    let mut lines = vec![
        format!("catalog items: {}", catalog.items.len()),
        format!("installer candidates: {}", scanned.len()),
        String::new(),
        "Catalog".to_string(),
    ];
    for item in catalog.items.iter().take(10) {
        lines.push(format!("  - {} [{}]", item.title, item.slug));
    }
    lines.push(String::new());
    lines.push("Scanned Installers".to_string());
    for package in scanned.iter().take(12) {
        let linked = catalog
            .items
            .iter()
            .find(|item| {
                installer_matches_catalog(package, item.slug.as_str(), item.title.as_str())
            })
            .map(|item| item.slug.clone())
            .unwrap_or_else(|| "unmatched".to_string());
        lines.push(format!(
            "  - {} [{} / {}] -> {}",
            package.file_name, package.platform, package.arch, linked
        ));
    }
    lines
}

/// 判断扫描到的安装包是否能匹配软件目录中的 slug 或标题。
pub fn installer_matches_catalog(package: &InstallerPackage, slug: &str, title: &str) -> bool {
    let normalized_name = package.file_name.to_ascii_lowercase();
    normalized_name.contains(&slug.to_ascii_lowercase())
        || normalized_name.contains(&title.to_ascii_lowercase().replace(' ', ""))
        || normalized_name.contains(&title.to_ascii_lowercase().replace(' ', "-"))
}

impl
    Plugin<
        DesktopInitContext,
        DesktopEvent,
        DesktopExecContext,
        DesktopViewContext,
        DesktopRenderLayer,
    > for SoftwareCenterPlugin
{
    fn name(&self) -> &'static str {
        "software-center"
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
                    .map(|()| DesktopActionOutcome::notified("software-center refreshed")),
                Self::ACTION_SCAN => self.refresh(ctx).map(|()| {
                    DesktopActionOutcome::notified(format!(
                        "scanned {} installers",
                        self.scanned.len()
                    ))
                }),
                Self::ACTION_ORGANIZE => self.organize(ctx),
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
    use crate::installer_matches_catalog;
    use crate::installer_scanner::InstallerPackage;

    #[test]
    fn links_installers_to_catalog_slugs() {
        let package = InstallerPackage {
            id: "1".to_string(),
            file_name: "raycast-1.2.3-macos-arm64.dmg".to_string(),
            source_path: String::new(),
            version: "1.2.3".to_string(),
            platform: "macOS".to_string(),
            arch: "arm64".to_string(),
            target_path: String::new(),
            install_status: "unconfirmed".to_string(),
            status: "pending".to_string(),
            md5: "x".to_string(),
        };

        assert!(installer_matches_catalog(&package, "raycast", "Raycast"));
    }
}
