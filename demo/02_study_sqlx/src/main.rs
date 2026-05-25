#![forbid(unsafe_code)]

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = database_url()?;
    run_sqlx_crud(&database_url).await?;
    Ok(())
}

fn database_url() -> Result<String> {
    std::env::var("SQLX_STUDY_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .context("set SQLX_STUDY_DATABASE_URL or DATABASE_URL before running the demo")
}

#[derive(Debug, PartialEq)]
struct TodoSnapshot {
    id: String,
    title: String,
    done: bool,
}

#[derive(Debug, PartialEq)]
struct SqlxCrudSummary {
    table_exists_after_create: bool,
    created: TodoSnapshot,
    loaded: TodoSnapshot,
    updated: TodoSnapshot,
    deleted_rows: u64,
}

async fn run_sqlx_crud(database_url: &str) -> Result<SqlxCrudSummary> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(database_url)
        .await
        .context("connect SQLx PostgreSQL database")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS demo_02_sqlx_todo (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await
    .context("create SQLx demo table")?;

    let table_exists_after_create: bool =
        sqlx::query_scalar("SELECT to_regclass('demo_02_sqlx_todo') IS NOT NULL")
            .fetch_one(&pool)
            .await
            .context("verify SQLx demo table exists")?;

    let id = format!("sqlx-{}", uuid::Uuid::new_v4());
    let created = sqlx::query_as::<_, (String, String, bool)>(
        r#"
        INSERT INTO demo_02_sqlx_todo (id, title, done)
        VALUES ($1, $2, $3)
        RETURNING id, title, done
        "#,
    )
    .bind(&id)
    .bind("learn SQLx with PostgreSQL")
    .bind(false)
    .fetch_one(&pool)
    .await
    .context("insert SQLx todo")?;
    println!("created via SQLx: {created:?}");
    let created = TodoSnapshot {
        id: created.0,
        title: created.1,
        done: created.2,
    };

    let loaded = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT id, title, done FROM demo_02_sqlx_todo WHERE id = $1",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await
    .context("select SQLx todo")?;
    println!("loaded via SQLx: {loaded:?}");
    let loaded = TodoSnapshot {
        id: loaded.0,
        title: loaded.1,
        done: loaded.2,
    };

    let updated = sqlx::query_as::<_, (String, String, bool)>(
        r#"
        UPDATE demo_02_sqlx_todo
        SET title = $2, done = $3, updated_at = NOW()
        WHERE id = $1
        RETURNING id, title, done
        "#,
    )
    .bind(&id)
    .bind("learn SQLx CRUD with PostgreSQL")
    .bind(true)
    .fetch_one(&pool)
    .await
    .context("update SQLx todo")?;
    println!("updated via SQLx: {updated:?}");
    let updated = TodoSnapshot {
        id: updated.0,
        title: updated.1,
        done: updated.2,
    };

    let deleted = sqlx::query("DELETE FROM demo_02_sqlx_todo WHERE id = $1")
        .bind(&id)
        .execute(&pool)
        .await
        .context("delete SQLx todo")?
        .rows_affected();
    println!("deleted via SQLx: {deleted}");

    Ok(SqlxCrudSummary {
        table_exists_after_create,
        created,
        loaded,
        updated,
        deleted_rows: deleted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlx_crud_creates_table_and_round_trips_todo() -> Result<()> {
        let Some(database_url) = database_url_for_test() else {
            eprintln!("skipping PostgreSQL CRUD test: set SQLX_STUDY_DATABASE_URL or DATABASE_URL");
            return Ok(());
        };

        let summary = run_sqlx_crud(&database_url).await?;

        // The test must prove the demo creates its own table before data writes.
        assert!(summary.table_exists_after_create);
        assert_eq!(
            summary.created,
            TodoSnapshot {
                id: summary.created.id.clone(),
                title: "learn SQLx with PostgreSQL".to_string(),
                done: false,
            }
        );
        assert_eq!(summary.loaded, summary.created);
        assert_eq!(
            summary.updated,
            TodoSnapshot {
                id: summary.created.id.clone(),
                title: "learn SQLx CRUD with PostgreSQL".to_string(),
                done: true,
            }
        );
        // Delete must affect exactly the row inserted by this test run.
        assert_eq!(summary.deleted_rows, 1);

        Ok(())
    }

    fn database_url_for_test() -> Option<String> {
        std::env::var("SQLX_STUDY_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .ok()
    }
}
