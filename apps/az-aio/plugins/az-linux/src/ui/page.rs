#![cfg(any())]
use az_aio_platform::plugin::api::NativeRenderContext;
use adui_dioxus::Card;
use dioxus::prelude::*;

use crate::ui::state::{LinuxPlanParams, load_snapshot};

const MAX_LIST_ROWS: usize = 12;

#[allow(non_snake_case)]
pub fn LinuxPage(context: NativeRenderContext) -> Element {
    let base_route = route_without_query(&context.active_route);
    let host = parse_query_param(&context.active_route, "host");
    let port = parse_query_param(&context.active_route, "port")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(22);
    let user = parse_query_param(&context.active_route, "user").unwrap_or_else(|| "ubuntu".to_string());
    let pair_token = parse_query_param(&context.active_route, "pair_token");
    let public_key = parse_query_param(&context.active_route, "public_key");
    let visible_base = context.api_base_url.trim().to_string();
    let snapshot = load_snapshot(LinuxPlanParams {
        host: host.clone(),
        port,
        user: user.clone(),
        client_endpoint: visible_base.clone(),
        install_base_url: visible_base.clone(),
        pair_token: pair_token.clone(),
        public_key: public_key.clone(),
    });
    let status_url = api_url(&context.api_base_url, "/api/linux/status");
    let profiles_url = api_url(&context.api_base_url, "/api/linux/profiles");
    let setup_url = api_url(&context.api_base_url, "/api/linux/setup-catalog");

    rsx! {
        div { class: "adui-space adui-space-vertical", style: "display:grid;gap:22px;",
            Card {
                p { class: "adui-typography-secondary", "Operations / Linux" }
                h1 { "Linux" }
                p { "Ubuntu 节点引导计划、环境脚本复用与 SSH 配置预览。" }
            }
            if !snapshot.errors.is_empty() {
                Card { class: "adui-alert adui-alert-error",
                    h2 { "计划错误" }
                    ul {
                        for error in snapshot.errors.iter() {
                            li { "{error}" }
                        }
                    }
                }
            }
            div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:16px;",
                Card {
                    h2 { "客户端状态" }
                    dl {
                        dt { "路由" }
                        dd { "{context.active_route}" }
                        dt { "状态接口" }
                        dd { a { href: status_url.clone(), "{status_url}" } }
                        dt { "契约版本" }
                        dd { code { "{snapshot.status.contract_version}" } }
                        dt { "模式" }
                        dd { "{snapshot.status.mode}" }
                        dt { "服务器 CLI" }
                        dd { "{snapshot.status.server_cli_phase}" }
                        dt { "更新时间" }
                        dd { "{snapshot.status.updated_at_ms}" }
                    }
                }
                Card {
                    h2 { "生成引导计划" }
                    if visible_base.is_empty() {
                        p { class: "adui-empty-description", "当前缺少 api_base_url，填写目标主机后仍不会生成可用远端 curl。" }
                    } else {
                        p { "填写真实目标主机和配对 token 后生成 curl 与 SSH 配置。" }
                    }
                    form {
                        method: "get",
                        action: "/",
                        input { r#type: "hidden", name: "route", value: "{base_route}" }
                        label { "host" }
                        input { name: "host", value: "{host.clone().unwrap_or_default()}", placeholder: "10.0.0.12", required: true }
                        label { "user" }
                        input { name: "user", value: "{user}", required: true }
                        label { "port" }
                        input { name: "port", value: "{port}", r#type: "number", min: "1", max: "65535" }
                        label { "pair token" }
                        input { name: "pair_token", value: "{pair_token.clone().unwrap_or_default()}", required: true }
                        label { "public key" }
                        input { name: "public_key", value: "{public_key.clone().unwrap_or_default()}", placeholder: "ssh-ed25519 ..." }
                        button { r#type: "submit", "生成计划" }
                    }
                }
                Card {
                    h2 { "发行版适配器" }
                    p { a { href: profiles_url.clone(), "{profiles_url}" } }
                    dl {
                        dt { "发行版" }
                        dd { "{snapshot.status.active_profile.label}" }
                        dt { "包管理器" }
                        dd { "{snapshot.status.active_profile.package_manager}" }
                        dt { "默认用户" }
                        dd { "{snapshot.status.active_profile.default_user}" }
                    }
                    if snapshot.status.active_profile.supported_steps.is_empty() {
                        p { class: "adui-empty-description", "当前适配器没有声明步骤。" }
                    } else {
                        ul {
                            for step in snapshot.status.active_profile.supported_steps.iter() {
                                li { code { "{step}" } }
                            }
                        }
                    }
                }
                Card {
                    h2 { "环境脚本目录" }
                    p { a { href: setup_url.clone(), "{setup_url}" } }
                    dl {
                        dt { "来源目录" }
                        dd { code { "{snapshot.catalog.source_root}" } }
                        dt { "命令数" }
                        dd { "{snapshot.catalog.commands.len()}" }
                        dt { "可用" }
                        dd { "{yes_no(snapshot.status.setup_source.available)}" }
                    }
                    ul {
                        for file in snapshot.catalog.source_files.iter() {
                            li {
                                code { "{file.path}" }
                                span { " · {exists_text(file.exists)}" }
                            }
                        }
                    }
                }
                Card {
                    h2 { "复用命令" }
                    if snapshot.catalog.commands.is_empty() {
                        p { class: "adui-empty-description", "当前没有从环境搭建笔记读取到可复用命令。" }
                    } else {
                        table { class: "adui-table",
                            thead {
                                tr {
                                    th { "阶段" }
                                    th { "命令" }
                                    th { "来源" }
                                }
                            }
                            tbody {
                                for command in snapshot.catalog.commands.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{command.stage}" }
                                        td { code { "{command.label}" } }
                                        td { code { "{command.source_path}:{command.source_line}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                Card {
                    h2 { "引导计划" }
                    if let Some(plan) = &snapshot.plan {
                        dl {
                            dt { "目标" }
                            dd { "{plan.target.user}@{plan.target.host}:{plan.target.port}" }
                            dt { "步骤数" }
                            dd { "{plan.steps.len()}" }
                            dt { "curl" }
                            dd { code { "{plan.manual_curl_command}" } }
                        }
                        if !plan.warnings.is_empty() {
                            ul {
                                for warning in plan.warnings.iter() {
                                    li { "{warning}" }
                                }
                            }
                        }
                    } else {
                        p { class: "adui-empty-description", "尚未填写目标主机，不生成引导计划。" }
                    }
                }
                Card {
                    h2 { "SSH 配置" }
                    if let Some(plan) = &snapshot.plan {
                        pre { code { "{plan.ssh_config.config_block}" } }
                        pre { code { "{plan.ssh_config.keygen_command}" } }
                        pre { code { "{plan.ssh_config.authorized_keys_command}" } }
                    } else {
                        p { class: "adui-empty-description", "生成计划后展示 SSH 配置。" }
                    }
                }
            }
        }
    }
}

fn api_url(base: &str, path: &str) -> String {
    if base.trim().is_empty() {
        path.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), path)
    }
}

fn route_without_query(route: &str) -> String {
    route.split('?').next().unwrap_or("/linux").to_string()
}

fn parse_query_param(route: &str, key: &str) -> Option<String> {
    let query = route.split('?').nth(1)?;
    query.split('&').find_map(|pair| {
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
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "是"
    } else {
        "否"
    }
}

fn exists_text(value: bool) -> &'static str {
    if value {
        "存在"
    } else {
        "不存在"
    }
}
