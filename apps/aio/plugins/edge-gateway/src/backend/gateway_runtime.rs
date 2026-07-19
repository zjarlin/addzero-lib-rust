use crate::{
    backend::{
        gateway_runtime_request::render_step_request,
        gateway_runtime_response::{failed_step, final_step_value, response_result},
        gateway_runtime_types::{
            GatewayRunRequest, GatewayRunResult, GatewayRunStepResult, GatewayRuntimeStep,
        },
    },
};
use anyhow::{Context, Result};
use handlebars::{Handlebars, no_escape};
use reqwest::Client;
use serde_json::{Map, Value, json};
use std::time::{Duration, Instant};

const GATEWAY_TIMEOUT_SECS: u64 = 30;

pub async fn run_gateway_plan(request: GatewayRunRequest) -> anyhow::Result<GatewayRunResult> {
    GatewayRunner::new(request)?.run().await
}

struct GatewayRunner {
    client: Client,
    context: Map<String, Value>,
    handlebars: Handlebars<'static>,
    request: GatewayRunRequest,
    steps: Vec<GatewayRunStepResult>,
}

impl GatewayRunner {
    fn new(request: GatewayRunRequest) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(GATEWAY_TIMEOUT_SECS))
            .build()
            .context("create gateway http client failed")?;
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(no_escape);
        handlebars.set_strict_mode(true);
        let context = Map::from_iter([("input".to_string(), request.input.clone())]);
        Ok(Self {
            client,
            context,
            handlebars,
            request,
            steps: Vec::new(),
        })
    }

    async fn run(mut self) -> anyhow::Result<GatewayRunResult> {
        if self.request.steps.is_empty() {
            return Ok(self.finish(false, "no steps", "execution plan is empty"));
        }
        for step in self.request.steps.clone() {
            let result = self.run_step(&step).await;
            let keep_running = result.ok;
            self.store_step_context(&result);
            self.steps.push(result);
            if !keep_running {
                break;
            }
        }
        let ok =
            self.steps.len() == self.request.steps.len() && self.steps.iter().all(|step| step.ok);
        let status = if ok { "completed" } else { "interrupted" };
        let message = self.finish_message(ok);
        Ok(self.finish(ok, status, &message))
    }

    async fn run_step(&self, step: &GatewayRuntimeStep) -> GatewayRunStepResult {
        let started = Instant::now();
        match self.send_step(step).await {
            Ok(result) => with_duration(result, started),
            Err(error) => failed_step(step, "", started.elapsed().as_millis(), error.to_string()),
        }
    }

    async fn send_step(&self, step: &GatewayRuntimeStep) -> Result<GatewayRunStepResult> {
        let request = render_step_request(&self.handlebars, &self.context, step)?;
        let url = request.url.to_string();
        let builder = self
            .client
            .request(request.method, request.url)
            .headers(request.headers);
        let response = match request.body {
            Some(value) => builder.body(value).send().await,
            None => builder.send().await,
        }
        .with_context(|| format!("step {} request failed", step.id))?;
        response_result(step, url, response).await
    }

    fn store_step_context(&mut self, result: &GatewayRunStepResult) {
        self.context.insert(
            result.id.clone(),
            json!({
                "capture": result.captured,
                "response": {
                    "body": result.response_body,
                    "headers": result.response_headers,
                    "status": result.status_code,
                },
            }),
        );
    }

    fn finish(self, ok: bool, status: &str, message: &str) -> GatewayRunResult {
        GatewayRunResult {
            entry_route: self.request.entry_route,
            final_result: self.steps.last().map(final_step_value),
            message: message.to_string(),
            ok,
            status: status.to_string(),
            steps: self.steps,
        }
    }

    fn finish_message(&self, ok: bool) -> String {
        if ok {
            return format!("executed {} steps in order", self.steps.len());
        }
        self.steps
            .iter()
            .find_map(|step| step.error.clone())
            .unwrap_or_else(|| "step execution failed".to_string())
    }
}

fn with_duration(mut result: GatewayRunStepResult, started: Instant) -> GatewayRunStepResult {
    result.duration_ms = started.elapsed().as_millis();
    result
}

#[cfg(test)]
mod tests {
    use super::run_gateway_plan;
    use crate::backend::gateway_runtime_types::{GatewayRunRequest, GatewayRuntimeStep};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };

    #[tokio::test]
    async fn cascades_captured_login_token_into_next_request() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            serve_request(listener.accept().await.unwrap().0).await;
            serve_request(listener.accept().await.unwrap().0).await;
        });

        let result = run_gateway_plan(GatewayRunRequest {
            entry_route: "/edge/session-proxy".to_string(),
            input: Value::Null,
            steps: vec![
                step(
                    "login",
                    "POST",
                    &format!("{base_url}/login"),
                    BTreeMap::new(),
                    "{\"account\":\"demo\"}",
                    "$.token",
                ),
                step(
                    "profile",
                    "GET",
                    &format!("{base_url}/profile"),
                    BTreeMap::from([(
                        "Authorization".to_string(),
                        "Bearer {{login.response.body.token}}".to_string(),
                    )]),
                    "",
                    "$.data.name",
                ),
            ],
        })
        .await
        .unwrap();

        server.await.unwrap();
        assert!(result.ok);
        assert_eq!(result.steps[0].captured, Some(json!("edge-token")));
        assert_eq!(result.steps[1].captured, Some(json!("demo")));
    }

    fn step(
        id: &str,
        method: &str,
        url: &str,
        headers: BTreeMap<String, String>,
        body_preview: &str,
        capture_path: &str,
    ) -> GatewayRuntimeStep {
        GatewayRuntimeStep {
            body_preview: body_preview.to_string(),
            capture_path: capture_path.to_string(),
            depends_on: Vec::new(),
            headers,
            id: id.to_string(),
            input_refs: Vec::new(),
            kind: "curl".to_string(),
            label: id.to_string(),
            method: method.to_string(),
            notes: String::new(),
            url: url.to_string(),
        }
    }

    async fn serve_request(mut stream: TcpStream) {
        let mut buffer = vec![0_u8; 4096];
        let read = stream.read(&mut buffer).await.unwrap();
        let request = String::from_utf8_lossy(&buffer[..read]);
        let normalized = request.to_ascii_lowercase();
        let response = if request.starts_with("POST /login ") {
            json_response(200, "{\"token\":\"edge-token\"}")
        } else if request.starts_with("GET /profile ")
            && normalized.contains("authorization: bearer edge-token")
        {
            json_response(200, "{\"data\":{\"name\":\"demo\"}}")
        } else {
            json_response(401, "{\"error\":\"missing token\"}")
        };
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    fn json_response(status: u16, body: &str) -> String {
        let reason = if status == 200 { "OK" } else { "Unauthorized" };
        format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }
}
