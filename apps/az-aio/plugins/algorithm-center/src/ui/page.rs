use std::collections::BTreeSet;

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::neobrutal::{
    Badge, BlockTitle, Button, Card, CodeBlock, Eyebrow, Field, Grid, Hero,
    LinkButton, Page, Split,
};
use dioxus::prelude::*;

type Descriptor = az_algorithm::catalog::model::AlgorithmComponentDescriptor;
const UPLOAD_FORM_SCRIPT: &str = r#"
event.preventDefault();
const form = event.currentTarget;
const result = document.getElementById('algorithm-upload-result');
const input = document.getElementById('algorithm-video-url');
const file = form.querySelector('input[type=file]');
if (!file || !file.files || file.files.length === 0) {
    if (result) result.textContent = '请先选择视频文件';
    return;
}
if (result) result.textContent = '上传中...';
fetch(form.action, { method: 'POST', body: new FormData(form) })
    .then(async (response) => {
        const payload = await response.json().catch(() => ({}));
        if (!response.ok || !payload.ok) {
            throw new Error(payload.msg || payload.error || '上传失败');
        }
        if (input) input.value = payload.uploaded_video_url || '';
        if (result) result.textContent = payload.uploaded_video_url || '上传完成';
    })
    .catch((error) => {
        if (result) result.textContent = error.message || '上传失败';
    });
"#;

const ALGORITHM_CENTER_STYLE: &str = r#"
.algorithm-center-page {
  gap: 22px;
}

.algorithm-center-hero__meta {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.algorithm-center-mosaic {
  align-items: stretch;
}

.algorithm-tile {
  min-height: 214px;
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr) auto auto;
  gap: 12px;
}

.algorithm-tile__icon {
  width: 48px;
  height: 48px;
  display: grid;
  place-items: center;
  border: 3px solid var(--page-line);
  border-radius: 8px;
  background: var(--page-accent);
  color: #ffffff;
  font-size: 18px;
  font-weight: 900;
}

.algorithm-tile__body {
  min-width: 0;
  display: grid;
  align-content: start;
  gap: 7px;
}

.algorithm-tile h2 {
  margin: 0;
  color: var(--page-ink);
  font-size: 17px;
  font-weight: 900;
  line-height: 1.15;
}

