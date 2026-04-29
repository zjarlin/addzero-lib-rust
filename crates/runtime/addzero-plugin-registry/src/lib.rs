use std::collections::BTreeMap;

use addzero_plugin_contract::{
    MarketplaceEntry, NavigationItem, NavigationItemKind, NavigationSection, PageScope,
    PluginDescriptor, PluginInstance, PluginKind, PluginMenuContribution, PluginStatus,
    ResolvedPage,
};

pub use inventory;

pub trait PluginStarter: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;
}

pub struct StarterRegistration {
    pub constructor: fn() -> Box<dyn PluginStarter>,
}

inventory::collect!(StarterRegistration);

pub fn load_system_starters() -> Vec<Box<dyn PluginStarter>> {
    inventory::iter::<StarterRegistration>
        .into_iter()
        .map(|registration| (registration.constructor)())
        .collect()
}

pub fn load_system_descriptors() -> Vec<PluginDescriptor> {
    let mut descriptors: Vec<_> = load_system_starters()
        .into_iter()
        .map(|starter| starter.descriptor())
        .collect();
    descriptors.sort_by(|left, right| left.name.cmp(&right.name));
    descriptors
}

#[derive(Clone, Debug, Default)]
pub struct PluginRegistry {
    system: BTreeMap<String, PluginDescriptor>,
    business: BTreeMap<String, PluginDescriptor>,
    instances: BTreeMap<String, PluginInstance>,
}

impl PluginRegistry {
    pub fn new(system_plugins: Vec<PluginDescriptor>) -> Self {
        Self {
            system: map_descriptors(system_plugins),
            business: BTreeMap::new(),
            instances: BTreeMap::new(),
        }
    }

    pub fn replace_business_plugins(&mut self, business_plugins: Vec<PluginDescriptor>) {
        self.business = map_descriptors(business_plugins);
    }

    pub fn replace_instances(&mut self, instances: Vec<PluginInstance>) {
        self.instances = instances
            .into_iter()
            .map(|instance| (instance.slug.clone(), instance))
            .collect();
    }

    pub fn system_plugins(&self) -> Vec<PluginDescriptor> {
        self.system.values().cloned().collect()
    }

    pub fn business_plugins(&self) -> Vec<PluginDescriptor> {
        self.business.values().cloned().collect()
    }

    pub fn instances(&self) -> Vec<PluginInstance> {
        self.instances.values().cloned().collect()
    }

    pub fn marketplace_entries(&self) -> Vec<MarketplaceEntry> {
        let mut entries = Vec::new();
        for descriptor in self.system.values() {
            entries.push(MarketplaceEntry {
                plugin_id: descriptor.id.clone(),
                name: descriptor.name.clone(),
                version: descriptor.version.clone(),
                kind: PluginKind::System,
                summary: descriptor.summary.clone(),
                tags: descriptor.tags.clone(),
                icon: descriptor.icon.clone(),
                compatibility: descriptor.compatibility.clone(),
                capabilities: descriptor.capabilities.clone(),
                status: PluginStatus::Installed,
                instances: 0,
            });
        }
        for descriptor in self.business.values() {
            let instances = self
                .instances
                .values()
                .filter(|instance| instance.plugin_id == descriptor.id)
                .count();
            entries.push(MarketplaceEntry {
                plugin_id: descriptor.id.clone(),
                name: descriptor.name.clone(),
                version: descriptor.version.clone(),
                kind: PluginKind::Business,
                summary: descriptor.summary.clone(),
                tags: descriptor.tags.clone(),
                icon: descriptor.icon.clone(),
                compatibility: descriptor.compatibility.clone(),
                capabilities: descriptor.capabilities.clone(),
                status: PluginStatus::Installed,
                instances,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }

    pub fn plugin_navigation(&self) -> Vec<NavigationSection> {
        let mut sections = Vec::new();
        let mut system_items = Vec::new();
        for descriptor in self.system.values() {
            for menu in sorted_menus(&descriptor.menus) {
                system_items.push(NavigationItem {
                    label: menu.label.clone(),
                    href: format!("/system/{}/{}", descriptor.id, menu.page_id),
                    plugin_id: Some(descriptor.id.clone()),
                    page_id: Some(menu.page_id.clone()),
                    badge: Some("系统".to_string()),
                    kind: NavigationItemKind::SystemPage,
                });
            }
        }
        if !system_items.is_empty() {
            sections.push(NavigationSection {
                label: "系统插件".to_string(),
                items: system_items,
            });
        }

        let mut instance_items = Vec::new();
        for instance in self.instances.values() {
            for page_id in &instance.page_ids {
                let Some(descriptor) = self.business.get(&instance.plugin_id) else {
                    continue;
                };
                let label = descriptor
                    .pages
                    .iter()
                    .find(|page| &page.id == page_id)
                    .map(|page| format!("{} / {}", instance.label, page.title))
                    .unwrap_or_else(|| format!("{} / {}", instance.label, page_id));
                instance_items.push(NavigationItem {
                    label,
                    href: format!("/apps/{}/{}", instance.slug, page_id),
                    plugin_id: Some(instance.plugin_id.clone()),
                    page_id: Some(page_id.clone()),
                    badge: Some(descriptor.name.clone()),
                    kind: NavigationItemKind::BusinessInstance,
                });
            }
        }
        if !instance_items.is_empty() {
            sections.push(NavigationSection {
                label: "业务应用".to_string(),
                items: instance_items,
            });
        }
        sections
    }

    pub fn resolve_system_page(&self, plugin_id: &str, page_id: &str) -> Option<ResolvedPage> {
        let descriptor = self.system.get(plugin_id)?;
        let page = descriptor.pages.iter().find(|page| page.id == page_id)?;
        Some(ResolvedPage {
            scope: PageScope::System,
            plugin_id: descriptor.id.clone(),
            plugin_name: descriptor.name.clone(),
            page_id: page.id.clone(),
            title: page.title.clone(),
            subtitle: page.subtitle.clone(),
            breadcrumbs: vec![
                "系统插件".to_string(),
                descriptor.name.clone(),
                page.title.clone(),
            ],
            schema: page.schema.clone(),
        })
    }

    pub fn resolve_instance_page(
        &self,
        instance_slug: &str,
        page_id: &str,
    ) -> Option<ResolvedPage> {
        let instance = self.instances.get(instance_slug)?;
        let descriptor = self.business.get(&instance.plugin_id)?;
        let page = descriptor.pages.iter().find(|page| page.id == page_id)?;
        Some(ResolvedPage {
            scope: PageScope::Instance,
            plugin_id: descriptor.id.clone(),
            plugin_name: descriptor.name.clone(),
            page_id: page.id.clone(),
            title: format!("{} · {}", instance.label, page.title),
            subtitle: page.subtitle.clone(),
            breadcrumbs: vec![
                "业务应用".to_string(),
                instance.label.clone(),
                page.title.clone(),
            ],
            schema: page.schema.clone(),
        })
    }
}

fn map_descriptors(descriptors: Vec<PluginDescriptor>) -> BTreeMap<String, PluginDescriptor> {
    descriptors
        .into_iter()
        .map(|descriptor| (descriptor.id.clone(), descriptor))
        .collect()
}

fn sorted_menus(menus: &[PluginMenuContribution]) -> Vec<&PluginMenuContribution> {
    let mut sorted: Vec<_> = menus.iter().collect();
    sorted.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(&right.label))
    });
    sorted
}
