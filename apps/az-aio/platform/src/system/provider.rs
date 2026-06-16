//! Aggregate admin provider for system-domain shell content.

use crate::{
    plugin::api::{ContributionSet, NavItemContribution, PageContribution},
    system::{
        catalog::{
            SYSTEM_DOMAIN_ID, SYSTEM_DOMAIN_LABEL, starter_backed_system_features,
            system_feature_views,
        },
        navigation::{AdminSectionSnapshot, registered_admin_sections},
    },
};

#[derive(Clone, Debug, Default)]
pub struct AdminProvider;

impl AdminProvider {
    pub fn system_sections(&self) -> Vec<AdminSectionSnapshot> {
        registered_admin_sections()
    }

    pub fn system_feature_views(&self) -> Vec<crate::system::catalog::SystemFeatureView> {
        system_feature_views()
    }

    pub fn contributions(&self) -> ContributionSet {
        let mut nav_items = Vec::new();
        let mut pages = Vec::new();

        for feature in starter_backed_system_features() {
            nav_items.push(NavItemContribution {
                id: format!("{SYSTEM_DOMAIN_ID}.{}.nav", feature.id),
                label: feature.label.to_string(),
                icon: feature.icon.to_string(),
                route: feature.route.to_string(),
                order: 1_000 + feature.order,
            });
            pages.push(PageContribution {
                route: feature.route.to_string(),
                title: format!("{SYSTEM_DOMAIN_LABEL} · {}", feature.label),
                subtitle: feature.description.to_string(),
                renderer_id: "system.placeholder".to_string(),
                placeholder_mark: feature.icon.to_string(),
                order: 1_000 + feature.order,
            });
        }

        ContributionSet {
            nav_items,
            pages,
            ..Default::default()
        }
    }
}

pub fn system_contributions() -> ContributionSet {
    AdminProvider.contributions()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_exports_only_implemented_system_routes_as_contributions() {
        let contributions = AdminProvider.contributions();
        let routes = contributions
            .nav_items
            .iter()
            .map(|item| item.route.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            routes,
            vec![
                "/system/identity/users",
                "/system/organization/departments",
                "/system/dictionary/note-types",
                "/system/menu/mounting",
                "/system/audit/events",
            ]
        );
        assert!(!routes.contains(&"/system/oauth2/clients"));
    }
}
