use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use addzero_plugin_contract::{
    ActorSnapshot, DisplayField, MarketplaceEntry, MarketplaceSnapshot, NavigationItem,
    NavigationItemKind, NavigationSection, PluginCounts, PluginDescriptor, PluginInstance,
    PluginKind, PluginStatus, RecordItem, RuntimeOverview, ShellSnapshot,
};
use addzero_plugin_registry::{PluginRegistry, load_system_descriptors};
use addzero_plugin_runtime::{PluginRuntime, RuntimeError};
use shaku::{Component, HasComponent, Interface, module};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("kernel lock was poisoned")]
    Poisoned,
}

pub trait AuthProvider: Interface {
    fn current_actor(&self) -> ActorSnapshot;
    fn dev_auth_mode(&self) -> String;
}

pub trait RbacService: Interface {
    fn is_allowed(&self, _permission: &str) -> bool;
}

pub trait DictionaryService: Interface {
    fn note_types(&self) -> Vec<DisplayField>;
}

pub trait AuditService: Interface {
    fn seed_entries(&self) -> Vec<RecordItem>;
}

pub trait StorageService: Interface {
    fn package_root_hint(&self) -> String;
}

#[derive(Component)]
#[shaku(interface = AuthProvider)]
struct DevAuthProvider;

impl AuthProvider for DevAuthProvider {
    fn current_actor(&self) -> ActorSnapshot {
        ActorSnapshot {
            username: "admin".to_string(),
            display_name: "平台管理员".to_string(),
            roles: vec!["admin".to_string(), "platform-owner".to_string()],
        }
    }

    fn dev_auth_mode(&self) -> String {
        "dev: admin / 123456".to_string()
    }
}

#[derive(Component)]
#[shaku(interface = RbacService)]
struct AllowAllRbacService;

impl RbacService for AllowAllRbacService {
    fn is_allowed(&self, _permission: &str) -> bool {
        true
    }
}

#[derive(Component)]
#[shaku(interface = DictionaryService)]
struct StaticDictionaryService;

impl DictionaryService for StaticDictionaryService {
    fn note_types(&self) -> Vec<DisplayField> {
        vec![
            DisplayField {
                label: "flash".to_string(),
                value: "闪念".to_string(),
                readonly: true,
            },
            DisplayField {
                label: "note".to_string(),
                value: "笔记".to_string(),
                readonly: true,
            },
            DisplayField {
                label: "knowledge".to_string(),
                value: "知识库".to_string(),
                readonly: true,
            },
            DisplayField {
                label: "skill".to_string(),
                value: "Skill".to_string(),
                readonly: true,
            },
        ]
    }
}

#[derive(Component)]
#[shaku(interface = AuditService)]
struct StaticAuditService;

impl AuditService for StaticAuditService {
    fn seed_entries(&self) -> Vec<RecordItem> {
        vec![
            RecordItem {
                title: "Kernel bootstrapped".to_string(),
                detail: "系统插件注册完成，宿主开始接受业务插件安装。".to_string(),
                meta: "system".to_string(),
            },
            RecordItem {
                title: "RBAC seeded".to_string(),
                detail: "开发环境默认允许 admin 进入所有插件页面。".to_string(),
                meta: "rbac".to_string(),
            },
        ]
    }
}

#[derive(Component)]
#[shaku(interface = StorageService)]
struct LocalStorageService;

impl StorageService for LocalStorageService {
    fn package_root_hint(&self) -> String {
        "target/addzero-plugin-host".to_string()
    }
}

module! {
    KernelModule {
        components = [
            DevAuthProvider,
            AllowAllRbacService,
            StaticDictionaryService,
            StaticAuditService,
            LocalStorageService
        ],
        providers = []
    }
}

pub struct PlatformKernel {
    module: KernelModule,
    runtime: Mutex<PluginRuntime>,
    registry: Mutex<PluginRegistry>,
}

impl PlatformKernel {
    pub fn new(
        catalog_dir: impl Into<PathBuf>,
        package_root: impl Into<PathBuf>,
    ) -> Result<Self, KernelError> {
        let module = KernelModule::builder().build();
        let runtime = PluginRuntime::new(catalog_dir, package_root)?;
        let registry = PluginRegistry::new(load_system_descriptors());
        let kernel = Self {
            module,
            runtime: Mutex::new(runtime),
            registry: Mutex::new(registry),
        };
        kernel.refresh_registry()?;
        Ok(kernel)
    }

    pub fn ensure_dev_package(
        &self,
        source_dir: &Path,
        package_name: &str,
    ) -> Result<PathBuf, KernelError> {
        let runtime = self.runtime.lock().map_err(|_| KernelError::Poisoned)?;
        Ok(runtime.ensure_dev_package(source_dir, package_name)?)
    }

    pub fn install_catalog_plugin(&self, plugin_id: &str) -> Result<PluginDescriptor, KernelError> {
        let mut runtime = self.runtime.lock().map_err(|_| KernelError::Poisoned)?;
        let descriptor = runtime.install_from_catalog(plugin_id)?;
        drop(runtime);
        self.refresh_registry()?;
        Ok(descriptor)
    }

