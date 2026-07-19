use serde::{Deserialize, Serialize};

/// Supported Git hosting platforms. The enum is intentionally small for the
/// first pass so provider-specific login behavior stays explicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitHostingProvider {
    GitHub,
    Gitee,
    GitLab,
}

impl GitHostingProvider {
    pub const ALL: [Self; 3] = [Self::GitHub, Self::Gitee, Self::GitLab];

    pub fn id(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Gitee => "gitee",
            Self::GitLab => "gitlab",
        }
    }

    pub fn info(self) -> GitHostingProviderInfo {
        match self {
            Self::GitHub => GitHostingProviderInfo {
                provider: self,
                label: "GitHub",
                host: "github.com",
                web_login_url: "https://github.com/login",
                token_url: "https://github.com/settings/tokens",
                supports_gh_cli: true,
            },
            Self::Gitee => GitHostingProviderInfo {
                provider: self,
                label: "Gitee",
                host: "gitee.com",
                web_login_url: "https://gitee.com/login",
                token_url: "https://gitee.com/profile/personal_access_tokens",
                supports_gh_cli: false,
            },
            Self::GitLab => GitHostingProviderInfo {
                provider: self,
                label: "GitLab",
                host: "gitlab.com",
                web_login_url: "https://gitlab.com/users/sign_in",
                token_url: "https://gitlab.com/-/user_settings/personal_access_tokens",
                supports_gh_cli: false,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GitHostingProviderInfo {
    pub provider: GitHostingProvider,
    pub label: &'static str,
    pub host: &'static str,
    pub web_login_url: &'static str,
    pub token_url: &'static str,
    pub supports_gh_cli: bool,
}
