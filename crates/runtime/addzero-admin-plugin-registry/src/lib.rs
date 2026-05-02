pub use inventory;

#[derive(Clone, Copy, Debug)]
pub struct AdminDomainRegistration {
    pub id: &'static str,
    pub label: &'static str,
    pub order: u16,
    pub default_href: &'static str,
}

impl PartialEq for AdminDomainRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.order == other.order
            && self.default_href == other.default_href
    }
}

impl Eq for AdminDomainRegistration {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdminPageRegistration {
    pub domain_id: &'static str,
    pub label: &'static str,
    pub order: u16,
    pub href: &'static str,
    pub active_patterns: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredAdminMenu {
    pub label: &'static str,
    pub href: &'static str,
    pub active_patterns: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredAdminSection {
    pub label: &'static str,
    pub menus: Vec<RegisteredAdminMenu>,
}

inventory::collect!(AdminDomainRegistration);
inventory::collect!(AdminPageRegistration);

pub fn primary_domain() -> Option<AdminDomainRegistration> {
    registered_domains().into_iter().next()
}

pub fn domain_for_path(path: &str) -> Option<AdminDomainRegistration> {
    let page = active_page(path)?;
    domain_by_id(page.domain_id)
}

pub fn registered_domains() -> Vec<AdminDomainRegistration> {
    let mut domains: Vec<_> = inventory::iter::<AdminDomainRegistration>
        .into_iter()
        .copied()
        .collect();
    domains.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(right.label))
            .then(left.id.cmp(right.id))
    });
    domains.retain(|domain| has_pages_for_domain(domain.id));
    domains
}

pub fn section_for_path(path: &str) -> Option<RegisteredAdminSection> {
    let domain = domain_for_path(path)?;
    Some(RegisteredAdminSection {
        label: domain.label,
        menus: pages_for_domain(domain.id)
            .into_iter()
            .map(|page| RegisteredAdminMenu {
                label: page.label,
                href: page.href,
                active_patterns: page.active_patterns,
            })
            .collect(),
    })
}

pub fn path_matches_patterns(path: &str, patterns: &[&str]) -> bool {
    let path = normalize_path(path);
    patterns
        .iter()
        .copied()
        .any(|pattern| path_matches_pattern(&path, pattern))
}

fn active_page(path: &str) -> Option<AdminPageRegistration> {
    all_pages()
        .into_iter()
        .find(|page| path_matches_patterns(path, page.active_patterns))
}

fn domain_by_id(domain_id: &str) -> Option<AdminDomainRegistration> {
    inventory::iter::<AdminDomainRegistration>
        .into_iter()
        .copied()
        .find(|domain| domain.id == domain_id)
}

fn all_pages() -> Vec<AdminPageRegistration> {
    let mut pages: Vec<_> = inventory::iter::<AdminPageRegistration>
        .into_iter()
        .copied()
        .filter(|page| domain_by_id(page.domain_id).is_some())
        .collect();
    pages.sort_by(|left, right| {
        domain_sort_key(left.domain_id)
            .cmp(&domain_sort_key(right.domain_id))
            .then(left.order.cmp(&right.order))
            .then(left.label.cmp(right.label))
    });
    pages
}

fn pages_for_domain(domain_id: &str) -> Vec<AdminPageRegistration> {
    all_pages()
        .into_iter()
        .filter(|page| page.domain_id == domain_id)
        .collect()
}

fn has_pages_for_domain(domain_id: &str) -> bool {
    inventory::iter::<AdminPageRegistration>
        .into_iter()
        .any(|page| page.domain_id == domain_id)
}

fn domain_sort_key(domain_id: &'static str) -> (u16, &'static str, &'static str) {
    if let Some(domain) = domain_by_id(domain_id) {
        (domain.order, domain.label, domain.id)
    } else {
        (u16::MAX, "", domain_id)
    }
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    let path = split_segments(path);
    let pattern = split_segments(pattern);
    if path.len() != pattern.len() {
        return false;
    }

    path.iter()
        .zip(pattern.iter())
        .all(|(segment, matcher)| matcher.starts_with(':') || segment == matcher)
}

fn split_segments(path: &str) -> Vec<String> {
    let normalized = normalize_path(path);
    if normalized == "/" {
        return Vec::new();
    }

    normalized
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_path(path: &str) -> String {
    let without_hash = path.split('#').next().unwrap_or(path);
    let without_query = without_hash.split('?').next().unwrap_or(without_hash);
    let trimmed = without_query.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_string()
    } else {
        format!("/{}", trimmed.trim_matches('/'))
    }
}

#[macro_export]
macro_rules! register_admin_domain {
    (
        id: $id:expr,
        label: $label:expr,
        order: $order:expr,
        default_href: $default_href:expr $(,)?
    ) => {
        $crate::inventory::submit! {
            $crate::AdminDomainRegistration {
                id: $id,
                label: $label,
                order: $order,
                default_href: $default_href,
            }
        }
    };
}

#[macro_export]
macro_rules! register_admin_page {
    (
        domain: $domain_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr $(,)?
    ) => {
        $crate::inventory::submit! {
            $crate::AdminPageRegistration {
                domain_id: $domain_id,
                label: $label,
                order: $order,
                href: $href,
                active_patterns: $active_patterns,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{normalize_path, path_matches_patterns};

    #[test]
    fn path_matching_should_support_dynamic_segments() {
        assert!(path_matches_patterns("/agents/demo", &["/agents/:name"]));
        assert!(path_matches_patterns(
            "/api/admin/system/users/42",
            &["/api/admin/system/users/:id"]
        ));
        assert!(!path_matches_patterns(
            "/agents/demo/edit",
            &["/agents/:name"]
        ));
    }

    #[test]
    fn normalize_path_should_strip_query_and_trailing_slash() {
        assert_eq!(normalize_path("/files/?tab=recent"), "/files");
        assert_eq!(normalize_path(" / "), "/");
    }
}