.algorithm-tile code {
  min-width: 0;
  overflow: hidden;
  color: var(--page-muted);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.algorithm-tile p {
  margin: 0;
  color: var(--page-muted);
  font-size: 13px;
  line-height: 1.42;
}

.algorithm-tile__meta,
.algorithm-tile__actions,
.algorithm-detail-panel__tags,
.algorithm-selected-stack__chips {
  grid-column: 1 / -1;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.algorithm-tile__actions {
  justify-content: flex-end;
}

.algorithm-detail-panel,
.algorithm-form-panel,
.algorithm-doc-panel,
.algorithm-action-column {
  display: grid;
  gap: 16px;
}

.algorithm-contract-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.algorithm-contract-list {
  padding: 12px;
  border: 3px solid var(--page-line);
  border-radius: 8px;
  background: #ffffff;
}

.algorithm-contract-list h3 {
  margin: 0 0 10px;
  font-size: 14px;
  font-weight: 900;
}

.algorithm-contract-list ul {
  margin: 0;
  padding-left: 18px;
  color: var(--page-muted);
  font-size: 13px;
  line-height: 1.55;
}

.algorithm-selected-stack {
  display: grid;
  gap: 10px;
  padding-top: 12px;
  border-top: 3px solid var(--page-line);
}

.algorithm-process-form,
.algorithm-upload-form {
  display: grid;
  gap: 12px;
}

.algorithm-upload-form {
  padding-top: 14px;
  border-top: 3px dashed var(--page-line);
}

.algorithm-result {
  display: grid;
  gap: 6px;
  padding: 12px;
  border: 3px solid var(--page-line);
  border-radius: 8px;
  background: #bbf7d0;
}

.algorithm-result__label {
  font-size: 12px;
  font-weight: 900;
}

.algorithm-result code {
  min-width: 0;
  overflow-wrap: anywhere;
  font-size: 12px;
}

@media (max-width: 920px) {
  .algorithm-contract-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .algorithm-center-hero__meta {
    justify-content: flex-start;
  }
}

@media (max-width: 720px) {
  .algorithm-center-page {
    padding: 22px 16px;
  }

  .algorithm-contract-grid {
    grid-template-columns: 1fr;
  }

  .algorithm-tile {
    min-height: 190px;
  }
}
"#;

#[allow(non_snake_case)]
pub fn AlgorithmCenterPage(context: NativeRenderContext) -> Element {
    let descriptors = az_algorithm::catalog::query::algorithm_component_descriptors();
    let selected_codes = selected_algorithm_codes(&context.active_route, &descriptors);
    let active_code = parse_query_param(&context.active_route, "active")
        .filter(|code| selected_codes.contains(code))
        .or_else(|| selected_codes.first().cloned())
        .unwrap_or_else(|| descriptors[0].code.clone());
    let active_descriptor = descriptors
        .iter()
        .find(|descriptor| descriptor.code == active_code)
        .unwrap_or(&descriptors[0]);
    let video_url = parse_query_param(&context.active_route, "video_url").unwrap_or_default();
    let processed_video_url =
        parse_query_param(&context.active_route, "processed_video_url").unwrap_or_default();
    let job_id = parse_query_param(&context.active_route, "job_id").unwrap_or_default();
    let process_message = parse_query_param(&context.active_route, "message").unwrap_or_default();
    let error = parse_query_param(&context.active_route, "error");
    let base_route = route_without_query(&context.active_route);
    let selected_summary = selected_codes
        .iter()
        .filter_map(|code| descriptors.iter().find(|descriptor| descriptor.code == *code))
        .map(|descriptor| descriptor.label.as_str())
        .collect::<Vec<_>>()
        .join(" + ");
    let request_json = process_request_json(&video_url, &selected_codes);
    let curl_example = process_curl_example(&request_json);

    rsx! {
        style {
            "data-az-style": "algorithm-center-page",
            dangerous_inner_html: ALGORITHM_CENTER_STYLE,
        }
        Page { class: "algorithm-center-page",
            Hero { class: "algorithm-center-hero",
                div { class: "algorithm-center-hero__copy",
                    Eyebrow { "Vision / Algorithm Intake" }
                    h1 { "算法接入中心" }
                    p {
                        "{descriptors.len()} 个视觉算法组件，支持多选叠加。先用视频 URL 或上传入口打通 REST 契约，后续把执行器替换成真实视频加工管线。"
                    }
                }
                div { class: "algorithm-center-hero__meta",
                    Badge { accent: true, "SSR 组件库" }
                    Badge { "POST /api/algorithm-center/process" }
                    Badge { "多算法叠加" }
                }
            }

            if let Some(error) = &error {
                Card { class: "algorithm-result",
                    span { class: "algorithm-result__label", "处理错误" }
                    code { "{error}" }
                }
            }

            Grid { class: "algorithm-center-mosaic",
                for descriptor in &descriptors {
                    AlgorithmTile {
                        descriptor: descriptor.clone(),
                        base_route: base_route.clone(),
                        selected_codes: selected_codes.clone(),
                        active_code: active_code.clone(),
                    }
                }
            }

            Split { class: "algorithm-workbench",
                Card { class: "algorithm-detail-panel", selected: true,
                    BlockTitle {
                        title: active_descriptor.label.clone(),
                        subtitle: active_descriptor.description.clone(),
                    }
                    div { class: "algorithm-detail-panel__tags",
                        Badge { accent: true, "{active_descriptor.code}" }
                        Badge { "输入 {active_descriptor.inputs.len()}" }
                        Badge { "输出 {active_descriptor.outputs.len()}" }
                    }
                    div { class: "algorithm-contract-grid",
                        ContractList {
                            title: "入参",
                            items: active_descriptor.inputs.iter().map(input_label).collect(),
                        }
                        ContractList {
                            title: "返回",
                            items: active_descriptor.outputs.iter().map(output_label).collect(),
                        }
                    }
                    div { class: "algorithm-selected-stack",
                        BlockTitle {
                            title: "当前叠加链路".to_string(),
                            subtitle: if selected_summary.is_empty() {
                                "默认选中第一个算法".to_string()
                            } else {
                                selected_summary.clone()
                            },
                        }
                        div { class: "algorithm-selected-stack__chips",
                            for code in &selected_codes {
                                if let Some(descriptor) = descriptors.iter().find(|descriptor| descriptor.code == *code) {
                                    Badge { "{descriptor.label}" }
                                }
                            }
                        }
                    }
                }

                div { class: "algorithm-action-column",
                    Card { class: "algorithm-form-panel", accent: true,
                        BlockTitle {
                            title: "视频处理".to_string(),
                            subtitle: "入参视频 URL，返回加工后 URL。多选算法按当前叠加链路提交。".to_string(),
                        }
                        form {
                            class: "algorithm-process-form",
                            method: "post",
                            action: "/api/algorithm-center/ui-action",
                            for code in &selected_codes {
                                input { r#type: "hidden", name: "algorithms", value: "{code}" }
                            }
                            Field {
                                label: "视频 URL".to_string(),
                                hint: "填写上传接口返回的 URL，或填入可访问的视频地址。".to_string(),
                                input {
                                    class: "input",
                                    id: "algorithm-video-url",
                                    name: "video_url",
                                    value: "{video_url}",
                                    placeholder: "粘贴视频 URL",
                                    required: "required",
                                }
                            }
                            Button { primary: true, button_type: "submit", "提交处理" }
                        }
                        form {
                            class: "algorithm-upload-form",
                            "onsubmit": UPLOAD_FORM_SCRIPT,
                            method: "post",
                            action: "/api/algorithm-center/upload",
                            enctype: "multipart/form-data",
                            Field {
                                label: "上传视频".to_string(),
                                hint: "当前接口固定上传契约，后续接对象存储落点。".to_string(),
                                input { class: "input", r#type: "file", name: "video", accept: "video/*" }
                            }
                            Button { button_type: "submit", "上传并获取 URL" }
                            div { id: "algorithm-upload-result", class: "algorithm-result__label" }
                        }
                        if !processed_video_url.is_empty() {
                            div { class: "algorithm-result",
                                span { class: "algorithm-result__label", "返回 processed_video_url" }
                                code { "{processed_video_url}" }
                                if !job_id.is_empty() {
                                    span { class: "algorithm-result__label", "job_id: {job_id}" }
                                }
                                if !process_message.is_empty() {
                                    span { class: "algorithm-result__label", "{process_message}" }
                                }
                            }
                        }
                    }

                    Card { class: "algorithm-doc-panel",
                        BlockTitle {
                            title: "REST 调用文档".to_string(),
                            subtitle: "页面表单与接口使用同一份字段。".to_string(),
                        }
                        CodeBlock { code: request_json.clone() }
                        CodeBlock { code: curl_example }
                        LinkButton {
                            href: "/api/algorithm-center/components".to_string(),
                            "查看组件目录 JSON"
                        }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn AlgorithmTile(
    descriptor: Descriptor,
    base_route: String,
    selected_codes: Vec<String>,
    active_code: String,
) -> Element {
    let selected = selected_codes.contains(&descriptor.code);
    let href = toggle_algorithm_href(&base_route, &selected_codes, &descriptor.code);
    let focus_href = selected_algorithm_href(&base_route, &selected_codes, &descriptor.code);

    rsx! {
        Card { class: "algorithm-tile", selected: selected,
            div { class: "algorithm-tile__icon", aria_hidden: "true",
                "{algorithm_icon(&descriptor.label)}"
            }
            div { class: "algorithm-tile__body",
                h2 { "{descriptor.label}" }
                code { "{descriptor.code}" }
                p { "{descriptor.description}" }
            }
            div { class: "algorithm-tile__meta",
                Badge { "{descriptor.inputs.len()} 输入" }
                Badge { "{descriptor.outputs.len()} 输出" }
            }
            div { class: "algorithm-tile__actions",
                LinkButton { href: focus_href, primary: descriptor.code == active_code, "详情" }
                LinkButton { href: href, primary: !selected,
                    if selected { "移除叠加" } else { "加入叠加" }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ContractList(title: String, items: Vec<String>) -> Element {
    rsx! {
        div { class: "algorithm-contract-list",
            h3 { "{title}" }
            ul {
                for item in items {
                    li { "{item}" }
                }
            }
        }
    }
}

fn selected_algorithm_codes(route: &str, descriptors: &[Descriptor]) -> Vec<String> {
    let known_codes = descriptors
        .iter()
        .map(|descriptor| descriptor.code.as_str())
        .collect::<BTreeSet<_>>();
    let mut selected = parse_query_params(route, "algorithm")
        .into_iter()
        .filter(|code| known_codes.contains(code.as_str()))
        .collect::<Vec<_>>();

    selected.sort();
    selected.dedup();

    if selected.is_empty() {
        selected.push(descriptors[0].code.clone());
    }

    selected
}

fn toggle_algorithm_href(base_route: &str, selected_codes: &[String], code: &str) -> String {
    let mut next = selected_codes
        .iter()
        .filter(|selected| selected.as_str() != code)
        .cloned()
        .collect::<Vec<_>>();
    if next.len() == selected_codes.len() {
        next.push(code.to_string());
    }
    if next.is_empty() {
        next.push(code.to_string());
    }
    selected_algorithm_href(base_route, &next, code)
}

fn selected_algorithm_href(base_route: &str, selected_codes: &[String], active_code: &str) -> String {
    let mut parts = vec![format!("route={}", urlencoding::encode(base_route))];
    for code in selected_codes {
        parts.push(format!("algorithm={}", urlencoding::encode(code)));
    }
    parts.push(format!("active={}", urlencoding::encode(active_code)));
    format!("/?{}", parts.join("&"))
}

fn route_without_query(route: &str) -> String {
    route.split('?').next().unwrap_or("/algorithms").to_string()
}

fn parse_query_param(route: &str, key: &str) -> Option<String> {
    parse_query_params(route, key).into_iter().next()
}

fn parse_query_params(route: &str, key: &str) -> Vec<String> {
    let Some(query) = route.split('?').nth(1) else {
        return Vec::new();
    };

    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            if parts.next()? != key {
                return None;
            }
            let raw = parts.next().unwrap_or_default();
            Some(
                urlencoding::decode(raw)
                    .unwrap_or_else(|_| raw.into())
                    .into_owned(),
            )
        })
        .collect()
}

fn process_request_json(video_url: &str, selected_codes: &[String]) -> String {
    let body = serde_json::json!({
        "video_url": video_url,
        "algorithms": selected_codes,
    });
    serde_json::to_string_pretty(&body).unwrap_or_else(|_| "{}".to_string())
}

fn process_curl_example(request_json: &str) -> String {
    format!(
        "curl -X POST http://localhost:18080/api/algorithm-center/process \\\n  -H 'content-type: application/json' \\\n  -d '{}'",
        request_json.replace('\'', "\\'")
    )
}

fn algorithm_icon(label: &str) -> &'static str {
    match label {
        "火焰检测" => "火",
        "人脸检测" => "脸",
        "人脸识别" => "识",
        "人员检测" => "人",
        "OCR文字识别" => "文",
        "安全帽检测" => "帽",
        "车辆检测" => "车",
        "二维码识别" => "码",
        "工人敲击计数" => "数",
        _ => "算",
    }
}

fn input_label(input: &az_algorithm::catalog::model::AlgorithmInputKind) -> String {
    match input {
        az_algorithm::catalog::model::AlgorithmInputKind::Image => "图片或视频帧".to_string(),
        az_algorithm::catalog::model::AlgorithmInputKind::ReferenceSet => "参考底库".to_string(),
        az_algorithm::catalog::model::AlgorithmInputKind::RegionOfInterest => {
            "感兴趣区域".to_string()
        }
        az_algorithm::catalog::model::AlgorithmInputKind::VideoFrames => "视频帧序列".to_string(),
        az_algorithm::catalog::model::AlgorithmInputKind::PersonTracks => "人员轨迹".to_string(),
        az_algorithm::catalog::model::AlgorithmInputKind::ActionScores => "动作置信度".to_string(),
        az_algorithm::catalog::model::AlgorithmInputKind::TargetObservations => {
            "目标观测".to_string()
        }
        az_algorithm::catalog::model::AlgorithmInputKind::ContactPoints => "接触点".to_string(),
    }
}

fn output_label(output: &az_algorithm::catalog::model::AlgorithmOutputKind) -> String {
    match output {
        az_algorithm::catalog::model::AlgorithmOutputKind::BoundingBox => "目标框".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::Confidence => "置信度".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::ClassLabel => "分类标签".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::Identity => "身份".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::SimilarityScore => "相似度".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::Text => "文本内容".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::QrPayload => "二维码载荷".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::EventCount => "事件计数".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::EventTimestamp => {
            "事件时间戳".to_string()
        }
        az_algorithm::catalog::model::AlgorithmOutputKind::PersonTrackId => {
            "人员轨迹 ID".to_string()
        }
        az_algorithm::catalog::model::AlgorithmOutputKind::ActionState => "动作状态".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::TargetId => "目标 ID".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::ContactPoint => "接触点".to_string(),
        az_algorithm::catalog::model::AlgorithmOutputKind::InvalidReason => "无效原因".to_string(),
    }
}
