#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod installer_scanner;
pub mod installer_scanner_utils;
pub mod paths;

use az_derive_aliases::{apply, plain_default};
use az_desktop_plugin::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopRenderLayer, DesktopViewContext,
    EventPropagation, Plugin,
};
use az_desktop_plugin_registry::register_desktop_plugin;
use az_software_catalog::SoftwareCatalogDto;
use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, rgb};

use crate::installer_scanner::{InstallerPackage, organize_installers, scan_installers};

const KNOWLEDGE_DOMAIN_ID: &str = "knowledge";
const SOFTWARE_BRANCH_ID: &str = "knowledge-software";

#[apply(plain_default)]
struct SoftwareCenterPlugin {
    lines: Vec<String>,
    scanned: Vec<InstallerPackage>,
}

impl SoftwareCenterPlugin {
    const PAGE_ID: &str = "software-center";
    const ROUTE: &str = "/software";
    const ACTION_REFRESH: &str = "software-center.refresh";
    const ACTION_SCAN: &str = "software-center.scan-installers";
    const ACTION_ORGANIZE: &str = "software-center.organize-installers";

    fn refresh(&mut self, ctx: &DesktopExecContext) -> Result<(), String> {
        let catalog = ctx.services.software_catalog()?;
        let scanned = scan_installers().map_err(|err| err.to_string())?;
        self.scanned = scanned.clone();
        self.lines = build_report_lines(&catalog, &scanned);
        Ok(())
    }

    fn organize(&mut self, ctx: &DesktopExecContext) -> Result<String, String> {
        let organized = organize_installers().map_err(|err| err.to_string())?;
        let count = organized.len();
        self.refresh(ctx)?;
        Ok(format!("organized {count} installers into archive targets"))
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

fn installer_matches_catalog(package: &InstallerPackage, slug: &str, title: &str) -> bool {
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
        ctx.register_domain(KNOWLEDGE_DOMAIN_ID, "Knowledge", 20, Self::ROUTE);
        ctx.register_branch(
            SOFTWARE_BRANCH_ID,
            KNOWLEDGE_DOMAIN_ID,
            None::<String>,
            "Software",
            20,
        );
        ctx.register_page(
            Self::PAGE_ID,
            KNOWLEDGE_DOMAIN_ID,
            Some(SOFTWARE_BRANCH_ID),
            "Software Center",
            "Installer scan, organize/archive, and catalog-linked package detail surfaces.",
            Self::ROUTE,
            20,
        );
        ctx.register_toolbar_action(
            Some(Self::ROUTE),
            Self::ACTION_REFRESH,
            "Refresh",
            "Reload catalog and installer scan",
            10,
            false,
        );
        ctx.register_toolbar_action(
            Some(Self::ROUTE),
            Self::ACTION_SCAN,
            "Scan",
            "Scan Downloads and Desktop for installers",
            20,
            true,
        );
        ctx.register_toolbar_action(
            Some(Self::ROUTE),
            Self::ACTION_ORGANIZE,
            "Organize",
            "Archive detected installers into software-center storage",
            30,
            false,
        );
        ctx.register_summary_card(
            "software-center-summary",
            "Software Center",
            "Installer scan, archive flow, and az_software_catalog linkage.",
            Self::ROUTE,
            30,
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
                let result = match action_id.as_str() {
                    Self::ACTION_REFRESH => self
                        .refresh(ctx)
                        .map(|()| "software-center refreshed".to_string()),
                    Self::ACTION_SCAN => self
                        .refresh(ctx)
                        .map(|()| format!("scanned {} installers", self.scanned.len())),
                    Self::ACTION_ORGANIZE => self.organize(ctx),
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

register_desktop_plugin!(SoftwareCenterPlugin);

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
