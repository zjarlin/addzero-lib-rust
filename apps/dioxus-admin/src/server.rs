//! Server-side bootstrap. Lazily attaches to Postgres on the first server
//! function call and triggers an initial reconcile against the local
//! `~/.agents/skills` tree.
#![cfg(feature = "server")]

use addzero_skills::{FsRepo, SkillService};
use tokio::sync::OnceCell;

static SERVICE: OnceCell<SkillService> = OnceCell::const_new();

pub async fn service() -> &'static SkillService {
    SERVICE
        .get_or_init(|| async {
            let fs = FsRepo::default_root().unwrap_or_else(|err| {
                log::warn!("could not resolve fs root, falling back to ./skills: {err:?}");
                FsRepo::new(std::path::PathBuf::from("./skills"))
            });
            let database_url = std::env::var("DATABASE_URL").ok();
            let svc = SkillService::try_attach(database_url.as_deref(), fs).await;
            if svc.is_pg_online() {
                if let Err(err) = svc.sync_now().await {
                    log::warn!("initial sync failed: {err:?}");
                }
            }
            svc
        })
        .await
}
