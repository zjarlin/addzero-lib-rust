#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod gateway_runtime;
pub mod gateway_runtime_request;
pub mod gateway_runtime_response;
pub mod gateway_runtime_types;

use std::collections::BTreeMap;

use az_desktop_plugin::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopPageContributionSpec,
    DesktopRenderLayer, DesktopToolbarActionSpec, DesktopViewContext, EventPropagation, Plugin,
};
use az_desktop_plugin_registry::declare_desktop_plugin;
use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, rgb};
use serde_json::Value;

use crate::gateway_runtime::run_gateway_plan;
use crate::gateway_runtime_types::{GatewayRunRequest, GatewayRunResult, GatewayRuntimeStep};

const OPS_DOMAIN_ID: &str = "operations";
const EDGE_BRANCH_ID: &str = "operations-edge";

declare_desktop_plugin! {
    struct EdgeGatewayPlugin {
        plan: Option<GatewayRunRequest>,
        result: Option<GatewayRunResult>,
        lines: Vec<String>,
    }
}

impl EdgeGatewayPlugin {
    const PAGE_ID: &str = "edge-gateway";
    const ROUTE: &str = "/gateway";
    const ACTION_REFRESH: &str = "edge-gateway.refresh";
    const ACTION_LOAD_EXAMPLE: &str = "edge-gateway.load-example";
    const ACTION_RUN_EXAMPLE: &str = "edge-gateway.run-example";
    const TOOLBAR_ACTIONS: &[DesktopToolbarActionSpec] = &[
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_REFRESH,
            "Refresh",
            "Refresh gateway panel state",
            10,
        ),
        DesktopToolbarActionSpec::secondary(
            Self::ACTION_LOAD_EXAMPLE,
            "Load Example",
            "Load a reference gateway plan",
            20,
        ),
        DesktopToolbarActionSpec::primary(
            Self::ACTION_RUN_EXAMPLE,
            "Run Example",
            "Execute the loaded example gateway plan",
            30,
        ),
    ];
    const CONTRIBUTION: DesktopPageContributionSpec = DesktopPageContributionSpec {
        domain_id: OPS_DOMAIN_ID,
        domain_label: "Operations",
        domain_order: 10,
        branch_id: EDGE_BRANCH_ID,
        parent_branch_id: None,
        branch_label: "Network",
        branch_order: 20,
        page_id: Self::PAGE_ID,
        page_title: "Edge Gateway",
        page_subtitle: "Gateway flow editor, plan generation, runtime execution, and helper references.",
        route: Self::ROUTE,
        page_order: 20,
        summary_card_id: "edge-gateway-summary",
        summary_title: "Edge Gateway",
        summary: "Flow editor/runtime reference with Rust planner and HTTP chain execution.",
        summary_order: 50,
        toolbar_actions: Self::TOOLBAR_ACTIONS,
    };

    fn refresh(&mut self) {
        let mut lines = vec![
            "Templates".to_string(),
            "  - Session Proxy".to_string(),
            "  - Login + Profile Chain".to_string(),
            "  - JSONPath Capture Reference".to_string(),
        ];
        if let Some(plan) = &self.plan {
            lines.push(String::new());
            lines.push(format!("Plan: {}", plan.entry_route));
            for step in &plan.steps {
                lines.push(format!(
                    "  - {} {} -> {}",
                    step.method, step.label, step.url
                ));
            }
        }
        if let Some(result) = &self.result {
            lines.push(String::new());
            lines.push(format!("Last Run: {} / {}", result.status, result.message));
            for step in &result.steps {
                lines.push(format!(
                    "  - {} ok={} status={:?} capture={}",
                    step.label,
                    step.ok,
                    step.status_code,
                    step.captured
                        .as_ref()
                        .map(Value::to_string)
                        .unwrap_or_else(|| "-".to_string())
                ));
            }
        }
        self.lines = lines;
    }

    fn load_example(&mut self) -> String {
        self.plan = Some(example_plan());
        self.refresh();
        "loaded gateway example plan".to_string()
    }

    fn run_example(&mut self) -> Result<String, String> {
        let plan = self.plan.clone().unwrap_or_else(example_plan);
        let runtime = tokio::runtime::Runtime::new().map_err(|err| err.to_string())?;
        let result = runtime
            .block_on(run_gateway_plan(plan.clone()))
            .map_err(|err| err.to_string())?;
        let message = result.message.clone();
        self.plan = Some(plan);
        self.result = Some(result);
        self.refresh();
        Ok(message)
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
                            .child("Edge Gateway"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x475467))
                            .child("Gateway flow templates, plan generation reference, runtime execution, and result panels."),
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

fn example_plan() -> GatewayRunRequest {
    GatewayRunRequest {
        entry_route: "/edge/session-proxy".to_string(),
        input: Value::Null,
        steps: vec![GatewayRuntimeStep {
            body_preview: String::new(),
            capture_path: "$.headers.host".to_string(),
            depends_on: Vec::new(),
            headers: BTreeMap::new(),
            id: "ping".to_string(),
            input_refs: Vec::new(),
            kind: "curl".to_string(),
            label: "GET postman echo".to_string(),
            method: "GET".to_string(),
            notes: "Reference flow".to_string(),
            url: "https://postman-echo.com/get?source=aio-desktop".to_string(),
        }],
    }
}

impl
    Plugin<
        DesktopInitContext,
        DesktopEvent,
        DesktopExecContext,
        DesktopViewContext,
        DesktopRenderLayer,
    > for EdgeGatewayPlugin
{
    fn name(&self) -> &'static str {
        "edge-gateway"
    }

    fn setup(&mut self, ctx: &mut DesktopInitContext) {
        ctx.register_page_contribution(Self::CONTRIBUTION);
    }

    fn on_event(&mut self, event: &DesktopEvent, ctx: &mut DesktopExecContext) -> EventPropagation {
        match event {
            DesktopEvent::Startup => self.refresh(),
            DesktopEvent::RouteChanged { route }
            | DesktopEvent::RefreshRequested { route: Some(route) }
                if route == Self::ROUTE =>
            {
                self.refresh();
            }
            DesktopEvent::RefreshRequested { route: None } => self.refresh(),
            DesktopEvent::ActionInvoked { route, action_id } if route == Self::ROUTE => {
                let result = match action_id.as_str() {
                    Self::ACTION_REFRESH => {
                        self.refresh();
                        Ok("edge-gateway refreshed".to_string())
                    }
                    Self::ACTION_LOAD_EXAMPLE => Ok(self.load_example()),
                    Self::ACTION_RUN_EXAMPLE => self.run_example(),
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
