//! 插件系统的运行时内核，负责插件生命周期管理、服务注入和前端 Shell 数据聚合。
//!
//! 本 crate 提供 [`PlatformKernel`] 作为插件系统的统一入口，核心职责包括：
//! - 插件安装与实例管理：从目录安装插件、创建运行时实例、刷新目录
//! - Shell 数据聚合：组合当前用户信息、导航树、插件计数，生成 [`ShellSnapshot`]
//! - 市场数据聚合：合并系统插件与业务插件，生成 [`MarketplaceSnapshot`]
//! - 页面解析：根据 `plugin_id` + `page_id` 或 `instance_slug` + `page_id` 解析页面
//!
//! 服务注入基于 `shaku` 框架，预置五个核心服务接口：
//! - [`AuthProvider`]：当前用户身份与认证模式
//! - [`RbacService`]：权限校验
//! - [`DictionaryService`]：字典数据（笔记类型等）
//! - [`AuditService`]：审计日志种子条目
//! - [`StorageService`]：存储路径提示
//!
//! 开发环境提供默认实现：`DevAuthProvider`（admin/admin）、`AllowAllRbacService`（全放行）等。

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use az_derive_aliases::{apply, error, plain_default_eq, plain_eq};
use az_plugin_contract::{
    ActorSnapshot, MarketplaceEntry, MarketplaceSnapshot, PluginCounts, PluginDescriptor,
    PluginInstance, PluginKind, ResolvedPage, RuntimeOverview, ShellSnapshot,
};
use az_plugin_registry::{PluginRegistry, load_system_descriptors};
use az_plugin_runtime::{PluginRuntime, RuntimeError};

pub type KernelResult<T> = Result<T, KernelError>;

