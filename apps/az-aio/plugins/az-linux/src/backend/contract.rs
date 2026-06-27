//! Linux 客户端侧操作契约。
//!
//! 这里先定义 AIO 客户端生成计划、脚本和 SSH 配置所需的数据模型；后续服务器版
//! CLI 应复用这些字段语义，避免 REST、CLI 与 admin 页面各自维护一套漂移接口。

use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: &str = "linux.client.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinuxDistribution {
    Ubuntu,
}

impl LinuxDistribution {
    pub fn id(self) -> &'static str {
        match self {
            Self::Ubuntu => "ubuntu",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ubuntu => "Ubuntu",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxProfileSummary {
    pub distribution: LinuxDistribution,
    pub label: String,
    pub package_manager: String,
    pub default_user: String,
    pub supported_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxTarget {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub distribution: LinuxDistribution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientPairingSeed {
    pub client_name: String,
    pub client_endpoint: String,
    pub pair_token: String,
    pub public_key: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPlanRequest {
    pub target: LinuxTarget,
    pub client: ClientPairingSeed,
    pub install_base_url: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPlan {
    pub contract_version: String,
    pub target: LinuxTarget,
    pub client: ClientPairingSeed,
    pub manual_curl_command: String,
    pub ssh_config: SshConfigPreview,
    pub steps: Vec<BootstrapStep>,
    pub warnings: Vec<String>,
    pub setup_source: EnvironmentSetupSourceSummary,
    pub reused_commands: Vec<EnvironmentSetupCommand>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStep {
    pub id: String,
    pub label: String,
    pub description: String,
    pub command: Option<String>,
    pub manual: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConfigPreview {
    pub host_alias: String,
    pub config_block: String,
    pub authorized_keys_command: String,
    pub keygen_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxClientStatusResponse {
    pub ok: bool,
    pub contract_version: String,
    pub mode: String,
    pub server_cli_phase: String,
    pub active_profile: LinuxProfileSummary,
    pub setup_source: EnvironmentSetupSourceSummary,
    pub endpoints: Vec<LinuxEndpoint>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxEndpoint {
    pub method: String,
    pub path: String,
    pub label: String,
    pub description: String,
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSetupCatalog {
    pub source_root: String,
    pub source_files: Vec<EnvironmentSetupSourceFile>,
    pub commands: Vec<EnvironmentSetupCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSetupSourceSummary {
    pub source_root: String,
    pub source_files: Vec<EnvironmentSetupSourceFile>,
    pub command_count: usize,
    pub available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSetupSourceFile {
    pub path: String,
    pub exists: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSetupCommand {
    pub id: String,
    pub label: String,
    pub stage: String,
    pub command: String,
    pub source_path: String,
    pub source_line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotePairingFile {
    pub contract_version: String,
    pub distribution: LinuxDistribution,
    pub client_name: String,
    pub client_endpoint: String,
    pub pair_token: String,
    pub generated_at_ms: u64,
}

pub fn endpoint_catalog() -> Vec<LinuxEndpoint> {
    vec![
        endpoint(
            "GET",
            "/api/linux/status",
            "客户端状态",
            "返回 Linux 客户端插件的契约版本、模式和可用端点。",
        ),
        endpoint(
            "GET",
            "/api/linux/profiles",
            "发行版适配器",
            "返回当前客户端支持的 Linux 发行版适配器，先实现 Ubuntu。",
        ),
        endpoint(
            "GET",
            "/api/linux/setup-catalog",
            "环境搭建脚本目录",
            "从 /Users/zjarlin/aio/note/环境搭建 读取可复用脚本命令，避免重复造轮子。",
        ),
        endpoint(
            "POST",
            "/api/linux/bootstrap-plan",
            "生成引导计划",
            "根据目标主机、客户端配对种子和安装入口生成 curl 引导与 SSH 配置预览。",
        ),
        endpoint(
            "GET",
            "/api/linux/bootstrap-script",
            "远端 curl 脚本",
            "供 Ubuntu 服务器手动 curl 执行，完成基础包、SSH 服务和配对种子落盘。",
        ),
    ]
}

fn endpoint(method: &str, path: &str, label: &str, description: &str) -> LinuxEndpoint {
    LinuxEndpoint {
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: description.to_string(),
    }
}
