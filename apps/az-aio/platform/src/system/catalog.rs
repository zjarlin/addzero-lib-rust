//! Yudao-inspired system feature catalog.

use serde::{Deserialize, Serialize};

pub const SYSTEM_DOMAIN_ID: &str = "system";
pub const SYSTEM_DOMAIN_LABEL: &str = "系统";
pub const SYSTEM_DEFAULT_ROUTE: &str = "/system/identity/users";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemFeatureStatus {
    StarterBacked,
    ReferenceOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemFeature {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub route: &'static str,
    pub icon: &'static str,
    pub order: i32,
    pub status: SystemFeatureStatus,
    pub source_modules: &'static [&'static str],
    pub data_objects: &'static [&'static str],
    pub permissions_any_of: &'static [&'static str],
}

impl SystemFeature {
    pub fn is_starter_backed(self) -> bool {
        self.status == SystemFeatureStatus::StarterBacked
    }

    pub fn view(self) -> SystemFeatureView {
        SystemFeatureView {
            id: self.id.to_string(),
            label: self.label.to_string(),
            description: self.description.to_string(),
            route: self.route.to_string(),
            icon: self.icon.to_string(),
            order: self.order,
            status: self.status,
            source_modules: self
                .source_modules
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            data_objects: self
                .data_objects
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            permissions_any_of: self
                .permissions_any_of
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemFeatureView {
    pub id: String,
    pub label: String,
    pub description: String,
    pub route: String,
    pub icon: String,
    pub order: i32,
    pub status: SystemFeatureStatus,
    pub source_modules: Vec<String>,
    pub data_objects: Vec<String>,
    pub permissions_any_of: Vec<String>,
}

pub fn system_features() -> &'static [SystemFeature] {
    SYSTEM_FEATURES
}

pub fn starter_backed_system_features() -> Vec<SystemFeature> {
    SYSTEM_FEATURES
        .iter()
        .copied()
        .filter(|feature| feature.is_starter_backed())
        .collect()
}

pub fn system_feature_views() -> Vec<SystemFeatureView> {
    SYSTEM_FEATURES
        .iter()
        .copied()
        .map(SystemFeature::view)
        .collect()
}

const SYSTEM_FEATURES: &[SystemFeature] = &[
    SystemFeature {
        id: "identity",
        label: "用户管理",
        description: "Admin users, roles, profile, and login-facing identity records.",
        route: "/system/identity/users",
        icon: "◉",
        order: 10,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["api/user", "service/user", "controller/admin/user"],
        data_objects: &["AdminUserDO", "RoleDO", "UserRoleDO"],
        permissions_any_of: &["system:user"],
    },
    SystemFeature {
        id: "organization",
        label: "部门管理",
        description: "Departments, posts, and user organization bindings.",
        route: "/system/organization/departments",
        icon: "◎",
        order: 20,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["api/dept", "service/dept", "controller/admin/dept"],
        data_objects: &["DeptDO", "PostDO", "UserPostDO"],
        permissions_any_of: &["system:dept"],
    },
    SystemFeature {
        id: "dictionary",
        label: "字典管理",
        description: "Dictionary types and values used by admin and app surfaces.",
        route: "/system/dictionary/note-types",
        icon: "▤",
        order: 30,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["api/dict", "service/dict", "controller/admin/dict"],
        data_objects: &["DictTypeDO", "DictDataDO"],
        permissions_any_of: &["system:dict"],
    },
    SystemFeature {
        id: "menu",
        label: "菜单挂载",
        description: "Admin menu tree, permissions, and route mounting.",
        route: "/system/menu/mounting",
        icon: "☰",
        order: 40,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["api/permission", "service/permission", "controller/admin/permission"],
        data_objects: &["MenuDO", "RoleMenuDO"],
        permissions_any_of: &["system:menu"],
    },
    SystemFeature {
        id: "audit",
        label: "审计日志",
        description: "Login logs and operation logs for admin traceability.",
        route: "/system/audit/events",
        icon: "◷",
        order: 50,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["api/logger", "service/logger", "controller/admin/logger"],
        data_objects: &["LoginLogDO", "OperateLogDO"],
        permissions_any_of: &["system:audit"],
    },
    SystemFeature {
        id: "auth",
        label: "认证中心",
        description: "Admin login, registration, password reset, captcha, and SMS login flows.",
        route: "/system/auth/sessions",
        icon: "●",
        order: 60,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["service/auth", "controller/admin/auth", "framework/security"],
        data_objects: &["OAuth2AccessTokenDO", "OAuth2RefreshTokenDO"],
        permissions_any_of: &["system:auth"],
    },
    SystemFeature {
        id: "tenant",
        label: "租户管理",
        description: "Tenants and tenant packages for multi-tenant admin deployment.",
        route: "/system/tenant/tenants",
        icon: "▥",
        order: 70,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["api/tenant", "service/tenant", "controller/admin/tenant"],
        data_objects: &["TenantDO", "TenantPackageDO"],
        permissions_any_of: &["system:tenant"],
    },
    SystemFeature {
        id: "messaging",
        label: "消息中心",
        description: "Mail, notification, and SMS templates, logs, and send operations.",
        route: "/system/messaging/templates",
        icon: "✉",
        order: 80,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["api/mail", "api/notify", "api/sms", "service/mail", "service/notify", "service/sms"],
        data_objects: &[
            "MailAccountDO",
            "MailTemplateDO",
            "MailLogDO",
            "NotifyTemplateDO",
            "NotifyMessageDO",
            "SmsChannelDO",
            "SmsTemplateDO",
            "SmsLogDO",
        ],
        permissions_any_of: &["system:message"],
    },
    SystemFeature {
        id: "oauth2",
        label: "OAuth2",
        description: "OAuth2 clients, approvals, codes, access tokens, and open user info.",
        route: "/system/oauth2/clients",
        icon: "◇",
        order: 90,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["api/oauth2", "service/oauth2", "controller/admin/oauth2"],
        data_objects: &[
            "OAuth2ClientDO",
            "OAuth2ApproveDO",
            "OAuth2CodeDO",
            "OAuth2AccessTokenDO",
            "OAuth2RefreshTokenDO",
        ],
        permissions_any_of: &["system:oauth2"],
    },
    SystemFeature {
        id: "social",
        label: "社交集成",
        description: "Social clients, user binding, WeChat JSAPI, QR code, and subscribe messages.",
        route: "/system/social/clients",
        icon: "◌",
        order: 100,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["api/social", "service/social", "controller/admin/socail"],
        data_objects: &["SocialClientDO", "SocialUserDO", "SocialUserBindDO"],
        permissions_any_of: &["system:social"],
    },
    SystemFeature {
        id: "area",
        label: "地区数据",
        description: "Area tree and IP location lookup used by admin and app surfaces.",
        route: "/system/area/tree",
        icon: "⌖",
        order: 110,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["controller/admin/ip", "controller/app/ip"],
        data_objects: &[],
        permissions_any_of: &["system:area"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_keeps_yudao_system_slices_visible_as_reference() {
        let ids = system_features()
            .iter()
            .map(|feature| feature.id)
            .collect::<Vec<_>>();

        assert!(ids.contains(&"identity"));
        assert!(ids.contains(&"organization"));
        assert!(ids.contains(&"dictionary"));
        assert!(ids.contains(&"menu"));
        assert!(ids.contains(&"audit"));
        assert!(ids.contains(&"auth"));
        assert!(ids.contains(&"messaging"));
        assert!(ids.contains(&"oauth2"));
        assert!(ids.contains(&"tenant"));
        assert!(ids.contains(&"social"));
    }

    #[test]
    fn visible_features_are_limited_to_current_starter_backed_set() {
        let visible_ids = starter_backed_system_features()
            .iter()
            .map(|feature| feature.id)
            .collect::<Vec<_>>();

        assert_eq!(
            visible_ids,
            vec!["identity", "organization", "dictionary", "menu", "audit"]
        );
    }
}
