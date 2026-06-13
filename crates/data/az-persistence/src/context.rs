use std::{
    env,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use anyhow::{Context, Result};
use az_derive_aliases::{apply, plain_clone};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr};
use sea_orm_migration::prelude::MigratorTrait;

use crate::{
    env_file::{LOCAL_ENV_FILE, read_database_url_from_path, workspace_env_path_from},
    migration::WorkspaceMigrator,
};

static WORKSPACE_MIGRATIONS_DONE: AtomicBool = AtomicBool::new(false);
static WORKSPACE_MIGRATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Shared database context for workspace services.
#[apply(plain_clone)]
pub struct PersistenceContext {
    database_url: String,
    db: DatabaseConnection,
}

impl PersistenceContext {
    pub async fn connect() -> Result<Self> {
        let database_url = database_url()
            .context("missing MSC_AIO_DATABASE_URL / DATABASE_URL / ~/.config/aio/aio.env")?;
        Self::connect_with_url(&database_url).await
    }

    pub async fn connect_with_url(database_url: &str) -> Result<Self> {
        let mut options = ConnectOptions::new(database_url.to_owned());
        options
            .max_connections(8)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .sqlx_logging(false);

        let db = Database::connect(options)
            .await
            .context("connect to postgres")?;

        if !WORKSPACE_MIGRATIONS_DONE.load(Ordering::Acquire) {
            let _guard = WORKSPACE_MIGRATION_LOCK.lock().await;
            if !WORKSPACE_MIGRATIONS_DONE.load(Ordering::Acquire) {
                cleanup_invalid_migration_records(&db)
                    .await
                    .context("run workspace migrations")?;
                match WorkspaceMigrator::up(&db, None).await {
                    Ok(()) => {}
                    Err(err) if is_concurrent_migration_conflict(&err) => {}
                    Err(err) => {
                        let error = anyhow::Error::from(err).context("run workspace migrations");

                        return Err(error);
                    }
                }
                WORKSPACE_MIGRATIONS_DONE.store(true, Ordering::Release);
            }
        }

        db.execute_unprepared("SELECT 1")
            .await
            .context("ping postgres")?;

        Ok(Self {
            database_url: database_url.to_owned(),
            db,
        })
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn into_connection(self) -> DatabaseConnection {
        self.db
    }
}

pub fn database_url() -> Option<String> {
    env::var("MSC_AIO_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(read_database_url_from_local_env)
}

#[deprecated(note = "AIO desktop configuration is stored in ~/.config/aio/aio.env")]
pub fn workspace_env_path() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    workspace_env_path_from(&cwd)
}

pub fn local_env_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(LOCAL_ENV_FILE))
}

fn read_database_url_from_local_env() -> Option<String> {
    let path = local_env_path()?;
    read_database_url_from_path(&path)
}

fn home_dir() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn is_concurrent_migration_conflict(err: &DbErr) -> bool {
    let message = err.to_string();
    message.contains("seaql_migrations_pkey")
        || message.contains("duplicate key value violates unique constraint")
            && message.contains("seaql_migrations")
}

async fn cleanup_invalid_migration_records(db: &DatabaseConnection) -> Result<(), DbErr> {
    WorkspaceMigrator::install(db).await?;
    db.execute_unprepared("DELETE FROM seaql_migrations WHERE version = 'lib'")
        .await?;
    Ok(())
}
