export interface SessionUser {
  authenticated: boolean;
  username: string | null;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export type SkillSourceDto = "pg" | "fs" | "both" | "unknown";

export interface SkillDto {
  name: string;
  keywords: string[];
  description: string;
  body: string;
  source: SkillSourceDto;
  updated_at: string;
}

export interface SkillUpsertDto {
  name: string;
  keywords: string[];
  description: string;
  body: string;
}

export interface SyncReportDto {
  synced_count: number;
  skipped_count: number;
  failed_count: number;
  pg_online: boolean;
  fs_root: string;
  status: string;
  detail: string;
}

export interface KnowledgeNoteDto {
  source_path: string;
  relative_path: string;
  title: string;
  preview: string;
  body: string;
  headings: string[];
  updated_at: string;
}

export interface KnowledgeEntryUpsertDto {
  source_path?: string | null;
  relative_path: string;
  body: string;
}

export interface KnowledgeEntryDeleteDto {
  source_path: string;
}

export interface ChatMessageDto {
  role: string;
  content: string;
}

export interface ChatRequestDto {
  messages: ChatMessageDto[];
}

export interface ChatResponseDto {
  message: ChatMessageDto;
}

export type BrandingLogoSource = "app_icon" | "custom_upload" | "text_only";

export interface StoredLogoDto {
  object_key: string;
  relative_path: string;
  file_name: string;
  content_type: string;
  backend_label: string;
}

export interface BrandingSettingsDto {
  site_name: string;
  logo_source: BrandingLogoSource;
  logo: StoredLogoDto | null;
  brand_copy: string;
  header_badge: string;
}

export interface BrandingSettingsUpdate {
  site_name: string;
  logo_source: BrandingLogoSource;
  logo: StoredLogoDto | null;
  brand_copy: string;
  header_badge: string;
}

export interface StorageBrowseRequestDto {
  prefix: string;
}

export interface StorageBreadcrumbDto {
  label: string;
  prefix: string;
}

export interface StorageFolderDto {
  name: string;
  prefix: string;
  relative_path: string;
  object_count: number;
  size_bytes: number;
}

export interface StorageFileDto {
  name: string;
  object_key: string;
  relative_path: string;
  size_bytes: number;
  content_type: string;
  last_modified: string;
  download_path: string;
}

export interface StorageBrowseResultDto {
  bucket: string;
  current_prefix: string;
  parent_prefix: string | null;
  breadcrumbs: StorageBreadcrumbDto[];
  backend_label: string;
  folder_count: number;
  file_count: number;
  folders: StorageFolderDto[];
  files: StorageFileDto[];
}

export interface StorageUploadFileDto {
  file_name: string;
  content_type?: string | null;
  bytes: number[];
}

export interface StorageUploadRequestDto {
  prefix: string;
  files: StorageUploadFileDto[];
}

export interface StorageUploadResultDto {
  uploaded_count: number;
  prefix: string;
  files: StorageFileDto[];
  message: string;
}

export interface StorageCreateFolderDto {
  parent_prefix: string;
  relative_path: string;
}

export interface StorageCreateFolderResultDto {
  prefix: string;
  message: string;
}

export interface StorageShareRequestDto {
  object_key: string;
  expiration_seconds?: number | null;
}

export interface StorageShareResultDto {
  object_key: string;
  relative_path: string;
  presigned_url: string;
  encrypted_url: string | null;
  expires_in_seconds: number;
}

export interface StorageDeleteObjectDto {
  object_key: string;
}

export interface StorageDeleteFolderDto {
  prefix: string;
}

export interface StorageDeleteResultDto {
  deleted_count: number;
  message: string;
}

export interface MenuDto {
  id: number;
  parent_id: number | null;
  name: string;
  code: string;
  path: string;
  sort: number;
  status: string;
}

export interface MenuUpsertDto {
  parent_id?: number | null;
  name: string;
  code: string;
  path: string;
  sort: number;
  status: string;
}

export interface RoleDto {
  id: number;
  name: string;
  code: string;
  status: string;
}

export interface RoleUpsertDto {
  name: string;
  code: string;
  status: string;
}

export interface RoleWithMenusDto {
  role: RoleDto;
  menu_ids: number[];
}

export interface UserDto {
  id: number;
  username: string;
  nickname: string;
  status: string;
}

export interface UserUpsertDto {
  username: string;
  password: string;
  nickname: string;
  status: string;
}

export interface UserWithRolesDto {
  user: UserDto;
  role_ids: number[];
}

export interface AuthorizeRoleMenusDto {
  menu_ids: number[];
}

export interface AuthorizeUserRolesDto {
  role_ids: number[];
}

export interface DepartmentDto {
  id: number;
  parent_id: number | null;
  name: string;
  code: string;
  leader: string;
  sort: number;
  status: string;
}

export interface DepartmentUpsertDto {
  parent_id?: number | null;
  name: string;
  code: string;
  leader: string;
  sort: number;
  status: string;
}

export interface DictGroupDto {
  id: number;
  name: string;
  code: string;
  status: string;
}

export interface DictGroupUpsertDto {
  name: string;
  code: string;
  status: string;
}

export interface DictItemDto {
  id: number;
  group_id: number;
  label: string;
  value: string;
  sort: number;
  status: string;
}

export interface DictItemUpsertDto {
  group_id: number;
  label: string;
  value: string;
  sort: number;
  status: string;
}

export interface CliLocaleText {
  locale: string;
  display_name: string;
  summary: string;
  description_md: string;
  install_guide_md: string;
  docs_summary: string;
  requires_text: string;
  install_command: string;
}

export interface CliInstallMethod {
  id: string | null;
  platform: string;
  installer_kind: string;
  package_id: string;
  command_template: string;
  validation_note: string;
  priority: number;
}

export interface CliDocRef {
  id: string | null;
  locale: string;
  title: string;
  url: string;
  version: string;
  source_label: string;
  summary: string;
}

export interface CliMarketEntry {
  id: string;
  slug: string;
  status: string;
  source_type: string;
  entry_kind: string;
  vendor_name: string;
  latest_version: string;
  homepage_url: string;
  repo_url: string;
  docs_url: string;
  entry_point: string;
  category_code: string;
  tags: string[];
  locales: CliLocaleText[];
  install_methods: CliInstallMethod[];
  doc_refs: CliDocRef[];
}

export interface CliMarketCatalog {
  entries: CliMarketEntry[];
}

export interface CliMarketEntryUpsert {
  id?: string | null;
  slug: string;
  status: string;
  source_type: string;
  entry_kind: string;
  vendor_name: string;
  latest_version: string;
  homepage_url: string;
  repo_url: string;
  docs_url: string;
  entry_point: string;
  category_code: string;
  tags: string[];
  locales: CliLocaleText[];
  install_methods: CliInstallMethod[];
  doc_refs: CliDocRef[];
  raw: unknown;
}

export interface CliMarketImportRequest {
  file_name: string;
  bytes_b64: string;
  format: string;
  mode: string;
}

export interface CliMarketImportJob {
  id: string;
  file_name: string;
  format: string;
  mode: string;
  status: string;
  created_at: string;
}

export interface CliMarketImportJobDetail extends CliMarketImportJob {
  result_summary: string;
}

export interface CliMarketExportRequest {
  include_draft: boolean;
}

export interface CliMarketInstallHistoryItem {
  id: string;
  platform: string;
  status: string;
  created_at: string;
  message: string;
}

export interface CliMarketInstallRequest {
  platform?: string | null;
}

export interface CliMarketInstallResult {
  command: string;
  stdout: string;
  stderr: string;
  exit_code: number;
}

export interface AuditItem {
  title: string;
  detail: string;
  actor: string;
  when: string;
}

export interface PackageCatalogEntry {
  slug: string;
  channel_slug: string;
  software_title: string;
  package_name: string;
  version: string;
  platform: string;
  format: string;
  status: string;
  source: string;
  install_target: string;
  checksum_state: string;
  relation: string;
  note: string;
}

async function request<T>(baseUrl: string, path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, {
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
    ...init,
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `HTTP ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

function jsonBody(body: unknown) {
  return { body: JSON.stringify(body) };
}

export function createApiClient(baseUrl: string) {
  return {
    getSession: () => request<SessionUser>(baseUrl, "/api/admin/session"),
    login: (input: LoginRequest) =>
      request<SessionUser>(baseUrl, "/api/admin/session/login", {
        method: "POST",
        ...jsonBody(input),
      }),
    logout: () =>
      request<SessionUser>(baseUrl, "/api/admin/session/logout", {
        method: "POST",
        ...jsonBody({}),
      }),
    getPermissions: () =>
      request<string[]>(baseUrl, "/api/admin/session/permissions"),
    listSkills: () => request<SkillDto[]>(baseUrl, "/api/skills"),
    getSkill: (name: string) =>
      request<SkillDto | null>(baseUrl, `/api/skills/${encodeURIComponent(name)}`),
    upsertSkill: (input: SkillUpsertDto) =>
      request<SkillDto>(baseUrl, "/api/skills/upsert", {
        method: "POST",
        ...jsonBody(input),
      }),
    deleteSkill: (name: string) =>
      request<void>(baseUrl, `/api/skills/${encodeURIComponent(name)}`, {
        method: "DELETE",
      }),
    syncSkills: () =>
      request<SyncReportDto>(baseUrl, "/api/skills/sync", {
        method: "POST",
        ...jsonBody({}),
      }),
    skillStatus: () => request<SyncReportDto>(baseUrl, "/api/skills/status"),
    listNotes: () => request<KnowledgeNoteDto[]>(baseUrl, "/api/knowledge/entries"),
    saveNote: (input: KnowledgeEntryUpsertDto) =>
      request<KnowledgeNoteDto>(baseUrl, "/api/knowledge/entries", {
        method: "POST",
        ...jsonBody(input),
      }),
    deleteNote: (input: KnowledgeEntryDeleteDto) =>
      request<{ ok: boolean }>(baseUrl, "/api/knowledge/entries/delete", {
        method: "POST",
        ...jsonBody(input),
      }),
    runChat: (input: ChatRequestDto) =>
      request<ChatResponseDto>(baseUrl, "/api/openai-chat/chat", {
        method: "POST",
        ...jsonBody(input),
      }),
    getBrandingSettings: () =>
      request<BrandingSettingsDto>(baseUrl, "/api/admin/settings/branding"),
    saveBrandingSettings: (input: BrandingSettingsUpdate) =>
      request<BrandingSettingsDto>(baseUrl, "/api/admin/settings/branding", {
        method: "POST",
        ...jsonBody(input),
      }),
    uploadLogo: (input: { file_name: string; content_type?: string | null; bytes: number[] }) =>
      request<StoredLogoDto>(baseUrl, "/api/admin/storage/logo", {
        method: "POST",
        ...jsonBody(input),
      }),
    browseFiles: (input: StorageBrowseRequestDto) =>
      request<StorageBrowseResultDto>(baseUrl, "/api/admin/storage/files/browse", {
        method: "POST",
        ...jsonBody(input),
      }),
    uploadFiles: (input: StorageUploadRequestDto) =>
      request<StorageUploadResultDto>(baseUrl, "/api/admin/storage/files/upload", {
        method: "POST",
        ...jsonBody(input),
      }),
    createFolder: (input: StorageCreateFolderDto) =>
      request<StorageCreateFolderResultDto>(baseUrl, "/api/admin/storage/files/folders", {
        method: "POST",
        ...jsonBody(input),
      }),
    shareFile: (input: StorageShareRequestDto) =>
      request<StorageShareResultDto>(baseUrl, "/api/admin/storage/files/share", {
        method: "POST",
        ...jsonBody(input),
      }),
    deleteFile: (input: StorageDeleteObjectDto) =>
      request<StorageDeleteResultDto>(baseUrl, "/api/admin/storage/files/delete", {
        method: "POST",
        ...jsonBody(input),
      }),
    deleteFolder: (input: StorageDeleteFolderDto) =>
      request<StorageDeleteResultDto>(baseUrl, "/api/admin/storage/files/folders/delete", {
        method: "POST",
        ...jsonBody(input),
      }),
    listMenus: () => request<MenuDto[]>(baseUrl, "/api/admin/system/menus"),
    createMenu: (input: MenuUpsertDto) =>
      request<MenuDto>(baseUrl, "/api/admin/system/menus", {
        method: "POST",
        ...jsonBody(input),
      }),
    updateMenu: (id: number, input: MenuUpsertDto) =>
      request<MenuDto>(baseUrl, `/api/admin/system/menus/${id}`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    deleteMenu: (id: number) =>
      request<void>(baseUrl, `/api/admin/system/menus/${id}`, { method: "DELETE" }),
    listRoles: () => request<RoleDto[]>(baseUrl, "/api/admin/system/roles"),
    getRole: (id: number) =>
      request<RoleWithMenusDto>(baseUrl, `/api/admin/system/roles/${id}`),
    createRole: (input: RoleUpsertDto) =>
      request<RoleDto>(baseUrl, "/api/admin/system/roles", {
        method: "POST",
        ...jsonBody(input),
      }),
    updateRole: (id: number, input: RoleUpsertDto) =>
      request<RoleDto>(baseUrl, `/api/admin/system/roles/${id}`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    deleteRole: (id: number) =>
      request<void>(baseUrl, `/api/admin/system/roles/${id}`, { method: "DELETE" }),
    authorizeRoleMenus: (id: number, input: AuthorizeRoleMenusDto) =>
      request<void>(baseUrl, `/api/admin/system/roles/${id}/menus`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    listUsers: () => request<UserDto[]>(baseUrl, "/api/admin/system/users"),
    getUser: (id: number) =>
      request<UserWithRolesDto>(baseUrl, `/api/admin/system/users/${id}`),
    createUser: (input: UserUpsertDto) =>
      request<UserDto>(baseUrl, "/api/admin/system/users", {
        method: "POST",
        ...jsonBody(input),
      }),
    updateUser: (id: number, input: UserUpsertDto) =>
      request<UserDto>(baseUrl, `/api/admin/system/users/${id}`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    deleteUser: (id: number) =>
      request<void>(baseUrl, `/api/admin/system/users/${id}`, { method: "DELETE" }),
    authorizeUserRoles: (id: number, input: AuthorizeUserRolesDto) =>
      request<void>(baseUrl, `/api/admin/system/users/${id}/roles`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    getUserMenus: (id: number) =>
      request<MenuDto[]>(baseUrl, `/api/admin/system/users/${id}/menus`),
    listDepartments: () =>
      request<DepartmentDto[]>(baseUrl, "/api/admin/system/departments"),
    createDepartment: (input: DepartmentUpsertDto) =>
      request<DepartmentDto>(baseUrl, "/api/admin/system/departments", {
        method: "POST",
        ...jsonBody(input),
      }),
    updateDepartment: (id: number, input: DepartmentUpsertDto) =>
      request<DepartmentDto>(baseUrl, `/api/admin/system/departments/${id}`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    deleteDepartment: (id: number) =>
      request<void>(baseUrl, `/api/admin/system/departments/${id}`, {
        method: "DELETE",
      }),
    listDictGroups: () =>
      request<DictGroupDto[]>(baseUrl, "/api/admin/system/dict-groups"),
    createDictGroup: (input: DictGroupUpsertDto) =>
      request<DictGroupDto>(baseUrl, "/api/admin/system/dict-groups", {
        method: "POST",
        ...jsonBody(input),
      }),
    updateDictGroup: (id: number, input: DictGroupUpsertDto) =>
      request<DictGroupDto>(baseUrl, `/api/admin/system/dict-groups/${id}`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    deleteDictGroup: (id: number) =>
      request<void>(baseUrl, `/api/admin/system/dict-groups/${id}`, {
        method: "DELETE",
      }),
    listDictItems: (groupId: number) =>
      request<DictItemDto[]>(
        baseUrl,
        `/api/admin/system/dict-items?group_id=${groupId}`,
      ),
    createDictItem: (input: DictItemUpsertDto) =>
      request<DictItemDto>(baseUrl, "/api/admin/system/dict-items", {
        method: "POST",
        ...jsonBody(input),
      }),
    updateDictItem: (id: number, input: DictItemUpsertDto) =>
      request<DictItemDto>(baseUrl, `/api/admin/system/dict-items/${id}`, {
        method: "PUT",
        ...jsonBody(input),
      }),
    deleteDictItem: (id: number) =>
      request<void>(baseUrl, `/api/admin/system/dict-items/${id}`, {
        method: "DELETE",
      }),
    cliCatalog: () => request<CliMarketCatalog>(baseUrl, "/api/cli-market"),
    cliEntry: (id: string) =>
      request<CliMarketEntry | null>(baseUrl, `/api/cli-market/${encodeURIComponent(id)}`),
    cliUpsert: (input: CliMarketEntryUpsert) =>
      request<CliMarketEntry>(baseUrl, "/api/cli-market/upsert", {
        method: "POST",
        ...jsonBody(input),
      }),
    cliImport: (input: CliMarketImportRequest) =>
      request<CliMarketImportJobDetail>(baseUrl, "/api/cli-market/import", {
        method: "POST",
        ...jsonBody(input),
      }),
    cliJobs: () => request<CliMarketImportJob[]>(baseUrl, "/api/cli-market/import-jobs"),
    cliJobDetail: (id: string) =>
      request<CliMarketImportJobDetail | null>(
        baseUrl,
        `/api/cli-market/import-jobs/${encodeURIComponent(id)}`,
      ),
    cliExportJson: (input: CliMarketExportRequest) =>
      request<{ bytes_b64: string; file_name: string }>(baseUrl, "/api/cli-market/export/json", {
        method: "POST",
        ...jsonBody(input),
      }),
    cliExportXlsx: (input: CliMarketExportRequest) =>
      request<{ bytes_b64: string; file_name: string }>(baseUrl, "/api/cli-market/export/xlsx", {
        method: "POST",
        ...jsonBody(input),
      }),
    cliInstallHistory: (id: string) =>
      request<CliMarketInstallHistoryItem[]>(
        baseUrl,
        `/api/cli-market/${encodeURIComponent(id)}/install-history`,
      ),
    cliInstall: (id: string, input: CliMarketInstallRequest) =>
      request<CliMarketInstallResult>(
        baseUrl,
        `/api/cli-market/${encodeURIComponent(id)}/install`,
        {
          method: "POST",
          ...jsonBody(input),
        },
      ),
    cliPublish: (id: string) =>
      request<CliMarketEntry>(baseUrl, `/api/cli-market/${encodeURIComponent(id)}/publish`, {
        method: "POST",
        ...jsonBody({}),
      }),
    cliArchive: (id: string) =>
      request<CliMarketEntry>(baseUrl, `/api/cli-market/${encodeURIComponent(id)}/archive`, {
        method: "POST",
        ...jsonBody({}),
      }),
  };
}