#[apply(error)]
pub enum KernelError {
    #[error("{0}")]
    Runtime(#[from] RuntimeError),
    #[error("kernel lock poisoned: {0}")]
    LockPoisoned(&'static str),
}

pub trait AuthProvider: Send + Sync {
    fn actor(&self) -> ActorSnapshot;
    fn dev_auth_mode(&self) -> String;
}

pub trait RbacService: Send + Sync {
    fn can_access(&self, actor: &ActorSnapshot, permission: &str) -> bool;
}

pub trait DictionaryService: Send + Sync {
    fn labels(&self, namespace: &str) -> Vec<String>;
}

#[apply(plain_default_eq)]
pub struct AuditRecord {
    pub action: String,
    pub subject: String,
    pub message: String,
}

pub trait AuditService: Send + Sync {
    fn record(&self, record: AuditRecord);
}

pub trait StorageService: Send + Sync {
    fn package_root_hint(&self) -> String;
}

#[apply(plain_eq)]
pub struct DevAuthProvider {
    actor: ActorSnapshot,
    dev_auth_mode: String,
}

impl Default for DevAuthProvider {
    fn default() -> Self {
        Self {
            actor: ActorSnapshot {
                username: "admin".to_string(),
                display_name: "Admin".to_string(),
                roles: vec!["admin".to_string()],
            },
            dev_auth_mode: "dev".to_string(),
        }
    }
}

impl DevAuthProvider {
    pub fn new(actor: ActorSnapshot, dev_auth_mode: impl Into<String>) -> Self {
        Self {
            actor,
            dev_auth_mode: dev_auth_mode.into(),
        }
    }
}

impl AuthProvider for DevAuthProvider {
    fn actor(&self) -> ActorSnapshot {
        self.actor.clone()
    }

    fn dev_auth_mode(&self) -> String {
        self.dev_auth_mode.clone()
    }
}

#[apply(plain_default_eq)]
pub struct AllowAllRbacService;

impl RbacService for AllowAllRbacService {
    fn can_access(&self, _actor: &ActorSnapshot, _permission: &str) -> bool {
        true
    }
}

#[apply(plain_default_eq)]
pub struct EmptyDictionaryService;

impl DictionaryService for EmptyDictionaryService {
    fn labels(&self, _namespace: &str) -> Vec<String> {
        Vec::new()
    }
}

#[apply(plain_default_eq)]
pub struct NoopAuditService;

impl AuditService for NoopAuditService {
    fn record(&self, _record: AuditRecord) {}
}

#[apply(plain_default_eq)]
pub struct LocalStorageService {
    package_root_hint: String,
}

impl LocalStorageService {
    pub fn new(package_root_hint: impl Into<String>) -> Self {
        Self {
            package_root_hint: package_root_hint.into(),
        }
    }
}

impl StorageService for LocalStorageService {
    fn package_root_hint(&self) -> String {
        self.package_root_hint.clone()
    }
}

#[derive(Clone)]
pub struct KernelServices {
    auth: Arc<dyn AuthProvider>,
    rbac: Arc<dyn RbacService>,
    dictionary: Arc<dyn DictionaryService>,
    audit: Arc<dyn AuditService>,
    storage: Arc<dyn StorageService>,
}

impl Default for KernelServices {
    fn default() -> Self {
        Self {
            auth: Arc::new(DevAuthProvider::default()),
            rbac: Arc::new(AllowAllRbacService),
            dictionary: Arc::new(EmptyDictionaryService),
            audit: Arc::new(NoopAuditService),
            storage: Arc::new(LocalStorageService::default()),
        }
    }
}

impl KernelServices {
    pub fn new(
        auth: Arc<dyn AuthProvider>,
        rbac: Arc<dyn RbacService>,
        dictionary: Arc<dyn DictionaryService>,
        audit: Arc<dyn AuditService>,
        storage: Arc<dyn StorageService>,
    ) -> Self {
        Self {
            auth,
            rbac,
            dictionary,
            audit,
            storage,
        }
    }

    pub fn auth(&self) -> &dyn AuthProvider {
        self.auth.as_ref()
    }

    pub fn rbac(&self) -> &dyn RbacService {
        self.rbac.as_ref()
    }

    pub fn dictionary(&self) -> &dyn DictionaryService {
        self.dictionary.as_ref()
    }

    pub fn audit(&self) -> &dyn AuditService {
        self.audit.as_ref()
    }

    pub fn storage(&self) -> &dyn StorageService {
        self.storage.as_ref()
    }

    pub fn with_auth_provider(mut self, provider: Arc<dyn AuthProvider>) -> Self {
        self.auth = provider;
        self
    }

    pub fn with_rbac_service(mut self, service: Arc<dyn RbacService>) -> Self {
        self.rbac = service;
        self
    }

    pub fn with_dictionary_service(mut self, service: Arc<dyn DictionaryService>) -> Self {
        self.dictionary = service;
        self
    }

    pub fn with_audit_service(mut self, service: Arc<dyn AuditService>) -> Self {
        self.audit = service;
        self
    }

    pub fn with_storage_service(mut self, service: Arc<dyn StorageService>) -> Self {
        self.storage = service;
        self
    }
}

pub struct PlatformKernel {
    runtime: RwLock<PluginRuntime>,
    registry: RwLock<PluginRegistry>,
    services: KernelServices,
}

impl PlatformKernel {
    pub fn new(
        catalog_dir: impl Into<PathBuf>,
        package_root: impl Into<PathBuf>,
    ) -> KernelResult<Self> {
        Self::with_services(catalog_dir, package_root, KernelServices::default())
    }

    pub fn with_services(
        catalog_dir: impl Into<PathBuf>,
        package_root: impl Into<PathBuf>,
        services: KernelServices,
    ) -> KernelResult<Self> {
        let runtime = PluginRuntime::new(catalog_dir, package_root)?;
        let mut registry = PluginRegistry::new(load_system_descriptors());
        sync_registry_from_runtime(&runtime, &mut registry);
        Ok(Self {
            runtime: RwLock::new(runtime),
            registry: RwLock::new(registry),
            services,
        })
    }

    pub fn services(&self) -> &KernelServices {
        &self.services
    }

    pub fn refresh_catalog(&self) -> KernelResult<()> {
        let mut runtime = self.write_runtime()?;
        runtime.refresh_catalog()?;
        self.sync_registry_from_runtime(&runtime)
    }

    pub fn install_catalog_plugin(&self, plugin_id: &str) -> KernelResult<PluginDescriptor> {
        let mut runtime = self.write_runtime()?;
        let descriptor = runtime.install_from_catalog(plugin_id)?;
        self.sync_registry_from_runtime(&runtime)?;
        self.services.audit().record(AuditRecord {
            action: "install_catalog_plugin".to_string(),
            subject: plugin_id.to_string(),
            message: descriptor.name.clone(),
        });
        Ok(descriptor)
    }

    pub fn create_instance(&self, plugin_id: &str, label: &str) -> KernelResult<PluginInstance> {
        let mut runtime = self.write_runtime()?;
        let instance = runtime.create_instance(plugin_id, label)?;
        self.sync_registry_from_runtime(&runtime)?;
        self.services.audit().record(AuditRecord {
            action: "create_instance".to_string(),
            subject: instance.slug.clone(),
            message: instance.label.clone(),
        });
        Ok(instance)
    }

    pub fn ensure_dev_package(
        &self,
        source_dir: impl AsRef<Path>,
        package_name: &str,
    ) -> KernelResult<PathBuf> {
        let package_path = {
            let runtime = self.read_runtime()?;
            runtime.ensure_dev_package(source_dir.as_ref(), package_name)?
        };
        self.refresh_catalog()?;
        Ok(package_path)
    }

    pub fn shell_snapshot(&self) -> KernelResult<ShellSnapshot> {
        let registry = self.read_registry()?;
        let actor = self.services.auth().actor();
        let mut nav_sections = registry.plugin_navigation();
        for section in &mut nav_sections {
            section.items.retain(|item| {
                item.plugin_id
                    .as_deref()
                    .map(|plugin_id| {
                        self.services
                            .rbac()
                            .can_access(&actor, &format!("plugin:{plugin_id}:read"))
                    })
                    .unwrap_or(true)
            });
        }
        Ok(ShellSnapshot {
            actor,
            nav_sections,
            counts: counts_from_registry(&registry),
            dev_auth_mode: self.services.auth().dev_auth_mode(),
        })
    }

    pub fn marketplace_snapshot(&self) -> KernelResult<MarketplaceSnapshot> {
        let runtime_snapshot = {
            let runtime = self.read_runtime()?;
            runtime.marketplace_snapshot()
        };
        let registry = self.read_registry()?;
        let mut entries = registry
            .marketplace_entries()
            .into_iter()
            .filter(|entry| entry.kind == PluginKind::System)
            .collect::<Vec<_>>();
        entries.extend(runtime_snapshot.entries);
        entries.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.plugin_id.cmp(&right.plugin_id))
        });
        Ok(MarketplaceSnapshot {
            tags: collect_tags(&entries),
            entries,
        })
    }

    pub fn runtime_overview(&self) -> KernelResult<RuntimeOverview> {
        let package_root = {
            let runtime = self.read_runtime()?;
            runtime.package_root().display().to_string()
        };
        let registry = self.read_registry()?;
        Ok(RuntimeOverview {
            counts: counts_from_registry(&registry),
            package_root,
            dev_auth_mode: self.services.auth().dev_auth_mode(),
        })
    }

    pub fn resolve_system_page(
        &self,
        plugin_id: &str,
        page_id: &str,
    ) -> KernelResult<Option<ResolvedPage>> {
        Ok(self
            .read_registry()?
            .resolve_system_page(plugin_id, page_id))
    }

    pub fn resolve_instance_page(
        &self,
        instance_slug: &str,
        page_id: &str,
    ) -> KernelResult<Option<ResolvedPage>> {
        Ok(self
            .read_registry()?
            .resolve_instance_page(instance_slug, page_id))
    }

    fn read_runtime(&self) -> KernelResult<RwLockReadGuard<'_, PluginRuntime>> {
        self.runtime
            .read()
            .map_err(|_| KernelError::LockPoisoned("runtime"))
    }

    fn write_runtime(&self) -> KernelResult<RwLockWriteGuard<'_, PluginRuntime>> {
        self.runtime
            .write()
            .map_err(|_| KernelError::LockPoisoned("runtime"))
    }

    fn read_registry(&self) -> KernelResult<RwLockReadGuard<'_, PluginRegistry>> {
        self.registry
            .read()
            .map_err(|_| KernelError::LockPoisoned("registry"))
    }

    fn write_registry(&self) -> KernelResult<RwLockWriteGuard<'_, PluginRegistry>> {
        self.registry
            .write()
            .map_err(|_| KernelError::LockPoisoned("registry"))
    }

    fn sync_registry_from_runtime(&self, runtime: &PluginRuntime) -> KernelResult<()> {
        let mut registry = self.write_registry()?;
        sync_registry_from_runtime(runtime, &mut registry);
        Ok(())
    }
}

