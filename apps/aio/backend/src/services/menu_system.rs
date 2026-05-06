use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Menu {
    pub id: Uuid,
    pub route_path: String,
    pub title: String,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub visible: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTreeNode {
    #[serde(flatten)]
    pub menu: Menu,
    pub children: Vec<MenuTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMenuRequest {
    pub route_path: String,
    pub title: String,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
    pub visible: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMenuRequest {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
    pub visible: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct MenuService {
    pool: PgPool,
}

impl MenuService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_menu_tree(&self) -> Result<Vec<MenuTreeNode>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, route_path, title, icon, parent_id, sort_order, visible, metadata, created_at, updated_at
            FROM admin_menus
            WHERE visible = true
            ORDER BY sort_order ASC, title ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let menus: Vec<Menu> = rows
            .into_iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let route_path: String = row.get("route_path");
                let title: String = row.get("title");
                let icon: Option<String> = row.get("icon");
                let parent_id: Option<Uuid> = row.get("parent_id");
                let sort_order: i32 = row.get("sort_order");
                let visible: bool = row.get("visible");
                let metadata: serde_json::Value = row.get("metadata");
                let created_at: DateTime<Utc> = row.get("created_at");
                let updated_at: DateTime<Utc> = row.get("updated_at");
                Menu {
                    id,
                    route_path,
                    title,
                    icon,
                    parent_id,
                    sort_order,
                    visible,
                    metadata,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(self.build_tree(&menus, None))
    }

    fn build_tree(&self, menus: &[Menu], parent_id: Option<Uuid>) -> Vec<MenuTreeNode> {
        menus
            .iter()
            .filter(|m| m.parent_id == parent_id)
            .map(|menu| MenuTreeNode {
                menu: menu.clone(),
                children: self.build_tree(menus, Some(menu.id)),
            })
            .collect()
    }

    pub async fn get_menu_by_id(&self, id: Uuid) -> Result<Option<Menu>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, route_path, title, icon, parent_id, sort_order, visible, metadata, created_at, updated_at
            FROM admin_menus
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let id: Uuid = r.get("id");
            let route_path: String = r.get("route_path");
            let title: String = r.get("title");
            let icon: Option<String> = r.get("icon");
            let parent_id: Option<Uuid> = r.get("parent_id");
            let sort_order: i32 = r.get("sort_order");
            let visible: bool = r.get("visible");
            let metadata: serde_json::Value = r.get("metadata");
            let created_at: DateTime<Utc> = r.get("created_at");
            let updated_at: DateTime<Utc> = r.get("updated_at");
            Menu {
                id,
                route_path,
                title,
                icon,
                parent_id,
                sort_order,
                visible,
                metadata,
                created_at,
                updated_at,
            }
        }))
    }

    pub async fn create_menu(&self, req: CreateMenuRequest) -> Result<Menu, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO admin_menus (id, route_path, title, icon, parent_id, sort_order, visible, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(id)
        .bind(&req.route_path)
        .bind(&req.title)
        .bind(&req.icon)
        .bind(&req.parent_id)
        .bind(req.sort_order.unwrap_or(0))
        .bind(req.visible.unwrap_or(true))
        .bind(&req.metadata.unwrap_or(serde_json::json!({})))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_menu_by_id(id).await.map(|m| m.unwrap())
    }

    pub async fn update_menu(
        &self,
        id: Uuid,
        req: UpdateMenuRequest,
    ) -> Result<Option<Menu>, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE admin_menus
            SET
                title = COALESCE($2, title),
                icon = COALESCE($3, icon),
                parent_id = COALESCE($4, parent_id),
                sort_order = COALESCE($5, sort_order),
                visible = COALESCE($6, visible),
                metadata = COALESCE($7, metadata),
                updated_at = $8
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.icon)
        .bind(&req.parent_id)
        .bind(&req.sort_order)
        .bind(&req.visible)
        .bind(&req.metadata)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_menu_by_id(id).await
    }

    pub async fn delete_menu(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM admin_menus WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_all_permissions(&self) -> Result<Vec<Permission>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, category, created_at
            FROM admin_permissions
            ORDER BY category, name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let category: Option<String> = row.get("category");
                let created_at: DateTime<Utc> = row.get("created_at");
                Permission {
                    id,
                    name,
                    description,
                    category,
                    created_at,
                }
            })
            .collect())
    }

    pub async fn sync_file_routes(&self, routes: Vec<String>) -> Result<u64, sqlx::Error> {
        let mut created_count = 0;
        let now = Utc::now();

        for route in routes {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM admin_menus WHERE route_path = $1)",
            )
            .bind(&route)
            .fetch_one(&self.pool)
            .await?;

            if !exists {
                let title = route.split('/').last().unwrap_or(&route).to_string();
                let title = title.replace('_', " ");

                sqlx::query(
                    r#"
                    INSERT INTO admin_menus (id, route_path, title, icon, parent_id, sort_order, visible, metadata, created_at, updated_at)
                    VALUES ($1, $2, $3, NULL, NULL, 0, true, '{}'::jsonb, $4, $5)
                    "#
                )
                .bind(Uuid::new_v4())
                .bind(&route)
                .bind(&title)
                .bind(now)
                .bind(now)
                .execute(&self.pool)
                .await?;

                created_count += 1;
            }
        }

        Ok(created_count)
    }
}
