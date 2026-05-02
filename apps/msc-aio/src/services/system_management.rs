use std::{future::Future, pin::Pin, rc::Rc};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SystemManagementError {
    #[error("{0}")]
    Message(String),
}

impl SystemManagementError {
    fn msg(m: impl Into<String>) -> Self {
        Self::Message(m.into())
    }
}

pub type SystemManagementResult<T> = Result<T, SystemManagementError>;

// ─── Shared PgPool (native only) ───────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
use sqlx::postgres::PgPool;

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
static PG_POOL: Lazy<Option<PgPool>> = Lazy::new(|| {
    let url = std::env::var("DATABASE_URL").ok()?;
    let rt = tokio::runtime::Handle::try_current().ok()?;
    let pool = rt.block_on(async { PgPool::connect(&url).await.ok() });
    pool
});

#[cfg(not(target_arch = "wasm32"))]
fn pg_pool() -> SystemManagementResult<PgPool> {
    PG_POOL
        .as_ref()
        .cloned()
        .ok_or_else(|| SystemManagementError::msg("DATABASE_URL not set or pool init failed"))
}

// ─── Bcrypt helpers (native only) ──────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn hash_password(plain: &str) -> SystemManagementResult<String> {
    bcrypt::hash(plain, 10)
        .map_err(|e| SystemManagementError::msg(format!("bcrypt hash: {e}")))
}

