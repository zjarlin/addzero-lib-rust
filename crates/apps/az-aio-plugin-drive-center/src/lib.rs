#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use az_desktop_plugin::{
    DesktopEvent, DesktopExecContext, DesktopHostServices, DesktopInitContext,
    DesktopPageContributionSpec, DesktopRenderLayer, DesktopToolbarActionSpec, DesktopViewContext,
    EventPropagation, Plugin,
};
use az_desktop_plugin_registry::declare_desktop_plugin;
use gpui::{AnyElement, IntoElement, div, prelude::*, rgb};

const OPS_DOMAIN_ID: &str = "operations";
const STORAGE_BRANCH_ID: &str = "operations-storage";

declare_desktop_plugin! {
    struct DriveCenterPlugin {
        lines: Vec<String>,
    }
}

impl DriveCenterPlugin {
    const PAGE_ID: &str = "drive-center";
    const ROUTE: &str = "/drive";
    const ACTION_REFRESH: &str = "drive.refresh";
    const ACTION_SYNC: &str = "drive.sync";
    const ACTION_RETRY: &str = "drive.retry-queue";
    const ACTION_HOST_SKILLS: &str = "drive.host-skills";
    const ACTION_UNHOST_SKILLS: &str = "drive.unhost-skills";
    const TOOLBAR_ACTIONS: &[DesktopToolbarActionSpec] = &[
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_REFRESH,
            "Refresh",
            "Reload drive snapshot",
            10,
        ),
        DesktopToolbarActionSpec::primary(Self::ACTION_SYNC, "Sync", "Run one sync cycle", 20),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_RETRY,
            "Retry Queue",
            "Retry queued sync items",
            30,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_HOST_SKILLS,
            "Host Skills",
            "Host ~/.agents/skills",
            40,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_UNHOST_SKILLS,
            "Unhost Skills",
            "Unhost ~/.agents/skills",
            50,
        ),
    ];
    const CONTRIBUTION: DesktopPageContributionSpec = DesktopPageContributionSpec {
        domain_id: OPS_DOMAIN_ID,
        domain_label: "Operations",
        domain_order: 10,
        branch_id: STORAGE_BRANCH_ID,
        parent_branch_id: None,
        branch_label: "Storage",
        branch_order: 10,
        page_id: Self::PAGE_ID,
        page_title: "Drive Center",
        page_subtitle: "Host, sync, inspect queue and conflicts.",
        route: Self::ROUTE,
        page_order: 10,
        summary_card_id: "drive-center-summary",
        summary_title: "Drive Center",
        summary: "Realtime drive operations, queue, conflicts, pool surfaces, and hosting status.",
        summary_order: 10,
        toolbar_actions: Self::TOOLBAR_ACTIONS,
    };

    fn refresh(&mut self, services: &dyn DesktopHostServices) -> Result<(), String> {
        let snapshot = services.load_drive_snapshot()?;
        let mut lines = vec![
            format!("roots: {}", snapshot.roots.len()),
            format!("hosted: {}", snapshot.hosted.len()),
            format!("tracked: {}", snapshot.tracked.len()),
            format!("conflicts: {}", snapshot.conflicts.len()),
            format!("queue: {}", snapshot.queue.len()),
            String::new(),
            "Roots".to_string(),
        ];
        for root in snapshot.roots.iter().take(8) {
            lines.push(format!("  - {} -> {}", root.alias, root.path.display()));
        }
        lines.push(String::new());
        lines.push("Hosted".to_string());
        for item in snapshot.hosted.iter().take(8) {
            lines.push(format!(
                "  - {} [{}] v{:?}",
                item.local_path.display(),
                item.owner_drive_id,
                item.base_version
            ));
        }
        lines.push(String::new());
        lines.push("Conflicts".to_string());
        if snapshot.conflicts.is_empty() {
            lines.push("  - no active conflicts".to_string());
        } else {
            for conflict in snapshot.conflicts.iter().take(8) {
                lines.push(format!(
                    "  - {} @ {}",
                    conflict.conflict_path, conflict.created_at
                ));
            }
        }
        lines.push(String::new());
        lines.push("Queue".to_string());
        if snapshot.queue.is_empty() {
            lines.push("  - queue is empty".to_string());
        } else {
            for item in snapshot.queue.iter().take(8) {
                lines.push(format!(
                    "  - {} [{} / {:?}]",
                    item.remote_path,
                    format!("{:?}", item.status),
                    item.kind
                ));
            }
        }
        self.lines = lines;
        Ok(())
    }

    fn handle_action(&mut self, action_id: &str, ctx: &DesktopExecContext) -> Result<bool, String> {
        match action_id {
            Self::ACTION_REFRESH => {
                self.refresh(ctx.services.as_ref())?;
                ctx.notify("drive-center refreshed");
                Ok(true)
            }
            Self::ACTION_SYNC => {
                let message = ctx.services.drive_sync_once()?;
                self.refresh(ctx.services.as_ref())?;
                ctx.notify(message);
                Ok(true)
            }
            Self::ACTION_RETRY => {
                let message = ctx.services.drive_retry_queue()?;
                self.refresh(ctx.services.as_ref())?;
                ctx.notify(message);
                Ok(true)
            }
            Self::ACTION_HOST_SKILLS => {
                let message = ctx.services.drive_host_path("~/.agents/skills")?;
                self.refresh(ctx.services.as_ref())?;
                ctx.notify(message);
                Ok(true)
            }
            Self::ACTION_UNHOST_SKILLS => {
                let message = ctx.services.drive_unhost_path("~/.agents/skills")?;
                self.refresh(ctx.services.as_ref())?;
                ctx.notify(message);
                Ok(true)
            }
            _ => Ok(false),
        }
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
                    .child(div().text_xl().font_weight(gpui::FontWeight::BOLD).child("Drive Center"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x475467))
                            .child("In-process drive hosting, tracked items, queue, conflicts, and root aliases."),
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
    > for DriveCenterPlugin
{
    fn name(&self) -> &'static str {
        "drive-center"
    }

    fn setup(&mut self, ctx: &mut DesktopInitContext) {
        ctx.register_page_contribution(Self::CONTRIBUTION);
        ctx.register_command(Self::ACTION_SYNC, "Run a drive sync cycle");
    }

    fn on_event(&mut self, event: &DesktopEvent, ctx: &mut DesktopExecContext) -> EventPropagation {
        if matches!(event, DesktopEvent::Startup) {
            let _ = self.refresh(ctx.services.as_ref());
        } else if event.refreshes_route(Self::ROUTE) || event.is_global_refresh() {
            if let Err(err) = self.refresh(ctx.services.as_ref()) {
                ctx.notify(err);
            }
        } else if let Some(action_id) = event.action_id_for_route(Self::ROUTE) {
            match self.handle_action(action_id, ctx) {
                Ok(true) => return EventPropagation::Stop,
                Ok(false) => {}
                Err(err) => {
                    ctx.notify(err);
                    return EventPropagation::Stop;
                }
            }
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
