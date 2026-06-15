use crate::backend::gateway_runtime_types::GatewayRuntimeStep;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use reqwest::{
    Method, Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde_json::{Map, Value};

pub struct RenderedStepRequest {
    pub body: Option<String>,
    pub headers: HeaderMap,
    pub method: Method,
    pub url: Url,
}

pub fn render_step_request(
    handlebars: &Handlebars<'static>,
    context: &Map<String, Value>,
    step: &GatewayRuntimeStep,
) -> Result<RenderedStepRequest> {
    let url = render_url(handlebars, context, step)?;
    Ok(RenderedStepRequest {
        body: render_body(handlebars, context, step)?,
        headers: render_headers(handlebars, context, step)?,
        method: step_method(step)?,
        url,
    })
}

fn render_body(
    handlebars: &Handlebars<'static>,
    context: &Map<String, Value>,
    step: &GatewayRuntimeStep,
) -> Result<Option<String>> {
    let body = step.body_preview.trim();
    if body.is_empty() {
        return Ok(None);
    }
    Ok(Some(render_template(
        handlebars,
        context,
        body,
        step,
        "request body",
    )?))
}

fn render_headers(
    handlebars: &Handlebars<'static>,
    context: &Map<String, Value>,
    step: &GatewayRuntimeStep,
) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    for (key, value) in &step.headers {
        if key.trim().is_empty() {
            continue;
        }
        let name = HeaderName::from_bytes(key.trim().as_bytes())
            .with_context(|| format!("step {} header name is invalid: {key}", step.id))?;
        let rendered = render_template(handlebars, context, value, step, key)?;
        let header = HeaderValue::from_str(&rendered)
            .with_context(|| format!("step {} header value is invalid: {key}", step.id))?;
        headers.insert(name, header);
    }
    Ok(headers)
}

fn render_url(
    handlebars: &Handlebars<'static>,
    context: &Map<String, Value>,
    step: &GatewayRuntimeStep,
) -> Result<Url> {
    let rendered = render_template(handlebars, context, &step.url, step, "url")?;
    Url::parse(&rendered).with_context(|| format!("step {} url is invalid: {rendered}", step.id))
}

fn render_template(
    handlebars: &Handlebars<'static>,
    context: &Map<String, Value>,
    template: &str,
    step: &GatewayRuntimeStep,
    field: &str,
) -> Result<String> {
    handlebars
        .render_template(template, context)
        .with_context(|| format!("step {} render {field} failed", step.id))
}

fn step_method(step: &GatewayRuntimeStep) -> Result<Method> {
    Method::from_bytes(step.method.trim().as_bytes())
        .with_context(|| format!("step {} invalid http method: {}", step.id, step.method))
}
