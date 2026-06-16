use std::collections::BTreeSet;

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::neobrutal::{
    NbBadge, NbBlockTitle, NbButton, NbCard, NbCodeBlock, NbEyebrow, NbField, NbGrid, NbHero,
    NbLinkButton, NbPage, NbSplit,
};
use dioxus::prelude::*;

type Descriptor = az_algorithm::catalog::AlgorithmComponentDescriptor;
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
            throw new Error(payload.error || '上传失败');
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
    let descriptors = az_algorithm::catalog::algorithm_component_descriptors();
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
    let has_run = parse_query_param(&context.active_route, "run").as_deref() == Some("1");
    let processed_video_url = processed_video_url(&video_url, &selected_codes);
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
        NbPage { class: "algorithm-center-page",
            NbHero { class: "algorithm-center-hero",
                div { class: "algorithm-center-hero__copy",
                    NbEyebrow { "Vision / Algorithm Intake" }
                    h1 { "算法接入中心" }
                    p {
                        "{descriptors.len()} 个视觉算法组件，支持多选叠加。先用视频 URL 或上传入口打通 REST 契约，后续把执行器替换成真实视频加工管线。"
                    }
                }
                div { class: "algorithm-center-hero__meta",
                    NbBadge { accent: true, "SSR 组件库" }
                    NbBadge { "POST /api/algorithm-center/process" }
                    NbBadge { "多算法叠加" }
                }
            }

            NbGrid { class: "algorithm-center-mosaic",
                for descriptor in &descriptors {
                    AlgorithmTile {
                        descriptor: descriptor.clone(),
                        base_route: base_route.clone(),
                        selected_codes: selected_codes.clone(),
                        active_code: active_code.clone(),
                    }
                }
            }

            NbSplit { class: "algorithm-workbench",
                NbCard { class: "algorithm-detail-panel", selected: true,
                    NbBlockTitle {
                        title: active_descriptor.label.clone(),
                        subtitle: active_descriptor.description.clone(),
                    }
                    div { class: "algorithm-detail-panel__tags",
                        NbBadge { accent: true, "{active_descriptor.code}" }
                        NbBadge { "输入 {active_descriptor.inputs.len()}" }
                        NbBadge { "输出 {active_descriptor.outputs.len()}" }
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
                        NbBlockTitle {
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
                                    NbBadge { "{descriptor.label}" }
                                }
                            }
                        }
                    }
                }

                div { class: "algorithm-action-column",
                    NbCard { class: "algorithm-form-panel", accent: true,
                        NbBlockTitle {
                            title: "视频处理".to_string(),
                            subtitle: "入参视频 URL，返回加工后 URL。多选算法按当前叠加链路提交。".to_string(),
                        }
                        form {
                            class: "algorithm-process-form",
                            method: "get",
                            action: "/",
                            input { r#type: "hidden", name: "route", value: "{base_route}" }
                            input { r#type: "hidden", name: "active", value: "{active_code}" }
                            for code in &selected_codes {
                                input { r#type: "hidden", name: "algorithm", value: "{code}" }
                            }
                            NbField {
                                label: "视频 URL".to_string(),
                                hint: "示例: https://example.com/input.mp4".to_string(),
                                input {
                                    class: "nb-input",
                                    id: "algorithm-video-url",
                                    name: "video_url",
                                    value: "{video_url}",
                                    placeholder: "https://example.com/input.mp4",
                                    required: "required",
                                }
                            }
                            NbButton { primary: true, button_type: "submit", "提交处理" }
                            input { r#type: "hidden", name: "run", value: "1" }
                        }
                        form {
                            class: "algorithm-upload-form",
                            "onsubmit": UPLOAD_FORM_SCRIPT,
                            method: "post",
                            action: "/api/algorithm-center/upload",
                            enctype: "multipart/form-data",
                            NbField {
                                label: "上传视频".to_string(),
                                hint: "当前接口固定上传契约，后续接对象存储落点。".to_string(),
                                input { class: "nb-input", r#type: "file", name: "video", accept: "video/*" }
                            }
                            NbButton { button_type: "submit", "上传并获取 URL" }
                            div { id: "algorithm-upload-result", class: "algorithm-result__label" }
                        }
                        if has_run && !video_url.is_empty() {
                            div { class: "algorithm-result",
                                span { class: "algorithm-result__label", "返回 processed_video_url" }
                                code { "{processed_video_url}" }
                            }
                        }
                    }

                    NbCard { class: "algorithm-doc-panel",
                        NbBlockTitle {
                            title: "REST 调用文档".to_string(),
                            subtitle: "页面表单与接口使用同一份字段。".to_string(),
                        }
                        NbCodeBlock { code: request_json.clone() }
                        NbCodeBlock { code: curl_example }
                        NbLinkButton {
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
        NbCard { class: "algorithm-tile", selected: selected,
            div { class: "algorithm-tile__icon", aria_hidden: "true",
                "{algorithm_icon(&descriptor.label)}"
            }
            div { class: "algorithm-tile__body",
                h2 { "{descriptor.label}" }
                code { "{descriptor.code}" }
                p { "{descriptor.description}" }
            }
            div { class: "algorithm-tile__meta",
                NbBadge { "{descriptor.inputs.len()} 输入" }
                NbBadge { "{descriptor.outputs.len()} 输出" }
            }
            div { class: "algorithm-tile__actions",
                NbLinkButton { href: focus_href, primary: descriptor.code == active_code, "详情" }
                NbLinkButton { href: href, primary: !selected,
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

fn processed_video_url(video_url: &str, selected_codes: &[String]) -> String {
    let codes = selected_codes.join(",");
    let job_id = format!("job-{:x}", md5::compute(format!("{video_url}|{codes}")));
    format!("/api/algorithm-center/results/{job_id}/processed.mp4")
}

fn process_request_json(video_url: &str, selected_codes: &[String]) -> String {
    let video_url = if video_url.is_empty() {
        "https://example.com/input.mp4"
    } else {
        video_url
    };
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

fn input_label(input: &az_algorithm::catalog::AlgorithmInputKind) -> String {
    match input {
        az_algorithm::catalog::AlgorithmInputKind::Image => "图片或视频帧".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::ReferenceSet => "参考底库".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::RegionOfInterest => "感兴趣区域".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::VideoFrames => "视频帧序列".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::PersonTracks => "人员轨迹".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::ActionScores => "动作置信度".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::TargetObservations => "目标观测".to_string(),
        az_algorithm::catalog::AlgorithmInputKind::ContactPoints => "接触点".to_string(),
    }
}

fn output_label(output: &az_algorithm::catalog::AlgorithmOutputKind) -> String {
    match output {
        az_algorithm::catalog::AlgorithmOutputKind::BoundingBox => "目标框".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::Confidence => "置信度".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::ClassLabel => "分类标签".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::Identity => "身份".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::Text => "文本内容".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::QrPayload => "二维码载荷".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::EventCount => "事件计数".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::EventTimestamp => "事件时间戳".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::PersonTrackId => "人员轨迹 ID".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::ActionState => "动作状态".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::TargetId => "目标 ID".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::ContactPoint => "接触点".to_string(),
        az_algorithm::catalog::AlgorithmOutputKind::InvalidReason => "无效原因".to_string(),
    }
}