fn sync_registry_from_runtime(runtime: &PluginRuntime, registry: &mut PluginRegistry) {
    registry.replace_business_plugins(runtime.installed_descriptors());
    registry.replace_instances(runtime.instances());
}

fn counts_from_registry(registry: &PluginRegistry) -> PluginCounts {
    PluginCounts {
        system_plugins: registry.system_plugins().len(),
        installed_business_plugins: registry.business_plugins().len(),
        plugin_instances: registry.instances().len(),
    }
}

fn collect_tags(entries: &[MarketplaceEntry]) -> Vec<String> {
    entries
        .iter()
        .flat_map(|entry| entry.tags.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::PlatformKernel;

    #[test]
    fn kernel_should_build_default_shell_snapshot() {
        let root = unique_temp_root("shell");
        let kernel = PlatformKernel::new(root.join("catalog"), root.join("packages"))
            .expect("kernel should initialize with empty plugin dirs");

        let snapshot = kernel
            .shell_snapshot()
            .expect("empty kernel should still produce a shell snapshot");

        assert_eq!(snapshot.actor.username, "admin");
        assert_eq!(snapshot.dev_auth_mode, "dev");
        assert_eq!(snapshot.counts.plugin_instances, 0);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn runtime_overview_should_report_package_root() {
        let root = unique_temp_root("overview");
        let package_root = root.join("packages");
        let kernel = PlatformKernel::new(root.join("catalog"), &package_root)
            .expect("kernel should initialize with empty plugin dirs");

        let overview = kernel
            .runtime_overview()
            .expect("empty kernel should expose runtime overview");

        assert_eq!(overview.package_root, package_root.display().to_string());
        fs::remove_dir_all(root).ok();
    }

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("az-plugin-kernel-{label}-{nanos}"))
    }
}