    pub fn create_instance(
        &self,
        plugin_id: &str,
        label: &str,
    ) -> Result<PluginInstance, KernelError> {
        let mut runtime = self.runtime.lock().map_err(|_| KernelError::Poisoned)?;
        let instance = runtime.create_instance(plugin_id, label)?;
        drop(runtime);
        self.refresh_registry()?;
        Ok(instance)
    }

    pub fn shell_snapshot(&self) -> Result<ShellSnapshot, KernelError> {
        let auth: Arc<dyn AuthProvider> = self.module.resolve();
        let registry = self.registry.lock().map_err(|_| KernelError::Poisoned)?;
        let mut nav_sections = vec![NavigationSection {
            label: "工作台".to_string(),
            items: vec![
                NavigationItem {
                    label: "总览".to_string(),
                    href: "/".to_string(),
                    plugin_id: None,
                    page_id: None,
                    badge: None,
                    kind: NavigationItemKind::Fixed,
                },
                NavigationItem {
                    label: "插件市场".to_string(),
                    href: "/marketplace".to_string(),
                    plugin_id: None,
                    page_id: None,
                    badge: Some("Market".to_string()),
                    kind: NavigationItemKind::Fixed,
                },
                NavigationItem {
                    label: "设置".to_string(),
                    href: "/settings".to_string(),
                    plugin_id: None,
                    page_id: None,
                    badge: None,
                    kind: NavigationItemKind::Fixed,
                },
            ],
        }];
        nav_sections.extend(registry.plugin_navigation());
        Ok(ShellSnapshot {
            actor: auth.current_actor(),
            nav_sections,
            counts: PluginCounts {
                system_plugins: registry.system_plugins().len(),
                installed_business_plugins: registry.business_plugins().len(),
                plugin_instances: registry.instances().len(),
            },
            dev_auth_mode: auth.dev_auth_mode(),
        })
    }

    pub fn marketplace_snapshot(&self) -> Result<MarketplaceSnapshot, KernelError> {
        let runtime = self.runtime.lock().map_err(|_| KernelError::Poisoned)?;
        let mut snapshot = runtime.marketplace_snapshot();
        drop(runtime);

        let registry = self.registry.lock().map_err(|_| KernelError::Poisoned)?;
        let mut entries: Vec<_> = registry
            .system_plugins()
            .into_iter()
            .map(|descriptor| MarketplaceEntry {
                plugin_id: descriptor.id,
                name: descriptor.name,
                version: descriptor.version,
                kind: PluginKind::System,
                summary: descriptor.summary,
                tags: descriptor.tags,
                icon: descriptor.icon,
                compatibility: descriptor.compatibility,
                capabilities: descriptor.capabilities,
                status: PluginStatus::Installed,
                instances: 0,
            })
            .collect();
        entries.extend(snapshot.entries);
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        snapshot.entries = entries;
        Ok(snapshot)
    }

    pub fn resolve_system_page(
        &self,
        plugin_id: &str,
        page_id: &str,
    ) -> Result<Option<addzero_plugin_contract::ResolvedPage>, KernelError> {
        let registry = self.registry.lock().map_err(|_| KernelError::Poisoned)?;
        Ok(registry.resolve_system_page(plugin_id, page_id))
    }

    pub fn resolve_instance_page(
        &self,
        instance_slug: &str,
        page_id: &str,
    ) -> Result<Option<addzero_plugin_contract::ResolvedPage>, KernelError> {
        let registry = self.registry.lock().map_err(|_| KernelError::Poisoned)?;
        Ok(registry.resolve_instance_page(instance_slug, page_id))
    }

    pub fn runtime_overview(&self) -> Result<RuntimeOverview, KernelError> {
        let auth: Arc<dyn AuthProvider> = self.module.resolve();
        let runtime = self.runtime.lock().map_err(|_| KernelError::Poisoned)?;
        let registry = self.registry.lock().map_err(|_| KernelError::Poisoned)?;
        Ok(RuntimeOverview {
            counts: PluginCounts {
                system_plugins: registry.system_plugins().len(),
                installed_business_plugins: registry.business_plugins().len(),
                plugin_instances: registry.instances().len(),
            },
            package_root: runtime.package_root().display().to_string(),
            dev_auth_mode: auth.dev_auth_mode(),
        })
    }

    pub fn dictionary_entries(&self) -> Vec<DisplayField> {
        let dictionary: Arc<dyn DictionaryService> = self.module.resolve();
        dictionary.note_types()
    }

    pub fn audit_entries(&self) -> Vec<RecordItem> {
        let audit: Arc<dyn AuditService> = self.module.resolve();
        audit.seed_entries()
    }

    fn refresh_registry(&self) -> Result<(), KernelError> {
        let runtime = self.runtime.lock().map_err(|_| KernelError::Poisoned)?;
        let mut registry = self.registry.lock().map_err(|_| KernelError::Poisoned)?;
        registry.replace_business_plugins(runtime.installed_descriptors());
        registry.replace_instances(runtime.instances());
        Ok(())
    }
}