// ─── DTOs ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuDto {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub route: String,
    pub icon: String,
    pub sort_order: i32,
    pub visible: bool,
    pub permission_code: String,
    pub menu_type: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuUpsertDto {
    pub parent_id: Option<i32>,
    pub name: String,
    pub route: String,
    pub icon: String,
    pub sort_order: i32,
    pub visible: bool,
    pub permission_code: String,
    pub menu_type: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDto {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub is_system: bool,
    pub menu_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleUpsertDto {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleWithMenusDto {
    pub role: RoleDto,
    pub menu_ids: Vec<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub nickname: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUpsertDto {
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserWithRolesDto {
    pub user: UserDto,
    pub role_ids: Vec<i32>,
    pub role_names: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizeRoleMenusDto {
    pub menu_ids: Vec<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizeUserRolesDto {
    pub role_ids: Vec<i32>,
}

// ─── Department DTOs ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepartmentDto {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepartmentUpsertDto {
    pub parent_id: Option<i32>,
    pub name: String,
    pub sort_order: i32,
}

// ─── Dictionary DTOs ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictGroupDto {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub item_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictGroupUpsertDto {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictItemDto {
    pub id: i32,
    pub group_id: i32,
    pub label: String,
    pub value: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictItemUpsertDto {
    pub group_id: i32,
    pub label: String,
    pub value: String,
    pub sort_order: i32,
}

// ─── Trait ───────────────────────────────────────────────────────────────────

pub trait SystemManagementApi: 'static {
    fn list_menus(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<MenuDto>>>;
    fn create_menu(&self, input: MenuUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<MenuDto>>;
    fn update_menu(&self, id: i32, input: MenuUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<MenuDto>>;
    fn delete_menu(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>>;
    fn list_roles(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<RoleDto>>>;
    fn get_role(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<RoleWithMenusDto>>;
    fn create_role(&self, input: RoleUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<RoleDto>>;
    fn update_role(&self, id: i32, input: RoleUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<RoleDto>>;
    fn delete_role(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>>;
    fn authorize_role_menus(&self, role_id: i32, menu_ids: Vec<i32>) -> LocalBoxFuture<'_, SystemManagementResult<()>>;
    fn list_users(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<UserWithRolesDto>>>;
    fn get_user(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<UserWithRolesDto>>;
    fn create_user(&self, input: UserUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<UserDto>>;
    fn update_user(&self, id: i32, input: UserUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<UserDto>>;
    fn delete_user(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>>;
    fn authorize_user_roles(&self, user_id: i32, role_ids: Vec<i32>) -> LocalBoxFuture<'_, SystemManagementResult<()>>;
    fn get_user_effective_menu_ids(&self, user_id: i32) -> LocalBoxFuture<'_, SystemManagementResult<Vec<i32>>>;

    // Departments
    fn list_departments(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DepartmentDto>>>;
    fn create_department(&self, input: DepartmentUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DepartmentDto>>;
    fn update_department(&self, id: i32, input: DepartmentUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DepartmentDto>>;
    fn delete_department(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>>;

    // Dictionary groups
    fn list_dict_groups(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DictGroupDto>>>;
    fn create_dict_group(&self, input: DictGroupUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictGroupDto>>;
    fn update_dict_group(&self, id: i32, input: DictGroupUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictGroupDto>>;
    fn delete_dict_group(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>>;

    // Dictionary items
    fn list_dict_items(&self, group_id: i32) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DictItemDto>>>;
    fn create_dict_item(&self, input: DictItemUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictItemDto>>;
    fn update_dict_item(&self, id: i32, input: DictItemUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictItemDto>>;
    fn delete_dict_item(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>>;
}

pub type SharedSystemManagementApi = Rc<dyn SystemManagementApi>;

pub fn default_system_management_api() -> SharedSystemManagementApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserSystemManagementApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedSystemManagementApi)
    }
}

// ─── Browser (wasm32) ───────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
struct BrowserSystemManagementApi;

#[cfg(target_arch = "wasm32")]
impl SystemManagementApi for BrowserSystemManagementApi {
    // Menus
    fn list_menus(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<MenuDto>>> {
        Box::pin(async { super::browser_http::get_json("/api/admin/system/menus").await.map_err(SystemManagementError::msg) })
    }
    fn create_menu(&self, input: MenuUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<MenuDto>> {
        Box::pin(async { super::browser_http::post_json("/api/admin/system/menus", &input).await.map_err(SystemManagementError::msg) })
    }
    fn update_menu(&self, id: i32, input: MenuUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<MenuDto>> {
        Box::pin(async move {
            super::browser_http::put_json(&format!("/api/admin/system/menus/{id}"), &input)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn delete_menu(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { super::browser_http::delete_empty(&format!("/api/admin/system/menus/{id}")).await.map_err(SystemManagementError::msg) })
    }

    // Roles
    fn list_roles(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<RoleDto>>> {
        Box::pin(async { super::browser_http::get_json("/api/admin/system/roles").await.map_err(SystemManagementError::msg) })
    }
    fn get_role(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<RoleWithMenusDto>> {
        Box::pin(async move { super::browser_http::get_json(&format!("/api/admin/system/roles/{id}")).await.map_err(SystemManagementError::msg) })
    }
    fn create_role(&self, input: RoleUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<RoleDto>> {
        Box::pin(async { super::browser_http::post_json("/api/admin/system/roles", &input).await.map_err(SystemManagementError::msg) })
    }
    fn update_role(&self, id: i32, input: RoleUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<RoleDto>> {
        Box::pin(async move {
            super::browser_http::put_json(&format!("/api/admin/system/roles/{id}"), &input)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn delete_role(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { super::browser_http::delete_empty(&format!("/api/admin/system/roles/{id}")).await.map_err(SystemManagementError::msg) })
    }
    fn authorize_role_menus(&self, role_id: i32, menu_ids: Vec<i32>) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move {
            let body = AuthorizeRoleMenusDto { menu_ids };
            super::browser_http::put_empty(&format!("/api/admin/system/roles/{role_id}/menus"), &body)
                .await
                .map_err(SystemManagementError::msg)
        })
    }

    // Users
    fn list_users(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<UserWithRolesDto>>> {
        Box::pin(async { super::browser_http::get_json("/api/admin/system/users").await.map_err(SystemManagementError::msg) })
    }
    fn get_user(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<UserWithRolesDto>> {
        Box::pin(async move { super::browser_http::get_json(&format!("/api/admin/system/users/{id}")).await.map_err(SystemManagementError::msg) })
    }
    fn create_user(&self, input: UserUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<UserDto>> {
        Box::pin(async { super::browser_http::post_json("/api/admin/system/users", &input).await.map_err(SystemManagementError::msg) })
    }
    fn update_user(&self, id: i32, input: UserUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<UserDto>> {
        Box::pin(async move {
            super::browser_http::put_json(&format!("/api/admin/system/users/{id}"), &input)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn delete_user(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { super::browser_http::delete_empty(&format!("/api/admin/system/users/{id}")).await.map_err(SystemManagementError::msg) })
    }
    fn authorize_user_roles(&self, user_id: i32, role_ids: Vec<i32>) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move {
            let body = AuthorizeUserRolesDto { role_ids };
            super::browser_http::put_empty(&format!("/api/admin/system/users/{user_id}/roles"), &body)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn get_user_effective_menu_ids(&self, user_id: i32) -> LocalBoxFuture<'_, SystemManagementResult<Vec<i32>>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/admin/system/users/{user_id}/menus"))
                .await
                .map_err(SystemManagementError::msg)
        })
    }

    // Departments
    fn list_departments(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DepartmentDto>>> {
        Box::pin(async { super::browser_http::get_json("/api/admin/system/departments").await.map_err(SystemManagementError::msg) })
    }
    fn create_department(&self, input: DepartmentUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DepartmentDto>> {
        Box::pin(async { super::browser_http::post_json("/api/admin/system/departments", &input).await.map_err(SystemManagementError::msg) })
    }
    fn update_department(&self, id: i32, input: DepartmentUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DepartmentDto>> {
        Box::pin(async move {
            super::browser_http::put_json(&format!("/api/admin/system/departments/{id}"), &input)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn delete_department(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { super::browser_http::delete_empty(&format!("/api/admin/system/departments/{id}")).await.map_err(SystemManagementError::msg) })
    }

    // Dictionary groups
    fn list_dict_groups(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DictGroupDto>>> {
        Box::pin(async { super::browser_http::get_json("/api/admin/system/dict-groups").await.map_err(SystemManagementError::msg) })
    }
    fn create_dict_group(&self, input: DictGroupUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictGroupDto>> {
        Box::pin(async { super::browser_http::post_json("/api/admin/system/dict-groups", &input).await.map_err(SystemManagementError::msg) })
    }
    fn update_dict_group(&self, id: i32, input: DictGroupUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictGroupDto>> {
        Box::pin(async move {
            super::browser_http::put_json(&format!("/api/admin/system/dict-groups/{id}"), &input)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn delete_dict_group(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { super::browser_http::delete_empty(&format!("/api/admin/system/dict-groups/{id}")).await.map_err(SystemManagementError::msg) })
    }

    // Dictionary items
    fn list_dict_items(&self, group_id: i32) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DictItemDto>>> {
        Box::pin(async move { super::browser_http::get_json(&format!("/api/admin/system/dict-items?group_id={group_id}")).await.map_err(SystemManagementError::msg) })
    }
    fn create_dict_item(&self, input: DictItemUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictItemDto>> {
        Box::pin(async { super::browser_http::post_json("/api/admin/system/dict-items", &input).await.map_err(SystemManagementError::msg) })
    }
    fn update_dict_item(&self, id: i32, input: DictItemUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictItemDto>> {
        Box::pin(async move {
            super::browser_http::put_json(&format!("/api/admin/system/dict-items/{id}"), &input)
                .await
                .map_err(SystemManagementError::msg)
        })
    }
    fn delete_dict_item(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { super::browser_http::delete_empty(&format!("/api/admin/system/dict-items/{id}")).await.map_err(SystemManagementError::msg) })
    }
}

// ─── Embedded (native) ──────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedSystemManagementApi;

#[cfg(not(target_arch = "wasm32"))]
impl SystemManagementApi for EmbeddedSystemManagementApi {
    fn list_menus(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<MenuDto>>> {
        Box::pin(async { list_menus_on_server().await })
    }
    fn create_menu(&self, input: MenuUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<MenuDto>> {
        Box::pin(async { create_menu_on_server(input).await })
    }
    fn update_menu(&self, id: i32, input: MenuUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<MenuDto>> {
        Box::pin(async move { update_menu_on_server(id, input).await })
    }
    fn delete_menu(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { delete_menu_on_server(id).await })
    }
    fn list_roles(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<RoleDto>>> {
        Box::pin(async { list_roles_on_server().await })
    }
    fn get_role(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<RoleWithMenusDto>> {
        Box::pin(async move { get_role_on_server(id).await })
    }
    fn create_role(&self, input: RoleUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<RoleDto>> {
        Box::pin(async { create_role_on_server(input).await })
    }
    fn update_role(&self, id: i32, input: RoleUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<RoleDto>> {
        Box::pin(async move { update_role_on_server(id, input).await })
    }
    fn delete_role(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { delete_role_on_server(id).await })
    }
    fn authorize_role_menus(&self, role_id: i32, menu_ids: Vec<i32>) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { authorize_role_menus_on_server(role_id, menu_ids).await })
    }
    fn list_users(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<UserWithRolesDto>>> {
        Box::pin(async { list_users_on_server().await })
    }
    fn get_user(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<UserWithRolesDto>> {
        Box::pin(async move { get_user_on_server(id).await })
    }
    fn create_user(&self, input: UserUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<UserDto>> {
        Box::pin(async { create_user_on_server(input).await })
    }
    fn update_user(&self, id: i32, input: UserUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<UserDto>> {
        Box::pin(async move { update_user_on_server(id, input).await })
    }
    fn delete_user(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { delete_user_on_server(id).await })
    }
    fn authorize_user_roles(&self, user_id: i32, role_ids: Vec<i32>) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { authorize_user_roles_on_server(user_id, role_ids).await })
    }
    fn get_user_effective_menu_ids(&self, user_id: i32) -> LocalBoxFuture<'_, SystemManagementResult<Vec<i32>>> {
        Box::pin(async move { get_user_effective_menu_ids_on_server(user_id).await })
    }
    fn list_departments(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DepartmentDto>>> {
        Box::pin(async { list_departments_on_server().await })
    }
    fn create_department(&self, input: DepartmentUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DepartmentDto>> {
        Box::pin(async { create_department_on_server(input).await })
    }
    fn update_department(&self, id: i32, input: DepartmentUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DepartmentDto>> {
        Box::pin(async move { update_department_on_server(id, input).await })
    }
    fn delete_department(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { delete_department_on_server(id).await })
    }
    fn list_dict_groups(&self) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DictGroupDto>>> {
        Box::pin(async { list_dict_groups_on_server().await })
    }
    fn create_dict_group(&self, input: DictGroupUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictGroupDto>> {
        Box::pin(async { create_dict_group_on_server(input).await })
    }
    fn update_dict_group(&self, id: i32, input: DictGroupUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictGroupDto>> {
        Box::pin(async move { update_dict_group_on_server(id, input).await })
    }
    fn delete_dict_group(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { delete_dict_group_on_server(id).await })
    }
    fn list_dict_items(&self, group_id: i32) -> LocalBoxFuture<'_, SystemManagementResult<Vec<DictItemDto>>> {
        Box::pin(async move { list_dict_items_on_server(group_id).await })
    }
    fn create_dict_item(&self, input: DictItemUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictItemDto>> {
        Box::pin(async { create_dict_item_on_server(input).await })
    }
    fn update_dict_item(&self, id: i32, input: DictItemUpsertDto) -> LocalBoxFuture<'_, SystemManagementResult<DictItemDto>> {
        Box::pin(async move { update_dict_item_on_server(id, input).await })
    }
    fn delete_dict_item(&self, id: i32) -> LocalBoxFuture<'_, SystemManagementResult<()>> {
        Box::pin(async move { delete_dict_item_on_server(id).await })
    }
}

// ─── Server functions: Menus ────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_menus_on_server() -> SystemManagementResult<Vec<MenuDto>> {
    let pool = pg_pool()?;
    let rows = sqlx::query_as::<_, (i32, Option<i32>, String, String, String, i32, bool, String, String)>(
        "SELECT id, parent_id, name, route, icon, sort_order, visible, permission_code, menu_type FROM sys_menu ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("list_menus: {e}")))?;
    Ok(rows
        .into_iter()
        .map(|(id, parent_id, name, route, icon, sort_order, visible, permission_code, menu_type)| MenuDto {
            id, parent_id, name, route, icon, sort_order, visible, permission_code, menu_type,
        })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_menu_on_server(input: MenuUpsertDto) -> SystemManagementResult<MenuDto> {
    let pool = pg_pool()?;
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO sys_menu (parent_id, name, route, icon, sort_order, visible, permission_code, menu_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(input.parent_id)
    .bind(&input.name)
    .bind(&input.route)
    .bind(&input.icon)
    .bind(input.sort_order)
    .bind(input.visible)
    .bind(&input.permission_code)
    .bind(&input.menu_type)
    .fetch_one(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("create_menu: {e}")))?;
    Ok(MenuDto { id: row.0, parent_id: input.parent_id, name: input.name, route: input.route, icon: input.icon, sort_order: input.sort_order, visible: input.visible, permission_code: input.permission_code, menu_type: input.menu_type })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_menu_on_server(id: i32, input: MenuUpsertDto) -> SystemManagementResult<MenuDto> {
    let pool = pg_pool()?;
    sqlx::query(
        "UPDATE sys_menu SET parent_id = $1, name = $2, route = $3, icon = $4, sort_order = $5, visible = $6, permission_code = $7, menu_type = $8, updated_at = NOW() WHERE id = $9",
    )
    .bind(input.parent_id)
    .bind(&input.name)
    .bind(&input.route)
    .bind(&input.icon)
    .bind(input.sort_order)
    .bind(input.visible)
    .bind(&input.permission_code)
    .bind(&input.menu_type)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("update_menu: {e}")))?;
    Ok(MenuDto { id, parent_id: input.parent_id, name: input.name, route: input.route, icon: input.icon, sort_order: input.sort_order, visible: input.visible, permission_code: input.permission_code, menu_type: input.menu_type })
}

/// 收集菜单及其所有后代 ID（服务端版本）
fn collect_menu_descendant_ids<'a>(pool: &'a sqlx::PgPool, menu_id: i32) -> std::pin::Pin<Box<dyn std::future::Future<Output = SystemManagementResult<Vec<i32>>> + Send + 'a>> {
    Box::pin(async move {
        let children: Vec<(i32,)> = sqlx::query_as("SELECT id FROM sys_menu WHERE parent_id = $1")
            .bind(menu_id)
            .fetch_all(pool)
            .await
            .map_err(|e| SystemManagementError::msg(format!("collect descendants: {e}")))?;
        let mut ids = vec![menu_id];
        for (child_id,) in children {
            ids.extend(collect_menu_descendant_ids(pool, child_id).await?);
        }
        Ok(ids)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_menu_on_server(id: i32) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    let all_ids = collect_menu_descendant_ids(&pool, id).await?;
    let mut tx = pool.begin().await.map_err(|e| SystemManagementError::msg(format!("tx begin: {e}")))?;
    // 先删关联的角色-菜单关系
    for mid in &all_ids {
        sqlx::query("DELETE FROM sys_role_menu WHERE menu_id = $1")
            .bind(mid)
            .execute(&mut *tx)
            .await
            .map_err(|e| SystemManagementError::msg(format!("delete role_menu: {e}")))?;
    }
    // 再删菜单（从叶子到根）
    for mid in all_ids.iter().rev() {
        sqlx::query("DELETE FROM sys_menu WHERE id = $1")
            .bind(mid)
            .execute(&mut *tx)
            .await
            .map_err(|e| SystemManagementError::msg(format!("delete_menu: {e}")))?;
    }
    tx.commit().await.map_err(|e| SystemManagementError::msg(format!("tx commit: {e}")))?;
    Ok(())
}

// ─── Server functions: Roles ────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_roles_on_server() -> SystemManagementResult<Vec<RoleDto>> {
    let pool = pg_pool()?;
    let rows = sqlx::query_as::<_, (i32, String, String, bool, i64)>(
        "SELECT r.id, r.name, r.description, r.is_system, COUNT(rm.menu_id) AS menu_count \
         FROM sys_role r LEFT JOIN sys_role_menu rm ON rm.role_id = r.id \
         GROUP BY r.id ORDER BY r.id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("list_roles: {e}")))?;
    Ok(rows
        .into_iter()
        .map(|(id, name, description, is_system, menu_count)| RoleDto { id, name, description, is_system, menu_count })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_role_on_server(id: i32) -> SystemManagementResult<RoleWithMenusDto> {
    let pool = pg_pool()?;
    let role_row = sqlx::query_as::<_, (i32, String, String, bool)>(
        "SELECT id, name, description, is_system FROM sys_role WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("get_role: {e}")))?
    .ok_or_else(|| SystemManagementError::msg("role not found"))?;

    let menu_count: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM sys_role_menu WHERE role_id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("get_role menu_count: {e}")))?;

    let menu_ids: Vec<(i32,)> = sqlx::query_as("SELECT menu_id FROM sys_role_menu WHERE role_id = $1")
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("get_role menus: {e}")))?;

    Ok(RoleWithMenusDto {
        role: RoleDto { id: role_row.0, name: role_row.1, description: role_row.2, is_system: role_row.3, menu_count: menu_count.0 },
        menu_ids: menu_ids.into_iter().map(|r| r.0).collect(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_role_on_server(input: RoleUpsertDto) -> SystemManagementResult<RoleDto> {
    let pool = pg_pool()?;
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO sys_role (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(&input.name)
    .bind(&input.description)
    .fetch_one(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("create_role: {e}")))?;
    Ok(RoleDto { id: row.0, name: input.name, description: input.description, is_system: false, menu_count: 0 })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_role_on_server(id: i32, input: RoleUpsertDto) -> SystemManagementResult<RoleDto> {
    let pool = pg_pool()?;
    sqlx::query("UPDATE sys_role SET name = $1, description = $2, updated_at = NOW() WHERE id = $3")
        .bind(&input.name)
        .bind(&input.description)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("update_role: {e}")))?;
    Ok(RoleDto { id, name: input.name, description: input.description, is_system: false, menu_count: 0 })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_role_on_server(id: i32) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    sqlx::query("DELETE FROM sys_role WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("delete_role: {e}")))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn authorize_role_menus_on_server(role_id: i32, menu_ids: Vec<i32>) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    let mut tx = pool.begin().await.map_err(|e| SystemManagementError::msg(format!("tx begin: {e}")))?;
    sqlx::query("DELETE FROM sys_role_menu WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| SystemManagementError::msg(format!("authorize_role_menus delete: {e}")))?;
    for mid in &menu_ids {
        sqlx::query("INSERT INTO sys_role_menu (role_id, menu_id) VALUES ($1, $2)")
            .bind(role_id)
            .bind(mid)
            .execute(&mut *tx)
            .await
            .map_err(|e| SystemManagementError::msg(format!("authorize_role_menus insert: {e}")))?;
    }
    tx.commit().await.map_err(|e| SystemManagementError::msg(format!("tx commit: {e}")))?;
    Ok(())
}

// ─── Server functions: Users ────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_users_on_server() -> SystemManagementResult<Vec<UserWithRolesDto>> {
    let pool = pg_pool()?;
    let users = sqlx::query_as::<_, (i32, String, String, String)>(
        "SELECT id, username, nickname, status FROM sys_user ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("list_users: {e}")))?;

    let mut result = Vec::with_capacity(users.len());
    for (id, username, nickname, status) in users {
        let roles = sqlx::query_as::<_, (i32, String)>(
            "SELECT r.id, r.name FROM sys_role r \
             JOIN sys_user_role ur ON ur.role_id = r.id \
             WHERE ur.user_id = $1 ORDER BY r.id",
        )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("list_users roles: {e}")))?;

        result.push(UserWithRolesDto {
            user: UserDto { id, username, nickname, status },
            role_ids: roles.iter().map(|r| r.0).collect(),
            role_names: roles.into_iter().map(|r| r.1).collect(),
        });
    }
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_user_on_server(id: i32) -> SystemManagementResult<UserWithRolesDto> {
    let pool = pg_pool()?;
    let user_row = sqlx::query_as::<_, (i32, String, String, String)>(
        "SELECT id, username, nickname, status FROM sys_user WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("get_user: {e}")))?
    .ok_or_else(|| SystemManagementError::msg("user not found"))?;

    let roles = sqlx::query_as::<_, (i32, String)>(
        "SELECT r.id, r.name FROM sys_role r \
         JOIN sys_user_role ur ON ur.role_id = r.id \
         WHERE ur.user_id = $1 ORDER BY r.id",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("get_user roles: {e}")))?;

    Ok(UserWithRolesDto {
        user: UserDto { id: user_row.0, username: user_row.1, nickname: user_row.2, status: user_row.3 },
        role_ids: roles.iter().map(|r| r.0).collect(),
        role_names: roles.into_iter().map(|r| r.1).collect(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_user_on_server(input: UserUpsertDto) -> SystemManagementResult<UserDto> {
    let pool = pg_pool()?;
    let hashed = hash_password(&input.password)?;
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO sys_user (username, password_hash, nickname, status) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&input.username)
    .bind(&hashed)
    .bind(&input.nickname)
    .bind(&input.status)
    .fetch_one(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("create_user: {e}")))?;
    Ok(UserDto { id: row.0, username: input.username, nickname: input.nickname, status: input.status })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_user_on_server(id: i32, input: UserUpsertDto) -> SystemManagementResult<UserDto> {
    let pool = pg_pool()?;
    if input.password.is_empty() {
        // Only update non-password fields
        sqlx::query(
            "UPDATE sys_user SET username = $1, nickname = $2, status = $3, updated_at = NOW() WHERE id = $4",
        )
        .bind(&input.username)
        .bind(&input.nickname)
        .bind(&input.status)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("update_user: {e}")))?;
    } else {
        let hashed = hash_password(&input.password)?;
        sqlx::query(
            "UPDATE sys_user SET username = $1, password_hash = $2, nickname = $3, status = $4, updated_at = NOW() WHERE id = $5",
        )
        .bind(&input.username)
        .bind(&hashed)
        .bind(&input.nickname)
        .bind(&input.status)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("update_user: {e}")))?;
    }
    Ok(UserDto { id, username: input.username, nickname: input.nickname, status: input.status })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_user_on_server(id: i32) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    sqlx::query("DELETE FROM sys_user WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("delete_user: {e}")))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn authorize_user_roles_on_server(user_id: i32, role_ids: Vec<i32>) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    let mut tx = pool.begin().await.map_err(|e| SystemManagementError::msg(format!("tx begin: {e}")))?;
    sqlx::query("DELETE FROM sys_user_role WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| SystemManagementError::msg(format!("authorize_user_roles delete: {e}")))?;
    for rid in &role_ids {
        sqlx::query("INSERT INTO sys_user_role (user_id, role_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(rid)
            .execute(&mut *tx)
            .await
            .map_err(|e| SystemManagementError::msg(format!("authorize_user_roles insert: {e}")))?;
    }
    tx.commit().await.map_err(|e| SystemManagementError::msg(format!("tx commit: {e}")))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_user_effective_menu_ids_on_server(user_id: i32) -> SystemManagementResult<Vec<i32>> {
    let pool = pg_pool()?;
    let rows = sqlx::query_as::<_, (i32,)>(
        "SELECT DISTINCT rm.menu_id FROM sys_user_role ur          JOIN sys_role_menu rm ON rm.role_id = ur.role_id          WHERE ur.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("get_user_effective_menu_ids: {e}")))?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// 根据用户名查询其所有角色关联的菜单 permission_code 集合。
/// admin 用户自动拥有全部权限（返回 None 表示不做过滤）。
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_effective_permission_codes_on_server(username: &str) -> SystemManagementResult<Option<Vec<String>>> {
    let pool = pg_pool()?;
    // admin 超级管理员不限制
    if username == "admin" {
        return Ok(None);
    }
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT m.permission_code FROM sys_user u          JOIN sys_user_role ur ON ur.user_id = u.id          JOIN sys_role_menu rm ON rm.role_id = ur.role_id          JOIN sys_menu m ON m.id = rm.menu_id          WHERE u.username = $1 AND m.permission_code != ''",
    )
    .bind(username)
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("get_effective_permission_codes: {e}")))?;
    Ok(Some(rows.into_iter().map(|r| r.0).collect()))
}

// ─── Server functions: Departments ──────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_departments_on_server() -> SystemManagementResult<Vec<DepartmentDto>> {
    let pool = pg_pool()?;
    let rows = sqlx::query_as::<_, (i32, Option<i32>, String, i32)>(
        "SELECT id, parent_id, name, sort_order FROM sys_department ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("list_departments: {e}")))?;
    Ok(rows
        .into_iter()
        .map(|(id, parent_id, name, sort_order)| DepartmentDto { id, parent_id, name, sort_order })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_department_on_server(input: DepartmentUpsertDto) -> SystemManagementResult<DepartmentDto> {
    let pool = pg_pool()?;
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO sys_department (parent_id, name, sort_order) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(input.parent_id)
    .bind(&input.name)
    .bind(input.sort_order)
    .fetch_one(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("create_department: {e}")))?;
    Ok(DepartmentDto { id: row.0, parent_id: input.parent_id, name: input.name, sort_order: input.sort_order })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_department_on_server(id: i32, input: DepartmentUpsertDto) -> SystemManagementResult<DepartmentDto> {
    let pool = pg_pool()?;
    sqlx::query(
        "UPDATE sys_department SET parent_id = $1, name = $2, sort_order = $3, updated_at = NOW() WHERE id = $4",
    )
    .bind(input.parent_id)
    .bind(&input.name)
    .bind(input.sort_order)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("update_department: {e}")))?;
    Ok(DepartmentDto { id, parent_id: input.parent_id, name: input.name, sort_order: input.sort_order })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_department_on_server(id: i32) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    sqlx::query("DELETE FROM sys_department WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("delete_department: {e}")))?;
    Ok(())
}

// ─── Server functions: Dict Groups ──────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_dict_groups_on_server() -> SystemManagementResult<Vec<DictGroupDto>> {
    let pool = pg_pool()?;
    let rows = sqlx::query_as::<_, (i32, String, String, i64)>(
        "SELECT g.id, g.name, g.description, COUNT(di.id) AS item_count \
         FROM sys_dict_group g LEFT JOIN sys_dict_item di ON di.group_id = g.id \
         GROUP BY g.id ORDER BY g.id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("list_dict_groups: {e}")))?;
    Ok(rows
        .into_iter()
        .map(|(id, name, description, item_count)| DictGroupDto { id, name, description, item_count })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_dict_group_on_server(input: DictGroupUpsertDto) -> SystemManagementResult<DictGroupDto> {
    let pool = pg_pool()?;
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO sys_dict_group (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(&input.name)
    .bind(&input.description)
    .fetch_one(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("create_dict_group: {e}")))?;
    Ok(DictGroupDto { id: row.0, name: input.name, description: input.description, item_count: 0 })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_dict_group_on_server(id: i32, input: DictGroupUpsertDto) -> SystemManagementResult<DictGroupDto> {
    let pool = pg_pool()?;
    sqlx::query("UPDATE sys_dict_group SET name = $1, description = $2, updated_at = NOW() WHERE id = $3")
        .bind(&input.name)
        .bind(&input.description)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("update_dict_group: {e}")))?;
    Ok(DictGroupDto { id, name: input.name, description: input.description, item_count: 0 })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_dict_group_on_server(id: i32) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    sqlx::query("DELETE FROM sys_dict_group WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("delete_dict_group: {e}")))?;
    Ok(())
}

// ─── Server functions: Dict Items ───────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_dict_items_on_server(group_id: i32) -> SystemManagementResult<Vec<DictItemDto>> {
    let pool = pg_pool()?;
    let rows = sqlx::query_as::<_, (i32, i32, String, String, i32)>(
        "SELECT id, group_id, label, value, sort_order FROM sys_dict_item WHERE group_id = $1 ORDER BY sort_order, id",
    )
    .bind(group_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("list_dict_items: {e}")))?;
    Ok(rows
        .into_iter()
        .map(|(id, group_id, label, value, sort_order)| DictItemDto { id, group_id, label, value, sort_order })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_dict_item_on_server(input: DictItemUpsertDto) -> SystemManagementResult<DictItemDto> {
    let pool = pg_pool()?;
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO sys_dict_item (group_id, label, value, sort_order) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(input.group_id)
    .bind(&input.label)
    .bind(&input.value)
    .bind(input.sort_order)
    .fetch_one(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("create_dict_item: {e}")))?;
    Ok(DictItemDto { id: row.0, group_id: input.group_id, label: input.label, value: input.value, sort_order: input.sort_order })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_dict_item_on_server(id: i32, input: DictItemUpsertDto) -> SystemManagementResult<DictItemDto> {
    let pool = pg_pool()?;
    sqlx::query(
        "UPDATE sys_dict_item SET group_id = $1, label = $2, value = $3, sort_order = $4, updated_at = NOW() WHERE id = $5",
    )
    .bind(input.group_id)
    .bind(&input.label)
    .bind(&input.value)
    .bind(input.sort_order)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| SystemManagementError::msg(format!("update_dict_item: {e}")))?;
    Ok(DictItemDto { id, group_id: input.group_id, label: input.label, value: input.value, sort_order: input.sort_order })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_dict_item_on_server(id: i32) -> SystemManagementResult<()> {
    let pool = pg_pool()?;
    sqlx::query("DELETE FROM sys_dict_item WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| SystemManagementError::msg(format!("delete_dict_item: {e}")))?;
    Ok(())
}
