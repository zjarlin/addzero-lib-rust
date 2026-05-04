use std::future::Future;
use std::pin::Pin;

/// Canonical boxed future alias for service trait methods.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub mod asset_graph;
pub mod auth;
pub mod branding_settings;
pub mod browser_http;
pub mod cli_market;
pub mod in_memory_skills;
pub mod knowledge_entries;
pub mod knowledge_graph;
pub mod logo_storage;
pub mod minio_files;
pub mod openai_chat;
pub mod skills;
pub mod software_catalog;
pub mod system_management;
pub mod terminal_sessions;

pub use asset_graph::{
    AssetGraphDto, AssetGraphEdgeDto, AssetGraphItemDto, AssetGraphTagDto, AssetKindDto,
    AssetSyncReportDto, SharedAssetGraphApi, default_asset_graph_api,
};
pub use auth::{SharedAuthApi, default_auth_api};
pub use branding_settings::{
    BrandingLogoSource, BrandingSettingsDto, BrandingSettingsUpdate, SharedBrandingSettingsApi,
    default_branding_settings_api,
};
pub use cli_market::{SharedCliMarketApi, default_cli_market_api};
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
pub use minio_files::{
    SharedMinioFilesApi, StorageBrowseRequestDto, StorageBrowseResultDto, StorageCreateFolderDto,
    StorageCreateFolderResultDto, StorageDeleteFolderDto, StorageDeleteObjectDto,
    StorageDeleteResultDto, StorageFileDto, StorageFolderDto, StorageShareRequestDto,
    StorageShareResultDto, StorageUploadFileDto, StorageUploadRequestDto, StorageUploadResultDto,
    default_minio_files_api,
};
pub use openai_chat::{
    ChatMessageDto, ChatRequestDto, ChatResponseDto, OpenAiChatConfigDto, SharedOpenAiChatApi,
    default_openai_chat_api,
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
