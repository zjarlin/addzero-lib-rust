use std::future::Future;
use std::pin::Pin;

/// Canonical boxed future alias for service trait methods.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub mod ai_chat;
pub mod asset_graph;
pub mod auth;
pub mod branding_settings;
pub mod browser_http;
pub mod cli_market;
#[cfg(not(target_arch = "wasm32"))]
pub mod cloudflare_tunnel;
pub mod desktop_bootstrap;
pub mod download_station;
pub mod in_memory_skills;
pub mod knowledge_entries;
pub mod knowledge_graph;
pub mod logo_storage;
pub mod menu_system;
pub mod platform_config;
pub mod shell_components;
pub mod skills;
pub mod software_catalog;
pub mod system_management;
pub mod terminal_sessions;
pub mod vibe_coding;

pub use ai_chat::{
    AiProviderConfigDto, AiProviderConfigUpsertDto, AiProviderKindDto, ChatMessageDto,
    ChatRequestDto, ChatResponseDto,
};
pub use asset_graph::{
    AssetGraphDto, AssetGraphEdgeDto, AssetGraphItemDto, AssetGraphTagDto, AssetKindDto,
    AssetSyncReportDto, SharedAssetGraphApi, default_asset_graph_api,
};
pub use auth::{SharedAuthApi, default_auth_api};
pub use az_config_center_contract::{
    ShellComponent, ShellComponentBuildConfig, ShellComponentBuildRequest,
    ShellComponentBuildResult, ShellComponentConfigUpdate, ShellComponentKind, ShellComponentPatch,
    ShellComponentRegistry, ShellComponentRemove, ShellComponentUpsert,
};
pub use branding_settings::{
    BrandingLogoSource, BrandingSettingsDto, BrandingSettingsUpdate, SharedBrandingSettingsApi,
    default_branding_settings_api,
};
pub use cli_market::{SharedCliMarketApi, default_cli_market_api};
#[cfg(not(target_arch = "wasm32"))]
pub use cloudflare_tunnel::{
    CloudflareTunnelCliCommandDto, CloudflareTunnelHostDto, CloudflareTunnelStatusDto,
};
pub use desktop_bootstrap::{
    BootstrapDatabaseSaveResultDto, BootstrapDatabaseSetupDto, BootstrapPlatformSaveResultDto,
    BootstrapPlatformSetupDto, BootstrapStatusDto,
};
pub use download_station::{
    FileIndexDto, FilterOptions, ScanStatsDto, ShareLinkDto, SharedDownloadStationApi,
    default_download_station_api,
};
pub use in_memory_skills::InMemorySkillsApi;
pub use knowledge_entries::{
    KnowledgeEntryDeleteDto, KnowledgeEntryUpsertDto, KnowledgeNoteDto, SharedKnowledgeEntriesApi,
    default_knowledge_entries_api,
};
pub use knowledge_graph::{
    IngestKnowledgeRawInput, KnowledgeExceptionCardDto, KnowledgeFeedDto,
    KnowledgeMaintenanceReportDto, KnowledgeNodeDetailDto, KnowledgeNodeSummaryDto,
    KnowledgeSourceRefDto, ResolveKnowledgeExceptionInput, SharedKnowledgeGraphApi,
    default_knowledge_graph_api,
};
pub use logo_storage::{
    LOGO_PREVIEW_BASE_URL, LogoUploadRequest, SharedLogoStorageApi, StoredLogoDto,
    build_preview_url, default_logo_storage_api,
};
pub use menu_system::{
    CreateMenuRequest, Menu, MenuService, MenuTreeNode, Permission, UpdateMenuRequest,
};
pub use platform_config::{
    PlatformConfigDto, PlatformConfigSaveResultDto, PostgresConfigDto, PostgresConfigUpdateDto,
};
pub use shell_components::{
    build_shell_components_on_server, current_shell_component_output_config,
    current_shell_component_registry_path, get_shell_component_on_server,
    load_shell_component_registry_on_server, patch_shell_component_on_server,
    remove_shell_component_on_server, save_shell_component_config_on_server,
    upsert_shell_component_on_server,
};
pub use skills::{
    SharedSkillsApi, SkillDto, SkillSourceDto, SkillUpsertDto, SyncReportDto, default_skills_api,
};
pub use software_catalog::{SharedSoftwareCatalogApi, default_software_catalog_api};
pub use system_management::{
    AuthorizeRoleMenusDto, AuthorizeUserRolesDto, DepartmentDto, DepartmentUpsertDto, DictGroupDto,
    DictGroupUpsertDto, DictItemDto, DictItemUpsertDto, MenuDto, MenuUpsertDto, RoleDto,
    RoleUpsertDto, RoleWithMenusDto, SharedSystemManagementApi, UserDto, UserUpsertDto,
    UserWithRolesDto, default_system_management_api,
};
pub use terminal_sessions::{
    SharedTerminalSessionsApi, TerminalProfileDto, TerminalSessionCreateDto,
    TerminalSessionInputDto, TerminalSessionListDto, TerminalSessionResizeDto,
    TerminalSessionSnapshotDto, TerminalSessionStateDto, TerminalSessionSummaryDto,
    default_terminal_sessions_api,
};
pub use vibe_coding::{StartVibeCodingRequestDto, StartVibeCodingResponseDto};
