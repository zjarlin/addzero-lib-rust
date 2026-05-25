#![forbid(unsafe_code)]

use anyhow::{Context, Result};

#[derive(Debug, toasty::Model)]
pub struct StudyTodo {
    #[key]
    pub id: String,
    pub title: String,
    pub done: bool,
}

pub fn database_url() -> Result<String> {
    Ok(std::env::var("TOASTY_DATABASE_URL")?)
}

pub async fn run_toasty_crud(database_url: &str) -> Result<()> {
    let mut db = toasty::Db::builder()
        .models(toasty::models!(StudyTodo))
        .table_name_prefix("demo_01_")
        .connect(database_url)
        .await
        .context("connect Toasty PostgreSQL database")?;

    db.push_schema().await.context("push Toasty schema")?;

    let id = format!("toasty-{}", uuid::Uuid::new_v4());
    let created = StudyTodo::create()
        .id(id.clone())
        .title("learn Toasty with PostgreSQL")
        .done(false)
        .exec(&mut db)
        .await
        .context("create Toasty todo")?;
    println!("created via Toasty: {created:?}");

    let loaded = StudyTodo::filter(StudyTodo::fields().id().eq(&id))
        .one()
        .exec(&mut db)
        .await
        .context("read Toasty todo")?;
    println!("loaded via Toasty: {loaded:?}");

    StudyTodo::filter(StudyTodo::fields().id().eq(&id))
        .update()
        .title("learn Toasty CRUD with PostgreSQL")
        .done(true)
        .exec(&mut db)
        .await
        .context("update Toasty todo")?;

    let updated = StudyTodo::filter(StudyTodo::fields().id().eq(&id))
        .one()
        .exec(&mut db)
        .await
        .context("read updated Toasty todo")?;
    println!("updated via Toasty: {updated:?}");

    StudyTodo::filter(StudyTodo::fields().id().eq(&id))
        .delete()
        .exec(&mut db)
        .await
        .context("delete Toasty todo")?;

    let remaining = StudyTodo::filter(StudyTodo::fields().id().eq(&id))
        .count()
        .exec(&mut db)
        .await
        .context("count deleted Toasty todo")?;
    println!("remaining via Toasty: {remaining}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_url_missing_env_returns_err() {
        let err = database_url().unwrap_err();
        let message = err.to_string();

        assert!(
            message.contains("environment variable not found") || message.contains("TOASTY_DATABASE_URL"),
            "unexpected error: {message}"
        );
    }

    #[tokio::test]
    async fn run_toasty_crud_rejects_invalid_database_url() {
        let err = run_toasty_crud("invalid-connection-string").await.unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("connect Toasty PostgreSQL database") || msg.contains("invalid"),
            "unexpected error: {msg}"
        );
    }
}
