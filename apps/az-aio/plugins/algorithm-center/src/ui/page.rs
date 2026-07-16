#![cfg(any())]
use std::collections::BTreeSet;

use az_aio_platform::plugin::api::NativeRenderContext;
use adui_dioxus::Card;
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
        div { class: "adui-space adui-space-vertical", style: "display:grid;gap:22px;",
            Card {
                div { class: "adui-card-head-title",
                    span { class: "adui-typography-secondary", "Vision / Algorithm Intake" }
                    h1 { "算法接入中心" }
                    p {
                        "{descriptors.len()} 个视觉算法组件，支持多选叠加。先用视频 URL 或上传入口打通 REST 契约，后续把执行器替换成真实视频加工管线。"
                    }
                }
                div { class: "adui-space", style: "display:flex;flex-wrap:wrap;gap:8px;",
                    span { class: "adui-tag", "SSR 组件库" }
                    span { class: "adui-tag", "POST /api/algorithm-center/process" }
                    span { class: "adui-tag", "多算法叠加" }
                }
            }

            if let Some(error) = &error {
                Card {
                    span { class: "adui-alert-message", "处理错误" }
                    code { "{error}" }
                }
            }

            div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:16px;",
                for descriptor in &descriptors {
                    AlgorithmTile {
                        descriptor: descriptor.clone(),
                        base_route: base_route.clone(),
                        selected_codes: selected_codes.clone(),
                        active_code: active_code.clone(),
                    }
                }
            }

            div { class: "adui-row", style: "display:grid;grid-template-columns:minmax(0,1fr) minmax(320px,420px);gap:16px;align-items:start;",
                Card {
                    h2 { "{active_descriptor.label}" }
                    p { class: "adui-typography-secondary", "{active_descriptor.description}" }
                    div { class: "adui-space", style: "display:flex;flex-wrap:wrap;gap:8px;",
                        span { class: "adui-tag", "{active_descriptor.code}" }
                        span { class: "adui-tag", "输入 {active_descriptor.inputs.len()}" }
                        span { class: "adui-tag", "输出 {active_descriptor.outputs.len()}" }
                    }
                    div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:14px;",
                        ContractList {
                            title: "入参",
                            items: active_descriptor.inputs.iter().map(input_label).collect(),
                        }
                        ContractList {
                            title: "返回",
                            items: active_descriptor.outputs.iter().map(output_label).collect(),
                        }
                    }
                    div { class: "adui-divider", style: "margin-top:16px;padding-top:16px;border-top:1px solid var(--adui-color-border);",
                        h3 { "当前叠加链路" }
                        p { class: "adui-typography-secondary", "{selected_summary}" }
                        div { class: "adui-space", style: "display:flex;flex-wrap:wrap;gap:8px;",
                            for code in &selected_codes {
                                if let Some(descriptor) = descriptors.iter().find(|descriptor| descriptor.code == *code) {
                                    span { class: "adui-tag", "{descriptor.label}" }
                                }
                            }
                        }
                    }
                }

                div { class: "adui-space adui-space-vertical", style: "display:grid;gap:16px;",
                    Card {
                        h2 { "视频处理" }
                        p { class: "adui-typography-secondary", "入参视频 URL，返回加工后 URL。多选算法按当前叠加链路提交。" }
                        form {
                            class: "adui-form",
                            method: "post",
                            action: "/api/algorithm-center/ui-action",
                            for code in &selected_codes {
                                input { r#type: "hidden", name: "algorithms", value: "{code}" }
                            }
                            label { class: "adui-form-item",
                                span { class: "adui-form-item-label", "视频 URL" }
                                input {
                                    class: "adui-input",
                                    id: "algorithm-video-url",
                                    name: "video_url",
                                    value: "{video_url}",
                                    placeholder: "粘贴视频 URL",
                                    required: "required",
                                }
                                small { class: "adui-form-item-extra", "填写上传接口返回的 URL，或填入可访问的视频地址。" }
                            }
                            button { class: "adui-btn adui-btn-solid adui-btn-primary", r#type: "submit", "提交处理" }
                        }
                        form {
                            class: "adui-form",
                            "onsubmit": UPLOAD_FORM_SCRIPT,
                            method: "post",
                            action: "/api/algorithm-center/upload",
                            enctype: "multipart/form-data",
                            label { class: "adui-form-item",
                                span { class: "adui-form-item-label", "上传视频" }
                                input { class: "adui-input", r#type: "file", name: "video", accept: "video/*" }
                                small { class: "adui-form-item-extra", "当前接口固定上传契约，后续接对象存储落点。" }
                            }
                            button { class: "adui-btn adui-btn-outlined adui-btn-default", r#type: "submit", "上传并获取 URL" }
                            div { id: "algorithm-upload-result", class: "adui-form-item-help" }
                        }
                        if !processed_video_url.is_empty() {
                            Card {
                                span { class: "adui-alert-message", "返回 processed_video_url" }
                                code { "{processed_video_url}" }
                                if !job_id.is_empty() {
                                    span { class: "adui-typography-secondary", "job_id: {job_id}" }
                                }
                                if !process_message.is_empty() {
                                    span { class: "adui-typography-secondary", "{process_message}" }
                                }
                            }
                        }
                    }

                    Card {
                        h2 { "REST 调用文档" }
                        p { class: "adui-typography-secondary", "页面表单与接口使用同一份字段。" }
                        pre { class: "adui-card", style: "overflow:auto;padding:12px;", code { "{request_json}" } }
                        pre { class: "adui-card", style: "overflow:auto;padding:12px;", code { "{curl_example}" } }
                        a { class: "adui-btn adui-btn-link", href: "/api/algorithm-center/components", "查看组件目录 JSON" }
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
        Card { class: if selected { "adui-card-hoverable adui-card-bordered" } else { "adui-card-bordered" },
            div { class: "adui-avatar", aria_hidden: "true",
                "{algorithm_icon(&descriptor.label)}"
            }
            div { class: "adui-card-meta",
                h2 { "{descriptor.label}" }
                code { "{descriptor.code}" }
                p { "{descriptor.description}" }
            }
            div { class: "adui-space", style: "display:flex;gap:8px;flex-wrap:wrap;",
                span { class: "adui-tag", "{descriptor.inputs.len()} 输入" }
                span { class: "adui-tag", "{descriptor.outputs.len()} 输出" }
            }
            div { class: "adui-space", style: "display:flex;justify-content:flex-end;gap:8px;",
                a { class: if descriptor.code == active_code { "adui-btn adui-btn-solid adui-btn-primary" } else { "adui-btn adui-btn-outlined adui-btn-default" }, href: focus_href, "详情" }
                a { class: if selected { "adui-btn adui-btn-outlined adui-btn-default" } else { "adui-btn adui-btn-solid adui-btn-primary" }, href,
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
        Card {
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
